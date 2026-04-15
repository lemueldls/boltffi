use askama::Template;

use super::plan::{
    KmpCallbackMethod, KmpClassStream, KmpFunction, KmpModule, KmpOptions, KmpOutputs,
    KmpStreamMode,
};

#[derive(Template)]
#[template(path = "render_kmp/common_main.txt", escape = "none")]
pub struct CommonMainTemplate<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub record_sources: &'a [String],
    pub enum_sources: &'a [String],
    pub class_sources: &'a [String],
    pub callback_sources: &'a [String],
    pub uses_flow: bool,
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
    pub class_imports: &'a [String],
    pub class_sources: &'a [String],
    pub uses_flow: bool,
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

#[derive(Template)]
#[template(path = "kmp_class_common.txt", escape = "none")]
pub struct ClassCommonTemplate<'a> {
    pub class_name: &'a str,
    pub doc: Option<&'a str>,
    pub constructor_sources: &'a [String],
    pub method_sources: &'a [String],
    pub stream_sources: &'a [String],
}

#[derive(Template)]
#[template(path = "kmp_class_actual.txt", escape = "none")]
pub struct ClassActualTemplate<'a> {
    pub class_name: &'a str,
    pub doc: Option<&'a str>,
    pub constructor_sources: &'a [String],
    pub method_sources: &'a [String],
    pub stream_sources: &'a [String],
}

