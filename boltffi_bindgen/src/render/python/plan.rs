use crate::ir::types::PrimitiveType;
use crate::render::python::primitives::PythonScalarTypeExt as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonSequenceType {
    Bytes,
    PrimitiveVec(PrimitiveType),
}

impl PythonSequenceType {
    pub fn parameter_annotation(&self) -> String {
        match self {
            Self::Bytes => "bytes".to_string(),
            Self::PrimitiveVec(PrimitiveType::U8) => "bytes | Sequence[int]".to_string(),
            Self::PrimitiveVec(primitive) => {
                format!("Sequence[{}]", primitive.python_annotation())
            }
        }
    }

    pub fn return_annotation(&self) -> String {
        match self {
            Self::Bytes | Self::PrimitiveVec(PrimitiveType::U8) => "bytes".to_string(),
            Self::PrimitiveVec(primitive) => {
                format!("list[{}]", primitive.python_annotation())
            }
        }
    }

    pub fn primitive_element(&self) -> Option<PrimitiveType> {
        match self {
            Self::Bytes => None,
            Self::PrimitiveVec(primitive) => Some(*primitive),
        }
    }

    pub fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes)
    }

    pub fn is_byte_like(&self) -> bool {
        matches!(self, Self::Bytes | Self::PrimitiveVec(PrimitiveType::U8))
    }

    pub fn is_primitive_vector(&self) -> bool {
        matches!(self, Self::PrimitiveVec(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonType {
    Void,
    Primitive(PrimitiveType),
    String,
    Sequence(PythonSequenceType),
}

impl PythonType {
    pub fn parameter_annotation(&self) -> String {
        match self {
            Self::Void => "None".to_string(),
            Self::Primitive(primitive) => primitive.python_annotation().to_string(),
            Self::String => "str".to_string(),
            Self::Sequence(sequence) => sequence.parameter_annotation(),
        }
    }

    pub fn return_annotation(&self) -> String {
        match self {
            Self::Void => "None".to_string(),
            Self::Primitive(primitive) => primitive.python_annotation().to_string(),
            Self::String => "str".to_string(),
            Self::Sequence(sequence) => sequence.return_annotation(),
        }
    }

    pub fn scalar_primitive(&self) -> Option<PrimitiveType> {
        match self {
            Self::Void => None,
            Self::Primitive(primitive) => Some(*primitive),
            Self::String => None,
            Self::Sequence(_) => None,
        }
    }

    pub fn sequence_primitive(&self) -> Option<PrimitiveType> {
        match self {
            Self::Sequence(sequence) => sequence.primitive_element(),
            _ => None,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    pub fn is_bytes(&self) -> bool {
        matches!(self, Self::Sequence(PythonSequenceType::Bytes))
    }

    pub fn is_byte_like(&self) -> bool {
        matches!(self, Self::Sequence(sequence) if sequence.is_byte_like())
    }

    pub fn is_primitive_vector(&self) -> bool {
        matches!(self, Self::Sequence(PythonSequenceType::PrimitiveVec(_)))
    }

    pub fn is_owned_buffer(&self) -> bool {
        matches!(self, Self::String | Self::Sequence(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonParameter {
    pub name: String,
    pub type_ref: PythonType,
}

impl PythonParameter {
    pub fn is_string(&self) -> bool {
        self.type_ref.is_string()
    }

    pub fn is_buffer_input(&self) -> bool {
        matches!(self.type_ref, PythonType::Sequence(_))
    }

    pub fn parser_state_name(&self) -> String {
        format!("{}_input", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonFunction {
    pub python_name: String,
    pub ffi_symbol: String,
    pub parameters: Vec<PythonParameter>,
    pub return_type: PythonType,
}

impl PythonFunction {
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn takes_no_parameters(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn returns_void(&self) -> bool {
        self.return_type.is_void()
    }

    pub fn returns_string(&self) -> bool {
        self.return_type.is_string()
    }

    pub fn return_primitive(&self) -> Option<PrimitiveType> {
        self.return_type.scalar_primitive()
    }

    pub fn returns_bytes(&self) -> bool {
        self.return_type.is_byte_like()
    }

    pub fn returns_primitive_vector(&self) -> bool {
        self.return_type.is_primitive_vector()
    }

    pub fn return_vector_primitive(&self) -> Option<PrimitiveType> {
        self.return_type.sequence_primitive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonModule {
    pub module_name: String,
    pub package_name: String,
    pub package_version: Option<String>,
    pub library_name: String,
    pub free_buffer_symbol: String,
    pub functions: Vec<PythonFunction>,
}

impl PythonModule {
    pub fn module_name_literal(&self) -> String {
        format!("{:?}", self.module_name)
    }

    pub fn package_name_literal(&self) -> String {
        format!("{:?}", self.package_name)
    }

    pub fn package_version_literal(&self) -> String {
        self.package_version
            .as_ref()
            .map(|version| format!("{version:?}"))
            .unwrap_or_else(|| "None".to_string())
    }

    pub fn exported_names(&self) -> Vec<&str> {
        self.functions
            .iter()
            .map(|function| function.python_name.as_str())
            .collect()
    }

    pub fn used_primitive_types(&self) -> Vec<PrimitiveType> {
        self.functions
            .iter()
            .flat_map(|function| {
                function
                    .parameters
                    .iter()
                    .flat_map(|parameter| {
                        parameter
                            .type_ref
                            .scalar_primitive()
                            .into_iter()
                            .chain(parameter.type_ref.sequence_primitive())
                    })
                    .chain(
                        function
                            .return_type
                            .scalar_primitive()
                            .into_iter()
                            .chain(function.return_type.sequence_primitive()),
                    )
            })
            .fold(Vec::new(), |mut scalar_types, primitive| {
                if !scalar_types.contains(&primitive) {
                    scalar_types.push(primitive);
                }
                scalar_types
            })
    }

    pub fn uses_string_parameters(&self) -> bool {
        self.functions
            .iter()
            .any(|function| function.parameters.iter().any(PythonParameter::is_string))
    }

    pub fn uses_buffer_parameters(&self) -> bool {
        self.functions.iter().any(|function| {
            function
                .parameters
                .iter()
                .any(PythonParameter::is_buffer_input)
        })
    }

    pub fn uses_bytes_parameters(&self) -> bool {
        self.functions.iter().any(|function| {
            function
                .parameters
                .iter()
                .any(|parameter| parameter.type_ref.is_bytes())
        })
    }

    pub fn uses_string_returns(&self) -> bool {
        self.functions.iter().any(PythonFunction::returns_string)
    }

    pub fn uses_bytes_returns(&self) -> bool {
        self.functions.iter().any(PythonFunction::returns_bytes)
    }

    pub fn uses_primitive_vector_returns(&self) -> bool {
        self.functions
            .iter()
            .any(PythonFunction::returns_primitive_vector)
    }

    pub fn uses_owned_buffer_returns(&self) -> bool {
        self.functions
            .iter()
            .any(|function| function.return_type.is_owned_buffer())
    }

    pub fn uses_sequence_parameter_annotations(&self) -> bool {
        self.functions.iter().any(|function| {
            function
                .parameters
                .iter()
                .any(|parameter| parameter.type_ref.is_primitive_vector())
        })
    }

    pub fn used_primitive_vector_parameter_types(&self) -> Vec<PrimitiveType> {
        self.functions
            .iter()
            .flat_map(|function| {
                function
                    .parameters
                    .iter()
                    .filter_map(|parameter| parameter.type_ref.sequence_primitive())
            })
            .fold(Vec::new(), |mut primitive_types, primitive| {
                if !primitive_types.contains(&primitive) {
                    primitive_types.push(primitive);
                }
                primitive_types
            })
    }

    pub fn used_primitive_vector_return_types(&self) -> Vec<PrimitiveType> {
        self.functions
            .iter()
            .filter_map(PythonFunction::return_vector_primitive)
            .filter(|primitive| *primitive != PrimitiveType::U8)
            .fold(Vec::new(), |mut primitive_types, primitive| {
                if !primitive_types.contains(&primitive) {
                    primitive_types.push(primitive);
                }
                primitive_types
            })
    }
}
