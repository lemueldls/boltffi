#[derive(Debug, Clone)]
pub struct KmpOptions {
    pub package_name: String,
    pub module_name: String,
    pub jvm_binding_package: String,
    pub native_binding_package: String,
    pub header_file_name: String,
    pub library_name: String,
}

#[derive(Debug, Clone)]
pub struct KmpOutputs {
    pub common_main_source: String,
    pub jvm_main_source: String,
    pub native_main_source: String,
    pub cinterop_def: String,
}

#[derive(Debug, Clone)]
pub struct KmpModule {
    pub records: Vec<KmpRecord>,
    pub enums: Vec<KmpEnum>,
    pub functions: Vec<KmpFunction>,
}

#[derive(Debug, Clone)]
pub struct KmpRecord {
    pub class_name: String,
    pub fields: Vec<KmpRecordField>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpRecordField {
    pub name: String,
    pub kotlin_type: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpEnum {
    pub class_name: String,
    pub is_c_style: bool,
    pub is_error: bool,
    pub value_type: Option<String>,
    pub variants: Vec<KmpEnumVariant>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpEnumVariant {
    pub name: String,
    pub tag: i128,
    pub fields: Vec<KmpEnumField>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpEnumField {
    pub name: String,
    pub kotlin_type: String,
}

#[derive(Debug, Clone)]
pub struct KmpFunction {
    pub public_name: String,
    pub ffi_symbol: String,
    pub params: Vec<KmpParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct KmpParam {
    pub name: String,
    pub kotlin_type: String,
}
