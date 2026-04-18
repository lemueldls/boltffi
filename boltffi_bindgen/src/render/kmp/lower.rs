use boltffi_ffi_rules::callable::ExecutionKind;

use crate::ir::abi::CallId;
use crate::ir::definitions::{ConstructorDef, EnumRepr, Receiver, ReturnDef, VariantPayload};
use crate::ir::types::TypeExpr;
use crate::ir::{AbiContract, FfiContract};
use crate::render::kotlin::NamingConvention;

use super::plan::{
    KmpCallback, KmpCallbackMethod, KmpClass, KmpClassConstructor, KmpClassFactory, KmpClassMethod,
    KmpEnum, KmpEnumKind, KmpEnumVariant, KmpFunction, KmpModule, KmpParam, KmpRecord,
};

pub struct KmpLowerer<'a> {
    _ffi_contract: &'a FfiContract,
    _abi_contract: &'a AbiContract,
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
            _ffi_contract: ffi_contract,
            _abi_contract: abi_contract,
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
            ._ffi_contract
            .catalog
            .all_records()
            .map(|record| KmpRecord {
                name: NamingConvention::class_name(record.id.as_str()),
                fields: record
                    .fields
                    .iter()
                    .map(|field| KmpParam {
                        name: NamingConvention::property_name(field.name.as_str()),
                        kotlin_type: self.kotlin_type(&field.type_expr),
                    })
                    .collect(),
            })
            .collect();

        let enums = self
            ._ffi_contract
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
            ._ffi_contract
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
            ._ffi_contract
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
            ._ffi_contract
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
        self._abi_contract
            .calls
            .iter()
            .find(|call| call.id == call_id)
            .map(|call| call.symbol.as_str().to_string())
            .unwrap_or_else(|| "boltffi_missing_symbol".to_string())
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
