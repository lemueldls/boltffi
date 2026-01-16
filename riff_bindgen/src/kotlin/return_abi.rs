use crate::model::{Module, Primitive, ReturnType, Type};
use super::wire;
use super::NamingConvention;
use super::TypeMapper;

#[derive(Debug, Clone)]
pub enum ReturnAbi {
    Unit,
    Direct { kotlin_type: String },
    WireEncoded { kotlin_type: String, decode_expr: String, throws: bool },
}

impl ReturnAbi {
    pub fn from_return_type(returns: &ReturnType, module: &Module) -> Self {
        match returns {
            ReturnType::Void => Self::Unit,
            ReturnType::Value(ty) => Self::from_value_type(ty, module),
            ReturnType::Fallible { ok, err } => Self::from_fallible(ok, err, module),
        }
    }

    fn from_value_type(ty: &Type, module: &Module) -> Self {
        match ty {
            Type::Void => Self::Unit,
            Type::Primitive(_) => Self::Direct {
                kotlin_type: TypeMapper::map_type(ty),
            },
            Type::String | Type::Record(_) | Type::Enum(_) | Type::Vec(_) | Type::Option(_) | Type::Bytes => {
                Self::WireEncoded {
                    kotlin_type: TypeMapper::map_type(ty),
                    decode_expr: Self::decode_at_zero(ty, module),
                    throws: false,
                }
            }
            _ => Self::Direct {
                kotlin_type: TypeMapper::map_type(ty),
            },
        }
    }

    fn from_fallible(ok: &Type, err: &Type, module: &Module) -> Self {
        let ok_kotlin = TypeMapper::map_type(ok);
        let err_kotlin = Self::error_type_name(err, module);

        Self::WireEncoded {
            kotlin_type: if ok.is_void() { "Unit".into() } else { ok_kotlin },
            decode_expr: Self::result_decode_expr(ok, err, &err_kotlin, module),
            throws: true,
        }
    }

    fn decode_at_zero(ty: &Type, module: &Module) -> String {
        let codec = wire::decode_type(ty, module);
        codec.value_at("0")
    }

    fn result_decode_expr(ok: &Type, err: &Type, err_kotlin: &str, module: &Module) -> String {
        let ok_decode = if ok.is_void() {
            "Unit to 0".into()
        } else {
            let codec = wire::decode_type(ok, module);
            codec.as_lambda_reader()
        };

        let err_decode = Self::error_decode_lambda(err, err_kotlin, module);

        format!(
            "wire.readResult(0, {}, {}).also {{ (result, _) -> result.getOrThrow() }}.first.getOrThrow()",
            ok_decode, err_decode
        )
    }

    fn error_type_name(err: &Type, module: &Module) -> String {
        match err {
            Type::String => "FfiException".into(),
            Type::Enum(name) => {
                if module.enums.iter().any(|e| &e.name == name && e.is_error) {
                    NamingConvention::class_name(name)
                } else {
                    "FfiException".into()
                }
            }
            _ => "FfiException".into(),
        }
    }

    fn error_decode_lambda(err: &Type, err_kotlin: &str, module: &Module) -> String {
        match err {
            Type::String => "{ FfiException(-1, wire.readString(it).first) to wire.readString(it).second }".into(),
            Type::Enum(name) => {
                let class_name = NamingConvention::class_name(name);
                if module.is_data_enum(name) {
                    format!("{{ {}.decode(wire, it) }}", class_name)
                } else {
                    format!("{{ FfiException(-1, \"Error code: ${{{}.fromValue(wire.readI32(it))}}\") to 4 }}", class_name)
                }
            }
            _ => "{ FfiException(-1, \"Unknown error\") to 0 }".into(),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Direct { .. })
    }

    pub fn is_wire_encoded(&self) -> bool {
        matches!(self, Self::WireEncoded { .. })
    }

    pub fn throws(&self) -> bool {
        matches!(self, Self::WireEncoded { throws: true, .. })
    }

    pub fn kotlin_type(&self) -> Option<&str> {
        match self {
            Self::Unit => None,
            Self::Direct { kotlin_type } | Self::WireEncoded { kotlin_type, .. } => Some(kotlin_type),
        }
    }

    pub fn decode_expr(&self) -> &str {
        match self {
            Self::WireEncoded { decode_expr, .. } => decode_expr,
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Enumeration, Record, RecordField};

    #[test]
    fn test_unit_return() {
        let module = Module::new("test");
        let abi = ReturnAbi::from_return_type(&ReturnType::Void, &module);
        assert!(abi.is_unit());
    }

    #[test]
    fn test_primitive_return() {
        let module = Module::new("test");
        let abi = ReturnAbi::from_return_type(
            &ReturnType::Value(Type::Primitive(Primitive::I32)),
            &module,
        );
        assert!(abi.is_direct());
        assert_eq!(abi.kotlin_type(), Some("Int"));
    }

    #[test]
    fn test_string_return_is_wire_encoded() {
        let module = Module::new("test");
        let abi = ReturnAbi::from_return_type(&ReturnType::Value(Type::String), &module);
        assert!(abi.is_wire_encoded());
        assert!(!abi.throws());
    }

    #[test]
    fn test_vec_return_is_wire_encoded() {
        let module = Module::new("test");
        let ty = Type::Vec(Box::new(Type::Primitive(Primitive::I32)));
        let abi = ReturnAbi::from_return_type(&ReturnType::Value(ty), &module);
        assert!(abi.is_wire_encoded());
        assert!(abi.decode_expr().contains("readList"));
    }

    #[test]
    fn test_fallible_is_throwing() {
        let module = Module::new("test");
        let abi = ReturnAbi::from_return_type(
            &ReturnType::Fallible {
                ok: Type::String,
                err: Type::String,
            },
            &module,
        );
        assert!(abi.is_wire_encoded());
        assert!(abi.throws());
    }
}
