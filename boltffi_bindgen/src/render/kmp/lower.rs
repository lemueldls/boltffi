use boltffi_ffi_rules::callable::ExecutionKind;
use heck::ToLowerCamelCase;

use crate::ir::{
    AbiContract,
    abi::{AbiCall, AbiEnum, AbiEnumField, AbiEnumPayload, CallId, CallMode},
    contract::FfiContract,
    definitions::{
        ClassDef, DefaultValue, EnumDef, FieldDef, FunctionDef, MethodDef, RecordDef, ReturnDef,
    },
    ids::BuiltinId,
    types::{PrimitiveType, TypeExpr},
};
use crate::render::kotlin::NamingConvention;
use boltffi_ffi_rules::transport::EnumTagStrategy;

use super::plan::{
    KmpClass, KmpClassConstructor, KmpClassMethod, KmpEnum, KmpEnumField, KmpEnumVariant,
    KmpFunction, KmpModule, KmpParam, KmpRecord, KmpRecordField,
};

pub struct KmpLowerer<'a> {
    contract: &'a FfiContract,
    abi: &'a AbiContract,
}

impl<'a> KmpLowerer<'a> {
    pub fn new(contract: &'a FfiContract, abi: &'a AbiContract) -> Self {
        Self { contract, abi }
    }

    pub fn lower(&self) -> KmpModule {
        let mut call_map = std::collections::HashMap::new();
        for call in &self.abi.calls {
            if let CallId::Function(function_id) = &call.id {
                call_map.insert(function_id.as_str(), call);
            }
        }

        let records = self
            .contract
            .catalog
            .all_records()
            .map(|record| self.lower_record(record))
            .collect::<Vec<_>>();
        let enums = self
            .contract
            .catalog
            .all_enums()
            .map(|enumeration| self.lower_enum(enumeration))
            .collect::<Vec<_>>();
        let classes = self
            .contract
            .catalog
            .all_classes()
            .map(|class| self.lower_class(class))
            .collect::<Vec<_>>();
        let mut functions = Vec::new();

        for function in &self.contract.functions {
            let Some(call) = call_map.get(function.id.as_str()) else {
                continue;
            };

            match self.supported_function(function, call) {
                Some(kmp_function) => functions.push(kmp_function),
                None => continue,
            }
        }

        KmpModule {
            records,
            enums,
            classes,
            functions,
        }
    }

    fn supported_function(&self, function: &FunctionDef, call: &AbiCall) -> Option<KmpFunction> {
        if call.params.len() != function.params.len() {
            return None;
        }

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            params.push(KmpParam {
                name: NamingConvention::param_name(param.name.as_str()),
                kotlin_type: self.kotlin_type(&param.type_expr),
            });
        }

        let return_type = match &function.returns {
            ReturnDef::Void => None,
            ReturnDef::Value(type_expr) => Some(self.kotlin_type(type_expr)),
            ReturnDef::Result { ok, err } => Some(format!(
                "BoltFFIResult<{}, {}>",
                self.kotlin_type(ok),
                self.kotlin_type(err)
            )),
        };

