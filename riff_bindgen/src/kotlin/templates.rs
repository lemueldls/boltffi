use askama::Template;
use riff_ffi_rules::naming;

use crate::model::{Enumeration, Function, Module, Record, Type};

use super::marshal::{ParamConversion, ReturnKind};
use super::{NamingConvention, TypeMapper};

#[derive(Template)]
#[template(path = "kotlin/preamble.txt", escape = "none")]
pub struct PreambleTemplate {
    pub package_name: String,
}

impl PreambleTemplate {
    pub fn from_module(module: &Module) -> Self {
        Self {
            package_name: NamingConvention::class_name(&module.name).to_lowercase(),
        }
    }
}

#[derive(Template)]
#[template(path = "kotlin/enum_c_style.txt", escape = "none")]
pub struct CStyleEnumTemplate {
    pub class_name: String,
    pub variants: Vec<EnumVariantView>,
}

pub struct EnumVariantView {
    pub name: String,
    pub value: i64,
}

impl CStyleEnumTemplate {
    pub fn from_enum(enumeration: &Enumeration) -> Self {
        let variants = enumeration
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| EnumVariantView {
                name: NamingConvention::enum_entry_name(&variant.name),
                value: variant.discriminant.unwrap_or(index as i64),
            })
            .collect();

        Self {
            class_name: NamingConvention::class_name(&enumeration.name),
            variants,
        }
    }
}

#[derive(Template)]
#[template(path = "kotlin/enum_sealed.txt", escape = "none")]
pub struct SealedEnumTemplate {
    pub class_name: String,
    pub variants: Vec<SealedVariantView>,
}

pub struct SealedVariantView {
    pub name: String,
    pub fields: Vec<FieldView>,
}

impl SealedEnumTemplate {
    pub fn from_enum(enumeration: &Enumeration) -> Self {
        let variants = enumeration
            .variants
            .iter()
            .map(|variant| SealedVariantView {
                name: NamingConvention::class_name(&variant.name),
                fields: variant
                    .fields
                    .iter()
                    .map(|field| FieldView {
                        name: NamingConvention::property_name(&field.name),
                        kotlin_type: TypeMapper::map_type(&field.field_type),
                    })
                    .collect(),
            })
            .collect();

        Self {
            class_name: NamingConvention::class_name(&enumeration.name),
            variants,
        }
    }
}

#[derive(Template)]
#[template(path = "kotlin/record.txt", escape = "none")]
pub struct RecordTemplate {
    pub class_name: String,
    pub fields: Vec<FieldView>,
}

pub struct FieldView {
    pub name: String,
    pub kotlin_type: String,
}

impl RecordTemplate {
    pub fn from_record(record: &Record) -> Self {
        let fields = record
            .fields
            .iter()
            .map(|field| FieldView {
                name: NamingConvention::property_name(&field.name),
                kotlin_type: TypeMapper::map_type(&field.field_type),
            })
            .collect();

        Self {
            class_name: NamingConvention::class_name(&record.name),
            fields,
        }
    }
}

#[derive(Template)]
#[template(path = "kotlin/function.txt", escape = "none")]
pub struct FunctionTemplate {
    pub func_name: String,
    pub ffi_name: String,
    pub prefix: String,
    pub params: Vec<ParamView>,
    pub return_type: Option<String>,
    pub return_kind: ReturnKind,
    pub inner_type: Option<String>,
    pub len_fn: Option<String>,
    pub copy_fn: Option<String>,
}

pub struct ParamView {
    pub name: String,
    pub kotlin_type: String,
    pub conversion: String,
}

impl FunctionTemplate {
    pub fn from_function(function: &Function) -> Self {
        let ffi_name = format!("{}_{}", naming::ffi_prefix(), function.name);
        let return_kind = function
            .output
            .as_ref()
            .map(|ty| ReturnKind::from_type(ty, &ffi_name))
            .unwrap_or(ReturnKind::Void);

        let params: Vec<ParamView> = function
            .inputs
            .iter()
            .map(|param| ParamView {
                name: NamingConvention::param_name(&param.name),
                kotlin_type: TypeMapper::map_type(&param.param_type),
                conversion: ParamConversion::to_ffi(
                    &NamingConvention::param_name(&param.name),
                    &param.param_type,
                ),
            })
            .collect();

        let return_type = function.output.as_ref().map(TypeMapper::map_type);
        let inner_type = return_kind.inner_type().map(String::from);
        let len_fn = return_kind.len_fn().map(String::from);
        let copy_fn = return_kind.copy_fn().map(String::from);

        Self {
            func_name: NamingConvention::method_name(&function.name),
            ffi_name,
            prefix: naming::ffi_prefix().to_string(),
            params,
            return_type,
            return_kind,
            inner_type,
            len_fn,
            copy_fn,
        }
    }
}
