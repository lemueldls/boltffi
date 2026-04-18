#[derive(Debug, Clone)]
pub struct KmpModule {
    pub package_name: String,
    pub module_name: String,
    pub library_name: String,
    pub jvm_binding_package: String,
    pub native_binding_package: String,
    pub records: Vec<KmpRecord>,
    pub enums: Vec<KmpEnum>,
    pub callbacks: Vec<KmpCallback>,
    pub classes: Vec<KmpClass>,
    pub functions: Vec<KmpFunction>,
}

#[derive(Debug, Clone)]
pub struct KmpOutputs {
    pub common_main_source: String,
    pub jvm_main_source: String,
    pub native_main_source: String,
}

#[derive(Debug, Clone)]
pub struct KmpParam {
    pub name: String,
    pub kotlin_type: String,
}

#[derive(Debug, Clone)]
pub struct KmpFunction {
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: String,
    pub is_async: bool,
    pub ffi_symbol: String,
}

#[derive(Debug, Clone)]
pub struct KmpRecord {
    pub name: String,
    pub fields: Vec<KmpParam>,
}

#[derive(Debug, Clone)]
pub enum KmpEnumKind {
    CStyle,
    Data,
}

#[derive(Debug, Clone)]
pub struct KmpEnumVariant {
    pub name: String,
    pub fields: Vec<KmpParam>,
}

#[derive(Debug, Clone)]
pub struct KmpEnum {
    pub name: String,
    pub kind: KmpEnumKind,
    pub variants: Vec<KmpEnumVariant>,
}

#[derive(Debug, Clone)]
pub struct KmpCallbackMethod {
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: String,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct KmpCallback {
    pub name: String,
    pub methods: Vec<KmpCallbackMethod>,
}

#[derive(Debug, Clone)]
pub struct KmpClassConstructor {
    pub params: Vec<KmpParam>,
    pub ffi_symbol: String,
}

#[derive(Debug, Clone)]
pub struct KmpClassFactory {
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: String,
    pub is_async: bool,
    pub ffi_symbol: String,
}

#[derive(Debug, Clone)]
pub struct KmpClassMethod {
    pub name: String,
    pub params: Vec<KmpParam>,
    pub return_type: String,
    pub is_async: bool,
    pub is_static: bool,
    pub ffi_symbol: String,
}

#[derive(Debug, Clone)]
pub struct KmpClass {
    pub name: String,
    pub constructors: Vec<KmpClassConstructor>,
    pub factories: Vec<KmpClassFactory>,
    pub methods: Vec<KmpClassMethod>,
}
