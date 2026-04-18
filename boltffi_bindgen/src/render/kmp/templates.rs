use super::plan::{KmpModule, KmpOutputs};

pub struct KmpTemplates;

impl KmpTemplates {
    pub fn render(module: &KmpModule) -> KmpOutputs {
        let common_main_source = format!(
            "package {}\n\nexpect object {} {{\n    // TODO: generated declarations\n}}\n",
            module.package_name, module.module_name
        );

        let jvm_main_source = format!(
            "package {}\n\nactual object {} {{\n    // TODO: generated JVM actual implementations\n}}\n",
            module.package_name, module.module_name
        );

        let native_main_source = format!(
            "package {}\n\nactual object {} {{\n    // TODO: generated native actual implementations\n}}\n\ninternal const val BOLTFFI_LIBRARY = \"{}\"\n",
            module.package_name, module.module_name, module.library_name
        );

        KmpOutputs {
            common_main_source,
            jvm_main_source,
            native_main_source,
        }
    }
}