#[derive(Template)]
#[template(path = "kmp_callback_trait.txt", escape = "none")]
pub struct CallbackTraitTemplate<'a> {
    pub interface_name: &'a str,
    pub is_closure: bool,
    pub doc: Option<&'a str>,
    pub method_sources: &'a [String],
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
    let class_common_sources = module
        .classes
        .iter()
        .map(|class| {
            let constructor_sources = class
                .constructors
                .iter()
                .map(|ctor| {
                    render_kmp_constructor_signature(&ctor.params, ctor.doc.as_deref(), false, None)
                })
                .collect::<Vec<_>>();
            let method_sources = class
                .methods
                .iter()
                .map(|method| render_kmp_method_signature(method, false, None))
                .collect::<Vec<_>>();
            let stream_sources = class
                .streams
                .iter()
                .map(|stream| render_kmp_stream_signature(stream, false))
                .collect::<Vec<_>>();

            ClassCommonTemplate {
                class_name: &class.class_name,
                doc: class.doc.as_deref(),
                constructor_sources: &constructor_sources,
                method_sources: &method_sources,
                stream_sources: &stream_sources,
            }
            .render()
            .expect("KMP class common template should render")
        })
        .collect::<Vec<_>>();
    let callback_sources = module
        .callbacks
        .iter()
        .map(|callback| {
            let method_sources = callback
                .methods
                .iter()
                .map(render_kmp_callback_method_signature)
                .collect::<Vec<_>>();
            CallbackTraitTemplate {
                interface_name: &callback.interface_name,
                is_closure: callback.is_closure,
                doc: callback.doc.as_deref(),
                method_sources: &method_sources,
            }
            .render()
            .expect("KMP callback trait template should render")
        })
        .collect::<Vec<_>>();
    let class_imports = module
        .classes
        .iter()
        .flat_map(|class| {
            let ctor_imports = class.constructors.iter().map(|ctor| {
                format!(
                    "import {}.{} as __native_class_{}",
                    options.native_binding_package, ctor.ffi_symbol, ctor.ffi_symbol
                )
            });
            let method_imports = class.methods.iter().map(|method| {
                format!(
                    "import {}.{} as __native_class_{}",
                    options.native_binding_package, method.ffi_symbol, method.ffi_symbol
                )
            });
            let stream_imports = class.streams.iter().flat_map(|stream| {
                [stream.subscribe_symbol.as_str()]
                    .into_iter()
                    .map(|symbol| {
                        format!(
                            "import {}.{} as __native_class_{}",
                            options.native_binding_package, symbol, symbol
                        )
                    })
                    .collect::<Vec<_>>()
            });

            ctor_imports.chain(method_imports).chain(stream_imports)
        })
        .collect::<Vec<_>>();
    let class_actual_sources = module
        .classes
        .iter()
        .map(|class| {
            let constructor_sources = class
                .constructors
                .iter()
                .map(|ctor| {
                    render_kmp_constructor_signature(
                        &ctor.params,
                        ctor.doc.as_deref(),
                        true,
                        Some(&ctor.ffi_symbol),
                    )
                })
                .collect::<Vec<_>>();
            let method_sources = class
                .methods
                .iter()
                .map(|method| render_kmp_method_signature(method, true, Some(&method.ffi_symbol)))
                .collect::<Vec<_>>();
            let stream_sources = class
                .streams
                .iter()
                .map(|stream| render_kmp_stream_signature(stream, true))
                .collect::<Vec<_>>();

            ClassActualTemplate {
                class_name: &class.class_name,
                doc: class.doc.as_deref(),
                constructor_sources: &constructor_sources,
                method_sources: &method_sources,
                stream_sources: &stream_sources,
            }
            .render()
            .expect("KMP class actual template should render")
        })
        .collect::<Vec<_>>();
    let uses_flow = module.classes.iter().any(|class| !class.streams.is_empty());

    KmpOutputs {
        common_main_source: CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            record_sources: &record_sources,
            enum_sources: &enum_sources,
            class_sources: &class_common_sources,
            callback_sources: &callback_sources,
            uses_flow,
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
            class_imports: &class_imports,
            class_sources: &class_actual_sources,
            uses_flow,
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
            classes: vec![super::super::plan::KmpClass {
                class_name: "Widget".to_string(),
                doc: Some("A handle-backed widget.".to_string()),
                constructors: vec![super::super::plan::KmpClassConstructor {
                    ffi_symbol: "boltffi_widget_new".to_string(),
                    params: vec![KmpParam {
                        name: "name".to_string(),
                        kotlin_type: "String".to_string(),
                    }],
                    doc: Some("Creates a widget.".to_string()),
                }],
                methods: vec![super::super::plan::KmpClassMethod {
                    ffi_symbol: "boltffi_widget_rename".to_string(),
                    name: "rename".to_string(),
                    params: vec![KmpParam {
                        name: "name".to_string(),
                        kotlin_type: "String".to_string(),
                    }],
                    return_type: None,
                    is_async: false,
                    doc: Some("Renames the widget.".to_string()),
                }],
                streams: vec![super::super::plan::KmpClassStream {
                    name: "updates".to_string(),
                    item_type: "Int".to_string(),
                    mode: super::super::plan::KmpStreamMode::Async,
                    subscribe_symbol: "boltffi_widget_updates_subscribe".to_string(),
                    poll_symbol: "boltffi_widget_updates_poll".to_string(),
                    pop_batch_symbol: "boltffi_widget_updates_pop_batch".to_string(),
                    wait_symbol: "boltffi_widget_updates_wait".to_string(),
                    unsubscribe_symbol: "boltffi_widget_updates_unsubscribe".to_string(),
                    free_symbol: "boltffi_widget_updates_free".to_string(),
                    doc: Some("Watches widget updates.".to_string()),
                }],
            }],
            callbacks: vec![super::super::plan::KmpCallback {
                interface_name: "WidgetEvents".to_string(),
                methods: vec![super::super::plan::KmpCallbackMethod {
                    name: "onUpdate".to_string(),
                    params: vec![KmpParam {
                        name: "value".to_string(),
                        kotlin_type: "Int".to_string(),
                    }],
                    return_type: None,
                    is_async: false,
                    doc: Some("Called when widget updates.".to_string()),
                }],
                is_closure: false,
                doc: Some("Widget callback interface.".to_string()),
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
        let constructor_sources = vec![render_kmp_constructor_signature(
            &[KmpParam {
                name: "name".to_string(),
                kotlin_type: "String".to_string(),
            }],
            Some("Creates a widget."),
            false,
            None,
        )];
        let method_sources = vec![render_kmp_method_signature(
            &super::super::plan::KmpClassMethod {
                ffi_symbol: "boltffi_widget_rename".to_string(),
                name: "rename".to_string(),
                params: vec![KmpParam {
                    name: "name".to_string(),
                    kotlin_type: "String".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Renames the widget.".to_string()),
            },
            false,
            None,
        )];
        let stream_sources = vec![render_kmp_stream_signature(
            &super::super::plan::KmpClassStream {
                name: "updates".to_string(),
                item_type: "Int".to_string(),
                mode: super::super::plan::KmpStreamMode::Async,
                subscribe_symbol: "boltffi_widget_updates_subscribe".to_string(),
                poll_symbol: "boltffi_widget_updates_poll".to_string(),
                pop_batch_symbol: "boltffi_widget_updates_pop_batch".to_string(),
                wait_symbol: "boltffi_widget_updates_wait".to_string(),
                unsubscribe_symbol: "boltffi_widget_updates_unsubscribe".to_string(),
                free_symbol: "boltffi_widget_updates_free".to_string(),
                doc: Some("Watches widget updates.".to_string()),
            },
            false,
        )];
        let class_common_sources = vec![
            ClassCommonTemplate {
                class_name: "Widget",
                doc: Some("A handle-backed widget."),
                constructor_sources: &constructor_sources,
                method_sources: &method_sources,
                stream_sources: &stream_sources,
            }
            .render()
            .unwrap(),
        ];
        let callback_method_sources = vec![render_kmp_callback_method_signature(
            &super::super::plan::KmpCallbackMethod {
                name: "onUpdate".to_string(),
                params: vec![KmpParam {
                    name: "value".to_string(),
                    kotlin_type: "Int".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Called when widget updates.".to_string()),
            },
        )];
        let callback_sources = vec![
            CallbackTraitTemplate {
                interface_name: "WidgetEvents",
                is_closure: false,
                doc: Some("Widget callback interface."),
                method_sources: &callback_method_sources,
            }
            .render()
            .unwrap(),
        ];
        let rendered = CommonMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            record_sources: &record_sources,
            enum_sources: &enum_sources,
            class_sources: &class_common_sources,
            callback_sources: &callback_sources,
            uses_flow: true,
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
        let class_imports = vec![
            "import com.example.demo.native.boltffi_widget_new as __native_class_boltffi_widget_new".to_string(),
            "import com.example.demo.native.boltffi_widget_rename as __native_class_boltffi_widget_rename".to_string(),
            "import com.example.demo.native.boltffi_widget_updates_subscribe as __native_class_boltffi_widget_updates_subscribe".to_string(),
        ];
        let constructor_sources = vec![render_kmp_constructor_signature(
            &[KmpParam {
                name: "name".to_string(),
                kotlin_type: "String".to_string(),
            }],
            Some("Creates a widget."),
            true,
            Some("boltffi_widget_new"),
        )];
        let method_sources = vec![render_kmp_method_signature(
            &super::super::plan::KmpClassMethod {
                ffi_symbol: "boltffi_widget_rename".to_string(),
                name: "rename".to_string(),
                params: vec![KmpParam {
                    name: "name".to_string(),
                    kotlin_type: "String".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Renames the widget.".to_string()),
            },
            true,
            Some("boltffi_widget_rename"),
        )];
        let stream_sources = vec![render_kmp_stream_signature(
            &super::super::plan::KmpClassStream {
                name: "updates".to_string(),
                item_type: "Int".to_string(),
                mode: super::super::plan::KmpStreamMode::Async,
                subscribe_symbol: "boltffi_widget_updates_subscribe".to_string(),
                poll_symbol: "boltffi_widget_updates_poll".to_string(),
                pop_batch_symbol: "boltffi_widget_updates_pop_batch".to_string(),
                wait_symbol: "boltffi_widget_updates_wait".to_string(),
                unsubscribe_symbol: "boltffi_widget_updates_unsubscribe".to_string(),
                free_symbol: "boltffi_widget_updates_free".to_string(),
                doc: Some("Watches widget updates.".to_string()),
            },
            true,
        )];
        let class_actual_sources = vec![
            ClassActualTemplate {
                class_name: "Widget",
                doc: Some("A handle-backed widget."),
                constructor_sources: &constructor_sources,
                method_sources: &method_sources,
                stream_sources: &stream_sources,
            }
            .render()
            .unwrap(),
        ];
        let rendered = NativeMainTemplate {
            package_name: &options.package_name,
            module_name: &options.module_name,
            native_binding_package: &options.native_binding_package,
            class_imports: &class_imports,
            class_sources: &class_actual_sources,
            uses_flow: true,
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
    fn snapshot_class_common_template() {
        let constructor_sources = vec![render_kmp_constructor_signature(
            &[KmpParam {
                name: "name".to_string(),
                kotlin_type: "String".to_string(),
            }],
            Some("Creates a widget."),
            false,
            None,
        )];
        let method_sources = vec![render_kmp_method_signature(
            &super::super::plan::KmpClassMethod {
                ffi_symbol: "boltffi_widget_rename".to_string(),
                name: "rename".to_string(),
                params: vec![KmpParam {
                    name: "name".to_string(),
                    kotlin_type: "String".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Renames the widget.".to_string()),
            },
            false,
            None,
        )];
        let stream_sources = vec![render_kmp_stream_signature(
            &super::super::plan::KmpClassStream {
                name: "updates".to_string(),
                item_type: "Int".to_string(),
                mode: super::super::plan::KmpStreamMode::Async,
                subscribe_symbol: "boltffi_widget_updates_subscribe".to_string(),
                poll_symbol: "boltffi_widget_updates_poll".to_string(),
                pop_batch_symbol: "boltffi_widget_updates_pop_batch".to_string(),
                wait_symbol: "boltffi_widget_updates_wait".to_string(),
                unsubscribe_symbol: "boltffi_widget_updates_unsubscribe".to_string(),
                free_symbol: "boltffi_widget_updates_free".to_string(),
                doc: Some("Watches widget updates.".to_string()),
            },
            false,
        )];
        let rendered = ClassCommonTemplate {
            class_name: "Widget",
            doc: Some("A handle-backed widget."),
            constructor_sources: &constructor_sources,
            method_sources: &method_sources,
            stream_sources: &stream_sources,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_class_actual_template() {
        let constructor_sources = vec![render_kmp_constructor_signature(
            &[KmpParam {
                name: "name".to_string(),
                kotlin_type: "String".to_string(),
            }],
            Some("Creates a widget."),
            true,
            Some("boltffi_widget_new"),
        )];
        let method_sources = vec![render_kmp_method_signature(
            &super::super::plan::KmpClassMethod {
                ffi_symbol: "boltffi_widget_rename".to_string(),
                name: "rename".to_string(),
                params: vec![KmpParam {
                    name: "name".to_string(),
                    kotlin_type: "String".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Renames the widget.".to_string()),
            },
            true,
            Some("boltffi_widget_rename"),
        )];
        let stream_sources = vec![render_kmp_stream_signature(
            &super::super::plan::KmpClassStream {
                name: "updates".to_string(),
                item_type: "Int".to_string(),
                mode: super::super::plan::KmpStreamMode::Async,
                subscribe_symbol: "boltffi_widget_updates_subscribe".to_string(),
                poll_symbol: "boltffi_widget_updates_poll".to_string(),
                pop_batch_symbol: "boltffi_widget_updates_pop_batch".to_string(),
                wait_symbol: "boltffi_widget_updates_wait".to_string(),
                unsubscribe_symbol: "boltffi_widget_updates_unsubscribe".to_string(),
                free_symbol: "boltffi_widget_updates_free".to_string(),
                doc: Some("Watches widget updates.".to_string()),
            },
            true,
        )];
        let rendered = ClassActualTemplate {
            class_name: "Widget",
            doc: Some("A handle-backed widget."),
            constructor_sources: &constructor_sources,
            method_sources: &method_sources,
            stream_sources: &stream_sources,
        }
        .render()
        .unwrap();

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn snapshot_callback_trait_template() {
        let method_sources = vec![render_kmp_callback_method_signature(
            &super::super::plan::KmpCallbackMethod {
                name: "onUpdate".to_string(),
                params: vec![KmpParam {
                    name: "value".to_string(),
                    kotlin_type: "Int".to_string(),
                }],
                return_type: None,
                is_async: false,
                doc: Some("Called when widget updates.".to_string()),
            },
        )];
        let rendered = CallbackTraitTemplate {
            interface_name: "WidgetEvents",
            is_closure: false,
            doc: Some("Widget callback interface."),
            method_sources: &method_sources,
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

fn render_kmp_constructor_signature(
    params: &[super::plan::KmpParam],
    doc: Option<&str>,
    actual: bool,
    ffi_symbol: Option<&str>,
) -> String {
    let signature = params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.kotlin_type))
        .collect::<Vec<_>>()
        .join(", ");
    let arg_list = params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let mut rendered = String::new();
    if let Some(doc) = doc {
        rendered.push_str(&render_doc_block(doc, "    "));
    }
    if actual {
        let ffi_symbol = ffi_symbol.expect("actual constructor requires ffi symbol");
        rendered.push_str(&format!(
            "    constructor({signature}) : this(__native_class_{ffi_symbol}({arg_list}))\n"
        ));
    } else {
        rendered.push_str(&format!("    constructor({signature})\n"));
    }
    rendered
}

fn render_kmp_method_signature(
    method: &super::plan::KmpClassMethod,
    actual: bool,
    ffi_symbol: Option<&str>,
) -> String {
    let signature = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.kotlin_type))
        .collect::<Vec<_>>()
        .join(", ");
    let arg_list = if method.params.is_empty() {
        "handle".to_string()
    } else {
        let param_args = method
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("handle, {param_args}")
    };
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ret| format!(": {ret}"))
        .unwrap_or_default();
    let suspend_prefix = if method.is_async { "suspend " } else { "" };
    let mut rendered = String::new();
    if let Some(doc) = method.doc.as_deref() {
        rendered.push_str(&render_doc_block(doc, "    "));
    }
    if actual {
        let ffi_symbol = ffi_symbol.expect("actual method requires ffi symbol");
        rendered.push_str(&format!(
            "    {suspend_prefix}fun {}({signature}){return_suffix} = __native_class_{ffi_symbol}({arg_list})\n",
            method.name
        ));
    } else {
        rendered.push_str(&format!(
            "    {suspend_prefix}fun {}({signature}){return_suffix}\n",
            method.name
        ));
    }
    rendered
}

fn render_kmp_stream_signature(stream: &KmpClassStream, actual: bool) -> String {
    let mut rendered = String::new();
    if let Some(doc) = stream.doc.as_deref() {
        rendered.push_str(&render_doc_block(doc, "    "));
    }
    if actual {
        let mode = match stream.mode {
            KmpStreamMode::Async => "async",
            KmpStreamMode::Batch => "batch",
            KmpStreamMode::Callback => "callback",
        };
        rendered.push_str(&format!(
            "    fun {}(): Flow<{}> = flow {{\n        val subscription = __native_class_{}(handle)\n        if (subscription == 0L) return@flow\n        error(\"KMP {} stream bridge not yet implemented\")\n    }}\n",
            stream.name, stream.item_type, stream.subscribe_symbol, mode
        ));
    } else {
        rendered.push_str(&format!(
            "    fun {}(): Flow<{}>\n",
            stream.name, stream.item_type
        ));
    }
    rendered
}

fn render_kmp_callback_method_signature(method: &KmpCallbackMethod) -> String {
    let signature = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.kotlin_type))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ret| format!(": {ret}"))
        .unwrap_or_default();
    let suspend_prefix = if method.is_async { "suspend " } else { "" };
    let mut rendered = String::new();
    if let Some(doc) = method.doc.as_deref() {
        rendered.push_str(&render_doc_block(doc, "    "));
    }
    rendered.push_str(&format!(
        "    {suspend_prefix}fun {}({signature}){return_suffix}\n",
        method.name
    ));
    rendered
}

fn render_doc_block(doc: &str, indent: &str) -> String {
    let mut rendered = format!("{indent}/**\n");
    for line in doc.lines() {
        if line.is_empty() {
            rendered.push_str(&format!("{indent} *\n"));
        } else {
            rendered.push_str(&format!("{indent} * {line}\n"));
        }
    }
    rendered.push_str(&format!("{indent} */\n"));
    rendered
}
