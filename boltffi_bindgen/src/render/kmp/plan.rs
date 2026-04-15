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
    pub classes: Vec<KmpClass>,
    pub callbacks: Vec<KmpCallback>,
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
pub struct KmpClass {
    pub class_name: String,
    pub doc: Option<String>,
    pub constructors: Vec<KmpClassConstructor>,
    pub methods: Vec<KmpClassMethod>,
    pub streams: Vec<KmpClassStream>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KmpStreamMode {
    Async,
    Batch,
    Callback,
}

#[derive(Debug, Clone)]
pub struct KmpClassStream {
    pub name: String,
    pub item_type: String,
    pub mode: KmpStreamMode,
    pub subscribe_symbol: String,
    pub poll_symbol: String,
    pub pop_batch_symbol: String,
    pub wait_symbol: String,
    pub unsubscribe_symbol: String,
    pub free_symbol: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpClassConstructor {
    pub ffi_symbol: String,
    pub params: Vec<KmpParam>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpClassMethod {
    pub ffi_symbol: String,
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpCallback {
    pub interface_name: String,
    pub methods: Vec<KmpCallbackMethod>,
    pub is_closure: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KmpCallbackMethod {
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub doc: Option<String>,
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
