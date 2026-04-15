use askama::Template;

use super::plan::{KmpFunction, KmpModule, KmpOptions, KmpOutputs};

#[derive(Template)]
#[template(path = "render_kmp/common_main.txt", escape = "none")]
pub struct CommonMainTemplate<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub functions: &'a [KmpFunction],
}

#[derive(Template)]
#[template(path = "render_kmp/jvm_main.txt", escape = "none")]
pub struct JvmMainTemplate<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub jvm_binding_package: &'a str,
    pub functions: &'a [KmpFunction],
}

#[derive(Template)]
#[template(path = "render_kmp/native_main.txt", escape = "none")]
pub struct NativeMainTemplate<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub native_binding_package: &'a str,
    pub functions: &'a [KmpFunction],
}

#[derive(Template)]
#[template(path = "render_kmp/cinterop_def.txt", escape = "none")]
pub struct CInteropDefTemplate<'a> {
    pub header_file_name: &'a str,
    pub native_binding_package: &'a str,
    pub library_name: &'a str,
}

pub fn render_outputs(module: &KmpModule, options: &KmpOptions) -> KmpOutputs {
    KmpOutputs {
        common_main_source: CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            functions: &module.functions,
        }
        .render()
        .expect("commonMain template should render"),
        jvm_main_source: JvmMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            jvm_binding_package: &options.jvm_binding_package,
            functions: &module.functions,
        }
        .render()
        .expect("jvmMain template should render"),
        native_main_source: NativeMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            native_binding_package: &options.native_binding_package,
            functions: &module.functions,
        }
        .render()
        .expect("nativeMain template should render"),
        cinterop_def: CInteropDefTemplate {
            header_file_name: &options.header_file_name,
            native_binding_package: &options.native_binding_package,
            library_name: &options.library_name,
        }
        .render()
        .expect("cinterop def template should render"),
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    use crate::render::kmp::plan::KmpParam;

    fn test_options() -> KmpOptions {
        KmpOptions {
            package_name: "com.example.demo".to_string(),
            module_name: "DemoKmp".to_string(),
            jvm_binding_package: "com.example.demo.jvmffi".to_string(),
            native_binding_package: "com.example.demo.native".to_string(),
            header_file_name: "demo.h".to_string(),
            library_name: "demo".to_string(),
        }
    }

    fn test_module() -> KmpModule {
        KmpModule {
            functions: vec![
                KmpFunction {
                    public_name: "echoI32".to_string(),
                    ffi_symbol: "boltffi_echo_i32".to_string(),
                    params: vec![KmpParam {
                        name: "value".to_string(),
                        kotlin_type: "Int".to_string(),
                    }],
                    return_type: Some("Int".to_string()),
                    is_async: false,
                },
                KmpFunction {
                    public_name: "fetchName".to_string(),
                    ffi_symbol: "boltffi_fetch_name".to_string(),
                    params: vec![KmpParam {
                        name: "id".to_string(),
                        kotlin_type: "Long".to_string(),
                    }],
                    return_type: Some("String".to_string()),
                    is_async: true,
                },
            ],
        }
    }

    #[test]
    fn snapshot_common_main_template() {
        let module = test_module();
        let options = test_options();
        let rendered = CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            functions: &module.functions,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_jvm_main_template() {
        let module = test_module();
        let options = test_options();
        let rendered = JvmMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            jvm_binding_package: &options.jvm_binding_package,
            functions: &module.functions,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_native_main_template() {
        let module = test_module();
        let options = test_options();
        let rendered = NativeMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            native_binding_package: &options.native_binding_package,
            functions: &module.functions,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_cinterop_def_template() {
        let options = test_options();
        let rendered = CInteropDefTemplate {
            header_file_name: &options.header_file_name,
            native_binding_package: &options.native_binding_package,
            library_name: &options.library_name,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }
}
