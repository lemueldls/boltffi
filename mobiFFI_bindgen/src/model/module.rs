use serde::{Deserialize, Serialize};

use super::class::Class;
use super::enumeration::Enumeration;
use super::function::Function;
use super::record::Record;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub classes: Vec<Class>,
    pub records: Vec<Record>,
    pub enums: Vec<Enumeration>,
    pub functions: Vec<Function>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            classes: Vec::new(),
            records: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn ffi_prefix(&self) -> String {
        format!("mffi_{}", self.name.to_lowercase())
    }

    pub fn with_class(mut self, class: Class) -> Self {
        self.classes.push(class);
        self
    }

    pub fn with_record(mut self, record: Record) -> Self {
        self.records.push(record);
        self
    }

    pub fn with_enum(mut self, enumeration: Enumeration) -> Self {
        self.enums.push(enumeration);
        self
    }

    pub fn with_function(mut self, function: Function) -> Self {
        self.functions.push(function);
        self
    }

    pub fn find_class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|class| class.name == name)
    }

    pub fn find_record(&self, name: &str) -> Option<&Record> {
        self.records.iter().find(|record| record.name == name)
    }

    pub fn find_enum(&self, name: &str) -> Option<&Enumeration> {
        self.enums.iter().find(|enumeration| enumeration.name == name)
    }
}
