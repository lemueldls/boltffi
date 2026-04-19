use boltffi_ffi_rules::callable::ExecutionKind;

use crate::ir::abi::{AbiRecord, CallId};
use crate::ir::definitions::{ConstructorDef, EnumRepr, Receiver, ReturnDef, VariantPayload};
use crate::ir::ops::{OffsetExpr, ReadOp, ReadSeq};
use crate::ir::types::TypeExpr;
use crate::ir::{AbiContract, FfiContract};
use crate::render::kotlin::NamingConvention;

use super::plan::{
    KmpCallback, KmpCallbackMethod, KmpClass, KmpClassConstructor, KmpClassFactory, KmpClassMethod,
    KmpEnum, KmpEnumKind, KmpEnumVariant, KmpFunction, KmpModule, KmpParam, KmpRecord,
    KmpRecordField,
};

pub struct KmpLowerer<'a> {
    ffi_contract: &'a FfiContract,
    abi_contract: &'a AbiContract,
    package_name: String,
    module_name: String,
    library_name: String,
}

impl<'a> KmpLowerer<'a> {
    pub fn new(
        ffi_contract: &'a FfiContract,
        abi_contract: &'a AbiContract,
        package_name: String,
        module_name: String,
        library_name: String,
    ) -> Self {
        Self {
            ffi_contract,
            abi_contract,
            package_name,
            module_name,
            library_name,
        }
    }

