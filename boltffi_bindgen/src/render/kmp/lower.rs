use boltffi_ffi_rules::callable::ExecutionKind;
use heck::ToLowerCamelCase;

use crate::ir::{
    AbiContract,
    abi::{AbiCall, CallId, CallMode},
    contract::FfiContract,
    definitions::{FunctionDef, ReturnDef},
    ids::BuiltinId,
    types::{PrimitiveType, TypeExpr},
};
use crate::render::kotlin::NamingConvention;

use super::plan::{KmpFunction, KmpModule, KmpParam};

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

        KmpModule { functions }
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