        Some(KmpFunction {
            public_name: function.id.as_str().to_lower_camel_case(),
            ffi_symbol: call.symbol.as_str().to_string(),
            params,
            return_type,
            is_async: function.execution_kind() == ExecutionKind::Async
                || matches!(call.mode, CallMode::Async(_)),
        })
    }

    fn kotlin_type(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Void => "Unit".to_string(),
            TypeExpr::Primitive(primitive) => Self::kotlin_primitive_type(*primitive),
            TypeExpr::String => "String".to_string(),
            TypeExpr::Bytes => "ByteArray".to_string(),
            TypeExpr::Vec(inner) => format!("List<{}>", self.kotlin_type(inner)),
            TypeExpr::Option(inner) => format!("{}?", self.kotlin_type(inner)),
            TypeExpr::Result { ok, err } => format!(
                "BoltFFIResult<{}, {}>",
                self.kotlin_type(ok),
                self.kotlin_type(err)
            ),
            TypeExpr::Record(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Enum(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Handle(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Callback(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Custom(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Builtin(id) => self.kotlin_builtin_type(id),
        }
    }

    fn kotlin_primitive_type(primitive: PrimitiveType) -> String {
        match primitive {
            PrimitiveType::Bool => "Boolean".to_string(),
            PrimitiveType::I8 => "Byte".to_string(),
            PrimitiveType::U8 => "UByte".to_string(),
            PrimitiveType::I16 => "Short".to_string(),
            PrimitiveType::U16 => "UShort".to_string(),
            PrimitiveType::I32 => "Int".to_string(),
            PrimitiveType::U32 => "UInt".to_string(),
            PrimitiveType::I64 | PrimitiveType::ISize => "Long".to_string(),
            PrimitiveType::U64 | PrimitiveType::USize => "ULong".to_string(),
            PrimitiveType::F32 => "Float".to_string(),
            PrimitiveType::F64 => "Double".to_string(),
        }
    }

    fn kotlin_builtin_type(&self, builtin_id: &BuiltinId) -> String {
        match builtin_id.as_str() {
            "Duration" => "Duration".to_string(),
            "SystemTime" => "Instant".to_string(),
            "Uuid" => "UUID".to_string(),
            "Url" => "URI".to_string(),
            _ => NamingConvention::class_name(builtin_id.as_str()),
        }
    }

    fn lower_record(&self, record: &RecordDef) -> KmpRecord {
        KmpRecord {
            class_name: NamingConvention::class_name(record.id.as_str()),
            fields: record
                .fields
                .iter()
                .map(|field| self.lower_record_field(field))
                .collect::<Vec<_>>(),
            doc: record.doc.clone(),
        }
    }

    fn lower_record_field(&self, field: &FieldDef) -> KmpRecordField {
        let kotlin_type = self.kotlin_type(&field.type_expr);
        KmpRecordField {
            name: NamingConvention::param_name(field.name.as_str()),
            kotlin_type: kotlin_type.clone(),
            default_value: field
                .default
                .as_ref()
                .map(|default| Self::kotlin_default_literal(default, &kotlin_type)),
        }
    }

    fn kotlin_default_literal(default: &DefaultValue, kotlin_type: &str) -> String {
        use heck::ToUpperCamelCase;

        match default {
            DefaultValue::Bool(true) => "true".to_string(),
            DefaultValue::Bool(false) => "false".to_string(),
            DefaultValue::Integer(v) => match kotlin_type {
                "Double" => format!("{}.0", v),
                "Float" => format!("{}.0f", v),
                "UInt" => format!("{}u", v),
                "ULong" => format!("{}uL", v),
                "UShort" => format!("{}u", v),
                "UByte" => format!("{}u", v),
                "Long" => format!("{}L", v),
                _ => v.to_string(),
            },
            DefaultValue::Float(v) => {
                let has_decimal = v.fract() != 0.0;
                let base = if has_decimal {
                    format!("{}", v)
                } else {
                    format!("{}.0", v)
                };
                match kotlin_type {
                    "Float" => format!("{}f", base),
                    _ => base,
                }
            }
            DefaultValue::String(v) => format!("\"{}\"", v),
            DefaultValue::EnumVariant {
                enum_name,
                variant_name,
            } => format!(
                "{}.{}",
                enum_name.to_upper_camel_case(),
                NamingConvention::enum_entry_name(variant_name)
            ),
            DefaultValue::Null => "null".to_string(),
        }
    }

    fn abi_enum_for(&self, enumeration: &EnumDef) -> &AbiEnum {
        self.abi
            .enums
            .iter()
            .find(|abi_enum| abi_enum.id == enumeration.id)
            .expect("abi enum missing")
    }

    fn lower_enum(&self, enumeration: &EnumDef) -> KmpEnum {
        let abi_enum = self.abi_enum_for(enumeration);
        let class_name = NamingConvention::class_name(enumeration.id.as_str());
        let is_c_style = abi_enum.is_c_style;
        let is_error = enumeration.is_error;
        let value_type = match &enumeration.repr {
            crate::ir::definitions::EnumRepr::CStyle { tag_type, .. } => {
                Some(Self::kotlin_primitive_type(*tag_type))
            }
            _ => None,
        };
        let variant_docs = enumeration.variant_docs();
        let variants = abi_enum
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let mut lowered = self.lower_enum_variant(abi_enum, variant, index, is_c_style);
                lowered.doc = variant_docs.get(index).cloned().flatten();
                lowered
            })
            .collect::<Vec<_>>();

        KmpEnum {
            class_name,
            is_c_style,
            is_error,
            value_type,
            variants,
            doc: enumeration.doc.clone(),
        }
    }

    fn lower_enum_variant(
        &self,
        abi_enum: &AbiEnum,
        variant: &crate::ir::abi::AbiEnumVariant,
        ordinal: usize,
        is_c_style: bool,
    ) -> KmpEnumVariant {
        let fields = match &variant.payload {
            AbiEnumPayload::Unit => Vec::new(),
            AbiEnumPayload::Tuple(fields) | AbiEnumPayload::Struct(fields) => fields
                .iter()
                .enumerate()
                .map(|(index, field)| self.lower_enum_field(field, index))
                .collect(),
        };
        let name = if is_c_style {
            NamingConvention::enum_entry_name(variant.name.as_str())
        } else {
            NamingConvention::class_name(variant.name.as_str())
        };

        KmpEnumVariant {
            name,
            tag: self.kotlin_enum_variant_tag(abi_enum, ordinal, variant.discriminant),
            fields,
            doc: None,
        }
    }

    fn lower_enum_field(&self, field: &AbiEnumField, index: usize) -> KmpEnumField {
        KmpEnumField {
            name: if field.name.as_str().is_empty() {
                format!("value_{}", index)
            } else {
                NamingConvention::property_name(field.name.as_str())
            },
            kotlin_type: self.kotlin_type(&field.type_expr),
        }
    }

    fn kotlin_enum_variant_tag(
        &self,
        abi_enum: &AbiEnum,
        ordinal: usize,
        discriminant: i128,
    ) -> i128 {
        match abi_enum.codec_tag_strategy {
            EnumTagStrategy::Discriminant => discriminant,
            EnumTagStrategy::OrdinalIndex => abi_enum.resolve_codec_tag(ordinal, discriminant),
        }
    }

    fn lower_class(&self, class: &ClassDef) -> KmpClass {
        KmpClass {
            class_name: NamingConvention::class_name(class.id.as_str()),
            doc: class.doc.clone(),
            constructors: class
                .constructors
                .iter()
                .map(|constructor| KmpClassConstructor {
                    params: constructor
                        .params()
                        .into_iter()
                        .map(|param| KmpParam {
                            name: NamingConvention::param_name(param.name.as_str()),
                            kotlin_type: self.kotlin_type(&param.type_expr),
                        })
                        .collect::<Vec<_>>(),
                    doc: constructor.doc().map(String::from),
                })
                .collect::<Vec<_>>(),
            methods: class
                .methods
                .iter()
                .map(|method| self.lower_class_method(method))
                .collect::<Vec<_>>(),
        }
    }

    fn lower_class_method(&self, method: &MethodDef) -> KmpClassMethod {
        KmpClassMethod {
            name: NamingConvention::method_name(method.id.as_str()),
            params: method
                .params
                .iter()
                .map(|param| KmpParam {
                    name: NamingConvention::param_name(param.name.as_str()),
                    kotlin_type: self.kotlin_type(&param.type_expr),
                })
                .collect::<Vec<_>>(),
            return_type: match &method.returns {
                ReturnDef::Void => None,
                ReturnDef::Value(type_expr) => Some(self.kotlin_type(type_expr)),
                ReturnDef::Result { ok, err } => Some(format!(
                    "BoltFFIResult<{}, {}>",
                    self.kotlin_type(ok),
                    self.kotlin_type(err)
                )),
            },
            is_async: method.execution_kind() == ExecutionKind::Async,
            doc: method.doc.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::ir::{Lowerer, build_contract};
    use crate::scan::scan_crate_with_pointer_width;

    use super::KmpLowerer;

    fn demo_source_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/demo")
    }

    #[test]
    fn lower_demo_contract_collects_supported_functions() {
        let mut scanned_module =
            scan_crate_with_pointer_width(&demo_source_directory(), "demo", None)
                .expect("demo crate should scan");
        let ffi_contract = build_contract(&mut scanned_module);
        let abi_contract = Lowerer::new(&ffi_contract).to_abi_contract();

        let module = KmpLowerer::new(&ffi_contract, &abi_contract).lower();

        assert!(!module.records.is_empty());
        assert!(!module.enums.is_empty());
        assert!(!module.classes.is_empty());
        assert!(!module.functions.is_empty());
        assert!(
            module
                .functions
                .iter()
                .any(|function| function.public_name == "echoI32")
        );
        assert!(
            module
                .functions
                .iter()
                .all(|function| !function.public_name.is_empty())
        );
    }
}
