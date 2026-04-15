use askama::Template;

use super::plan::{KmpFunction, KmpModule, KmpOptions, KmpOutputs};

#[derive(Template)]
#[template(path = "render_kmp/common_main.txt", escape = "none")]
pub struct CommonMainTemplate<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub record_sources: &'a [String],
    pub enum_sources: &'a [String],
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

#[derive(Template)]
#[template(path = "kmp_record.txt", escape = "none")]
pub struct RecordTemplate<'a> {
    pub class_name: &'a str,
    pub fields: &'a [KmpRecordFieldView<'a>],
    pub doc: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "kmp_enum.txt", escape = "none")]
pub struct EnumTemplate<'a> {
    pub class_name: &'a str,
    pub is_c_style: bool,
    pub is_error: bool,
    pub value_type: Option<&'a str>,
    pub variants: &'a [KmpEnumVariantView],
    pub doc: Option<&'a str>,
}

pub struct KmpEnumVariantView {
    pub name: String,
    pub tag: i128,
    pub fields: Vec<KmpEnumFieldView>,
    pub doc: Option<String>,
}

pub struct KmpEnumFieldView {
    pub name: String,
    pub kotlin_type: String,
}

pub struct KmpRecordFieldView<'a> {
    pub name: &'a str,
    pub kotlin_type: &'a str,
    pub default_value: Option<&'a str>,
}

pub fn render_outputs(module: &KmpModule, options: &KmpOptions) -> KmpOutputs {
    let record_sources = module
        .records
        .iter()
        .map(|record| {
            let fields = record
                .fields
                .iter()
                .map(|field| KmpRecordFieldView {
                    name: &field.name,
                    kotlin_type: &field.kotlin_type,
                    default_value: field.default_value.as_deref(),
                })
                .collect::<Vec<_>>();

            RecordTemplate {
                class_name: &record.class_name,
                fields: &fields,
                doc: record.doc.as_deref(),
            }
            .render()
            .expect("KMP record template should render")
        })
        .collect::<Vec<_>>();
    let enum_sources = module
        .enums
        .iter()
        .map(|enumeration| {
            let variants = enumeration
                .variants
                .iter()
                .map(|variant| {
                    let fields = variant
                        .fields
                        .iter()
                        .map(|field| KmpEnumFieldView {
                            name: field.name.clone(),
                            kotlin_type: field.kotlin_type.clone(),
                        })
                        .collect::<Vec<_>>();
                    KmpEnumVariantView {
                        name: variant.name.clone(),
                        tag: variant.tag,
                        fields,
                        doc: variant.doc.clone(),
                    }
                })
                .collect::<Vec<_>>();

            EnumTemplate {
                class_name: &enumeration.class_name,
                is_c_style: enumeration.is_c_style,
                is_error: enumeration.is_error,
                value_type: enumeration.value_type.as_deref(),
                variants: &variants,
                doc: enumeration.doc.as_deref(),
            }
            .render()
            .expect("KMP enum template should render")
        })
        .collect::<Vec<_>>();

    KmpOutputs {
        common_main_source: CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            record_sources: &record_sources,
            enum_sources: &enum_sources,
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
            records: vec![super::super::plan::KmpRecord {
                class_name: "Location".to_string(),
                fields: vec![super::super::plan::KmpRecordField {
                    name: "id".to_string(),
                    kotlin_type: "Long".to_string(),
                    default_value: None,
                }],
                doc: Some("A physical location.".to_string()),
            }],
            enums: vec![super::super::plan::KmpEnum {
                class_name: "Result".to_string(),
                is_c_style: false,
                is_error: false,
                value_type: None,
                variants: vec![super::super::plan::KmpEnumVariant {
                    name: "Success".to_string(),
                    tag: 0,
                    fields: vec![],
                    doc: Some("Operation succeeded.".to_string()),
                }],
                doc: Some("The result of an operation.".to_string()),
            }],
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
        let record_fields = vec![KmpRecordFieldView {
            name: "id",
            kotlin_type: "Long",
            default_value: None,
        }];
        let record_sources = vec![
            RecordTemplate {
                class_name: "Location",
                fields: &record_fields,
                doc: Some("A physical location."),
            }
            .render()
            .unwrap(),
        ];
        let enum_variants = vec![KmpEnumVariantView {
            name: "Success".to_string(),
            tag: 0,
            fields: vec![],
            doc: Some("Operation succeeded.".to_string()),
        }];
        let enum_sources = vec![
            EnumTemplate {
                class_name: "Result",
                is_c_style: false,
                is_error: false,
                value_type: None,
                variants: &enum_variants,
                doc: Some("The result of an operation."),
            }
            .render()
            .unwrap(),
        ];
        let rendered = CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            record_sources: &record_sources,
            enum_sources: &enum_sources,
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
    fn snapshot_record_template() {
        let fields = vec![
            KmpRecordFieldView {
                name: "id",
                kotlin_type: "Long",
                default_value: None,
            },
            KmpRecordFieldView {
                name: "name",
                kotlin_type: "String",
                default_value: Some("\"demo\""),
            },
        ];

        let rendered = RecordTemplate {
            class_name: "Location",
            fields: &fields,
            doc: Some("A physical location."),
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_enum_template() {
        let variants = vec![
            KmpEnumVariantView {
                name: "Success".to_string(),
                tag: 0,
                fields: vec![],
                doc: Some("Operation succeeded.".to_string()),
            },
            KmpEnumVariantView {
                name: "Error".to_string(),
                tag: 1,
                fields: vec![KmpEnumFieldView {
                    name: "message".to_string(),
                    kotlin_type: "String".to_string(),
                }],
                doc: Some("Operation failed.".to_string()),
            },
        ];

        let rendered = EnumTemplate {
            class_name: "Result",
            is_c_style: false,
            is_error: false,
            value_type: None,
            variants: &variants,
            doc: Some("The result of an operation."),
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