    pub fn lower(self) -> KmpModule {
        let package_name = self.package_name.clone();
        let jvm_binding_package = format!("{}.jvmffi", package_name);
        let native_binding_package = format!("{}.native", package_name);

        let records = self
            .ffi_contract
            .catalog
            .all_records()
            .map(|record| {
                let abi_record = self.abi_record_for(&record.id);
                let is_blittable = abi_record.is_some_and(|record| record.is_blittable);
                let struct_size = abi_record.and_then(|record| record.size).unwrap_or(0);
                let offsets = abi_record.and_then(|record| self.record_field_offsets(record));

                KmpRecord {
                    name: NamingConvention::class_name(record.id.as_str()),
                    is_blittable,
                    struct_size,
                    fields: record
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let kotlin_type = self.kotlin_type(&field.type_expr);
                            let (read_method, write_method) =
                                self.wire_primitive_methods(&field.type_expr);
                            KmpRecordField {
                                name: NamingConvention::property_name(field.name.as_str()),
                                kotlin_type,
                                offset: offsets
                                    .as_ref()
                                    .and_then(|offsets| offsets.get(index).copied())
                                    .unwrap_or(0),
                                read_method,
                                write_method,
                            }
                        })
                        .collect(),
                }
            })
            .collect();

        let enums = self
            .ffi_contract
            .catalog
            .all_enums()
            .map(|enumeration| {
                let (kind, variants) = match &enumeration.repr {
                    EnumRepr::CStyle { variants, .. } => (
                        KmpEnumKind::CStyle,
                        variants
                            .iter()
                            .map(|variant| KmpEnumVariant {
                                name: NamingConvention::enum_entry_name(variant.name.as_str()),
                                fields: Vec::new(),
                            })
                            .collect(),
                    ),
                    EnumRepr::Data { variants, .. } => (
                        KmpEnumKind::Data,
                        variants
                            .iter()
                            .map(|variant| {
                                let fields = match &variant.payload {
                                    VariantPayload::Unit => Vec::new(),
                                    VariantPayload::Tuple(types) => types
                                        .iter()
                                        .enumerate()
                                        .map(|(index, type_expr)| KmpParam {
                                            name: format!("p{}", index),
                                            kotlin_type: self.kotlin_type(type_expr),
                                        })
                                        .collect(),
                                    VariantPayload::Struct(fields) => fields
                                        .iter()
                                        .map(|field| KmpParam {
                                            name: NamingConvention::property_name(
                                                field.name.as_str(),
                                            ),
                                            kotlin_type: self.kotlin_type(&field.type_expr),
                                        })
                                        .collect(),
                                };
                                KmpEnumVariant {
                                    name: NamingConvention::class_name(variant.name.as_str()),
                                    fields,
                                }
                            })
                            .collect(),
                    ),
                };

                KmpEnum {
                    name: NamingConvention::class_name(enumeration.id.as_str()),
                    kind,
                    variants,
                }
            })
            .collect();

        let callbacks = self
            .ffi_contract
            .catalog
            .all_callbacks()
            .map(|callback| KmpCallback {
                name: NamingConvention::class_name(callback.id.as_str()),
                methods: callback
                    .methods
                    .iter()
                    .map(|method| KmpCallbackMethod {
                        name: NamingConvention::method_name(method.id.as_str()),
                        params: method
                            .params
                            .iter()
                            .map(|param| KmpParam {
                                name: NamingConvention::param_name(param.name.as_str()),
                                kotlin_type: self.kotlin_type(&param.type_expr),
                            })
                            .collect(),
                        return_type: self.kotlin_return_type(&method.returns),
                        is_async: method.execution_kind == ExecutionKind::Async,
                    })
                    .collect(),
            })
            .collect();

        let classes = self
            .ffi_contract
            .catalog
            .all_classes()
            .map(|class| {
                let constructors = class
                    .constructors
                    .iter()
                    .enumerate()
                    .filter_map(|(index, constructor)| match constructor {
                        ConstructorDef::Default { .. } | ConstructorDef::NamedInit { .. } => {
                            Some((constructor.params(), index))
                        }
                        ConstructorDef::NamedFactory { .. } => None,
                    })
                    .map(|(params, index)| KmpClassConstructor {
                        params: params
                            .iter()
                            .map(|param| KmpParam {
                                name: NamingConvention::param_name(param.name.as_str()),
                                kotlin_type: self.kotlin_type(&param.type_expr),
                            })
                            .collect(),
                        ffi_symbol: self.call_symbol(CallId::Constructor {
                            class_id: class.id.clone(),
                            index,
                        }),
                    })
                    .collect();

                let factories = class
                    .constructors
                    .iter()
                    .enumerate()
                    .filter_map(|(index, constructor)| match constructor {
                        ConstructorDef::NamedFactory {
                            name,
                            is_fallible: _,
                            ..
                        } => Some((name.as_str(), constructor.params(), index, constructor)),
                        _ => None,
                    })
                    .map(|(name, params, index, constructor)| KmpClassFactory {
                        name: NamingConvention::method_name(name),
                        params: params
                            .iter()
                            .map(|param| KmpParam {
                                name: NamingConvention::param_name(param.name.as_str()),
                                kotlin_type: self.kotlin_type(&param.type_expr),
                            })
                            .collect(),
                        return_type: if constructor.is_optional() {
                            format!("{}?", NamingConvention::class_name(class.id.as_str()))
                        } else {
                            NamingConvention::class_name(class.id.as_str())
                        },
                        is_async: false,
                        ffi_symbol: self.call_symbol(CallId::Constructor {
                            class_id: class.id.clone(),
                            index,
                        }),
                    })
                    .collect();

                let methods = class
                    .methods
                    .iter()
                    .map(|method| KmpClassMethod {
                        name: NamingConvention::method_name(method.id.as_str()),
                        params: method
                            .params
                            .iter()
                            .map(|param| KmpParam {
                                name: NamingConvention::param_name(param.name.as_str()),
                                kotlin_type: self.kotlin_type(&param.type_expr),
                            })
                            .collect(),
                        return_type: self.kotlin_return_type(&method.returns),
                        is_async: method.execution_kind == ExecutionKind::Async,
                        is_static: method.receiver == Receiver::Static,
                        ffi_symbol: self.call_symbol(CallId::Method {
                            class_id: class.id.clone(),
                            method_id: method.id.clone(),
                        }),
                    })
                    .collect();

                KmpClass {
                    name: NamingConvention::class_name(class.id.as_str()),
                    constructors,
                    factories,
                    methods,
                }
            })
            .collect();

        let functions = self
            .ffi_contract
            .functions
            .iter()
            .map(|function| KmpFunction {
                name: NamingConvention::method_name(function.id.as_str()),
                params: function
                    .params
                    .iter()
                    .map(|param| KmpParam {
                        name: NamingConvention::param_name(param.name.as_str()),
                        kotlin_type: self.kotlin_type(&param.type_expr),
                    })
                    .collect(),
                return_type: self.kotlin_return_type(&function.returns),
                is_async: function.execution_kind == ExecutionKind::Async,
                ffi_symbol: self.call_symbol(CallId::Function(function.id.clone())),
            })
            .collect();

        KmpModule {
            package_name: self.package_name,
            module_name: self.module_name,
            library_name: self.library_name,
            jvm_binding_package,
            native_binding_package,
            records,
            enums,
            callbacks,
            classes,
            functions,
        }
    }

    fn call_symbol(&self, call_id: CallId) -> String {
        self.abi_contract
            .calls
            .iter()
            .find(|call| call.id == call_id)
            .map(|call| call.symbol.as_str().to_string())
            .unwrap_or_else(|| "boltffi_missing_symbol".to_string())
    }

    fn abi_record_for(&self, record_id: &crate::ir::ids::RecordId) -> Option<&AbiRecord> {
        self.abi_contract
            .records
            .iter()
            .find(|record| record.id == *record_id)
    }

    fn record_field_offsets(&self, record: &AbiRecord) -> Option<Vec<usize>> {
        match record.decode_ops.ops.first() {
            Some(ReadOp::Record { fields, .. }) => fields
                .iter()
                .map(|field| read_seq_offset(&field.seq))
                .collect(),
            _ => None,
        }
    }

    fn wire_primitive_methods(&self, type_expr: &TypeExpr) -> (String, String) {
        match type_expr {
            TypeExpr::Primitive(primitive) => match primitive {
                crate::ir::types::PrimitiveType::Bool => {
                    ("readBooleanAt".to_string(), "writeBooleanAt".to_string())
                }
                crate::ir::types::PrimitiveType::I8 => {
                    ("readByteAt".to_string(), "writeByteAt".to_string())
                }
                crate::ir::types::PrimitiveType::U8 => {
                    ("readUByteAt".to_string(), "writeUByteAt".to_string())
                }
                crate::ir::types::PrimitiveType::I16 => {
                    ("readShortAt".to_string(), "writeShortAt".to_string())
                }
                crate::ir::types::PrimitiveType::U16 => {
                    ("readUShortAt".to_string(), "writeUShortAt".to_string())
                }
                crate::ir::types::PrimitiveType::I32 => {
                    ("readIntAt".to_string(), "writeIntAt".to_string())
                }
                crate::ir::types::PrimitiveType::U32 => {
                    ("readUIntAt".to_string(), "writeUIntAt".to_string())
                }
                crate::ir::types::PrimitiveType::I64 | crate::ir::types::PrimitiveType::ISize => {
                    ("readLongAt".to_string(), "writeLongAt".to_string())
                }
                crate::ir::types::PrimitiveType::U64 | crate::ir::types::PrimitiveType::USize => {
                    ("readULongAt".to_string(), "writeULongAt".to_string())
                }
                crate::ir::types::PrimitiveType::F32 => {
                    ("readFloatAt".to_string(), "writeFloatAt".to_string())
                }
                crate::ir::types::PrimitiveType::F64 => {
                    ("readDoubleAt".to_string(), "writeDoubleAt".to_string())
                }
            },
            _ => ("readByteAt".to_string(), "writeByteAt".to_string()),
        }
    }

    fn kotlin_return_type(&self, returns: &ReturnDef) -> String {
        match returns {
            ReturnDef::Void => "Unit".to_string(),
            ReturnDef::Value(type_expr) => self.kotlin_type(type_expr),
            ReturnDef::Result { ok, .. } => self.kotlin_type(ok),
        }
    }

    fn kotlin_type(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Primitive(primitive) => {
                match primitive {
                    crate::ir::types::PrimitiveType::Bool => "Boolean".to_string(),
                    crate::ir::types::PrimitiveType::I8 => "Byte".to_string(),
                    crate::ir::types::PrimitiveType::U8 => "UByte".to_string(),
                    crate::ir::types::PrimitiveType::I16 => "Short".to_string(),
                    crate::ir::types::PrimitiveType::U16 => "UShort".to_string(),
                    crate::ir::types::PrimitiveType::I32 => "Int".to_string(),
                    crate::ir::types::PrimitiveType::U32 => "UInt".to_string(),
                    crate::ir::types::PrimitiveType::I64
                    | crate::ir::types::PrimitiveType::ISize => "Long".to_string(),
                    crate::ir::types::PrimitiveType::U64
                    | crate::ir::types::PrimitiveType::USize => "ULong".to_string(),
                    crate::ir::types::PrimitiveType::F32 => "Float".to_string(),
                    crate::ir::types::PrimitiveType::F64 => "Double".to_string(),
                }
            }
            TypeExpr::String => "String".to_string(),
            TypeExpr::Bytes => "ByteArray".to_string(),
            TypeExpr::Vec(inner) => format!("List<{}>", self.kotlin_type(inner)),
            TypeExpr::Option(inner) => format!("{}?", self.kotlin_type(inner)),
            TypeExpr::Result { ok, .. } => format!("Result<{}>", self.kotlin_type(ok)),
            TypeExpr::Record(record_id) => NamingConvention::class_name(record_id.as_str()),
            TypeExpr::Enum(enum_id) => NamingConvention::class_name(enum_id.as_str()),
            TypeExpr::Custom(custom_id) => NamingConvention::class_name(custom_id.as_str()),
            TypeExpr::Builtin(builtin_id) => match builtin_id.as_str() {
                "Duration" => "Duration".to_string(),
                "SystemTime" => "Instant".to_string(),
                "Uuid" => "UUID".to_string(),
                "Url" => "URI".to_string(),
                _ => "String".to_string(),
            },
            TypeExpr::Handle(class_id) => NamingConvention::class_name(class_id.as_str()),
            TypeExpr::Callback(callback_id) => NamingConvention::class_name(callback_id.as_str()),
            TypeExpr::Void => "Unit".to_string(),
        }
    }
}

fn read_seq_offset(seq: &ReadSeq) -> Option<usize> {
    let op = seq.ops.first()?;
    let offset = match op {
        ReadOp::Primitive { offset, .. }
        | ReadOp::String { offset }
        | ReadOp::Bytes { offset }
        | ReadOp::Builtin { offset, .. }
        | ReadOp::Record { offset, .. }
        | ReadOp::Enum { offset, .. } => offset,
        _ => return None,
    };

    match offset {
        OffsetExpr::Fixed(value) => Some(*value),
        OffsetExpr::Base => Some(0),
        OffsetExpr::BasePlus(value) => Some(*value),
        _ => None,
    }
}
