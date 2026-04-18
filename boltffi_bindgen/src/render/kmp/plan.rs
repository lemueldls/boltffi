#[derive(Debug, Clone)]
pub struct KmpModule {
    pub package_name: String,
    pub module_name: String,
    pub library_name: String,
}

#[derive(Debug, Clone)]
pub struct KmpOutputs {
    pub common_main_source: String,
    pub jvm_main_source: String,
    pub native_main_source: String,
}
