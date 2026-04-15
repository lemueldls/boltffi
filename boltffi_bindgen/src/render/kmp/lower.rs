use boltffi_ffi_rules::callable::ExecutionKind;
use heck::ToLowerCamelCase;

use crate::ir::{
    AbiContract,
    abi::{AbiCall, CallId, CallMode, ErrorTransport, ParamRole},
    contract::FfiContract,
    definitions::{FunctionDef, ReturnDef},
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
        let mut skipped_functions = Vec::new();

        for function in &self.contract.functions {
            let Some(call) = call_map.get(function.id.as_str()) else {
                skipped_functions.push(format!("{} (missing ABI call)", function.id.as_str()));
                continue;
            };

            match self.supported_function(function, call) {
                Some(kmp_function) => functions.push(kmp_function),
                None => skipped_functions.push(function.id.as_str().to_string()),
            }
        }

        KmpModule {
            functions,
            skipped_functions,
        }
    }

    fn supported_function(&self, function: &FunctionDef, call: &AbiCall) -> Option<KmpFunction> {
        if function.execution_kind() != ExecutionKind::Sync {
            return None;
        }

        if !matches!(call.mode, CallMode::Sync) {
            return None;
        }

        if !matches!(call.error, ErrorTransport::None) {
            return None;
        }

        if call.params.len() != function.params.len() {
            return None;
        }

        if call
            .params
            .iter()
            .any(|param| !matches!(param.role, ParamRole::Input { .. }))
        {
            return None;
        }

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            let TypeExpr::Primitive(primitive) = &param.type_expr else {
                return None;
            };

            params.push(KmpParam {
                name: NamingConvention::param_name(param.name.as_str()),
                kotlin_type: Self::kotlin_primitive_type(*primitive),
            });
        }

        let return_type = match &function.returns {
            ReturnDef::Void => None,
            ReturnDef::Value(TypeExpr::Primitive(primitive)) => {
                Some(Self::kotlin_primitive_type(*primitive))
            }
            _ => return None,
        };

        Some(KmpFunction {
            public_name: function.id.as_str().to_lower_camel_case(),
            ffi_symbol: call.symbol.as_str().to_string(),
            params,
            return_type,
        })
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
}
