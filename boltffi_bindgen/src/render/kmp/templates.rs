use std::collections::BTreeSet;

use super::plan::{
    KmpClass, KmpClassFactory, KmpClassMethod, KmpEnumKind, KmpFunction, KmpModule, KmpOutputs,
    KmpParam,
};

pub struct KmpTemplates;

impl KmpTemplates {
    pub fn render(module: &KmpModule) -> KmpOutputs {
        KmpOutputs {
            common_main_source: render_common_main(module),
            jvm_main_source: render_platform_main(module, Platform::Jvm),
            native_main_source: render_platform_main(module, Platform::Native),
        }
    }
}

#[derive(Clone, Copy)]
enum Platform {
    Jvm,
    Native,
}

fn render_common_main(module: &KmpModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("package {}\n\n", module.package_name));
    out.push_str("import kotlin.Result\n\n");

    for record in &module.records {
        let fields = render_params(&record.fields);
        out.push_str(&format!("expect class {}({}) {{\n", record.name, fields));
        for field in &record.fields {
            out.push_str(&format!("    val {}: {}\n", field.name, field.kotlin_type));
        }
        out.push_str("}\n\n");
    }

    for enumeration in &module.enums {
        match enumeration.kind {
            KmpEnumKind::CStyle => {
                let variants = enumeration
                    .variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!(
                    "enum class {} {{ {} }}\n\n",
                    enumeration.name, variants
                ));
            }
            KmpEnumKind::Data => {
                out.push_str(&format!("sealed interface {} {{\n", enumeration.name));
                for variant in &enumeration.variants {
                    if variant.fields.is_empty() {
                        out.push_str(&format!(
                            "    data object {} : {}\n",
                            variant.name, enumeration.name
                        ));
                    } else {
                        let fields = render_params(&variant.fields);
                        out.push_str(&format!(
                            "    expect class {}({}) : {} {{\n",
                            variant.name, fields, enumeration.name
                        ));
                        for field in &variant.fields {
                            out.push_str(&format!(
                                "        val {}: {}\n",
                                field.name, field.kotlin_type
                            ));
                        }
                        out.push_str("    }\n");
                    }
                }
                out.push_str("}\n\n");
            }
        }
    }

    for callback in &module.callbacks {
        out.push_str(&format!("interface {} {{\n", callback.name));
        for method in &callback.methods {
            let suspend_kw = if method.is_async { "suspend " } else { "" };
            let params = render_params(&method.params);
            out.push_str(&format!(
                "    {}fun {}({}): {}\n",
                suspend_kw, method.name, params, method.return_type
            ));
        }
        out.push_str("}\n\n");
    }

    for class in &module.classes {
        out.push_str(&format!("expect class {} {{\n", class.name));
        for constructor in &class.constructors {
            let params = render_params(&constructor.params);
            out.push_str(&format!("    constructor({})\n", params));
        }

        let instance_methods = class
            .methods
            .iter()
            .filter(|method| !method.is_static)
            .collect::<Vec<_>>();
        for method in &instance_methods {
            out.push_str(&format!("    {}\n", render_method_signature(method, 4)));
        }

        if !class.factories.is_empty() || class.methods.iter().any(|method| method.is_static) {
            out.push_str("\n    companion object {\n");
            for factory in &class.factories {
                out.push_str(&format!("        {}\n", render_factory_signature(factory)));
            }
            for method in class.methods.iter().filter(|method| method.is_static) {
                out.push_str(&format!("        {}\n", render_method_signature(method, 8)));
            }
            out.push_str("    }\n");
        }

        out.push_str("}\n\n");
    }

    out.push_str(&format!("expect object {} {{\n", module.module_name));
    for function in &module.functions {
        out.push_str(&format!("    {}\n", render_function_signature(function)));
    }
    out.push_str("}\n");

    out
}

fn render_platform_main(module: &KmpModule, platform: Platform) -> String {
    let mut out = String::new();
    out.push_str(&format!("package {}\n\n", module.package_name));

    let imports = collect_imports(module, platform);
    for import in imports {
        out.push_str(&import);
        out.push('\n');
    }

    if !module.classes.is_empty() || !module.functions.is_empty() {
        out.push('\n');
    }

    out.push_str(&render_platform_data_classes(module));
    if !module.records.is_empty()
        || module
            .enums
            .iter()
            .any(|enumeration| matches!(enumeration.kind, KmpEnumKind::Data))
    {
        out.push('\n');
    }

    for class in &module.classes {
        out.push_str(&render_actual_class(class, platform));
        out.push('\n');
    }

    out.push_str(&format!("actual object {} {{\n", module.module_name));
    for function in &module.functions {
        let suspend_kw = if function.is_async { "suspend " } else { "" };
        let params = render_params(&function.params);
        let args = render_call_args(function.params.iter().map(|param| param.name.as_str()));
        let alias = symbol_alias(&function.ffi_symbol, platform);
        if function.return_type == "Unit" {
            out.push_str(&format!(
                "    actual {}fun {}({}): {} = {}({})\n",
                suspend_kw, function.name, params, function.return_type, alias, args
            ));
        } else {
            out.push_str(&format!(
                "    actual {}fun {}({}): {} = {}({})\n",
                suspend_kw, function.name, params, function.return_type, alias, args
            ));
        }
    }
    out.push_str("}\n\n");
    out.push_str(&format!(
        "internal const val BOLTFFI_LIBRARY = \"{}\"\n",
        module.library_name
    ));

    out
}

fn render_platform_data_classes(module: &KmpModule) -> String {
    let mut out = String::new();

    for record in &module.records {
        out.push_str(&format!(
            "actual data class {} actual constructor({})\n\n",
            record.name,
            render_actual_property_params(&record.fields)
        ));
    }

    for enumeration in &module.enums {
        if !matches!(enumeration.kind, KmpEnumKind::Data) {
            continue;
        }

        for variant in &enumeration.variants {
            if variant.fields.is_empty() {
                continue;
            }

            out.push_str(&format!(
                "actual data class {} actual constructor({}) : {}\n\n",
                variant.name,
                render_actual_property_params(&variant.fields),
                enumeration.name
            ));
        }
    }

    out
}

fn render_actual_class(class: &KmpClass, platform: Platform) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "actual class {} private constructor(private val handle: Long) {{\n",
        class.name
    ));

    for constructor in &class.constructors {
        out.push_str(&format!(
            "    actual constructor({}) : this({}({}))\n",
            render_params(&constructor.params),
            symbol_alias(&constructor.ffi_symbol, platform),
            render_call_args(constructor.params.iter().map(|param| param.name.as_str()))
        ));
    }

    for method in class.methods.iter().filter(|method| !method.is_static) {
        let suspend_kw = if method.is_async { "suspend " } else { "" };
        let params = render_params(&method.params);
        let mut call_args = Vec::with_capacity(method.params.len() + 1);
        call_args.push("handle".to_string());
        call_args.extend(method.params.iter().map(|param| param.name.clone()));
        let args = render_call_args(call_args.iter().map(String::as_str));
        let alias = symbol_alias(&method.ffi_symbol, platform);
        out.push_str(&format!(
            "    actual {}fun {}({}): {} = {}({})\n",
            suspend_kw, method.name, params, method.return_type, alias, args
        ));
    }

    if !class.factories.is_empty() || class.methods.iter().any(|method| method.is_static) {
        out.push_str("\n    actual companion object {\n");
        for factory in &class.factories {
            let suspend_kw = if factory.is_async { "suspend " } else { "" };
            let params = render_params(&factory.params);
            let args = render_call_args(factory.params.iter().map(|param| param.name.as_str()));
            let alias = symbol_alias(&factory.ffi_symbol, platform);
            out.push_str(&format!(
                "        actual {}fun {}({}): {} = {}({})\n",
                suspend_kw, factory.name, params, factory.return_type, alias, args
            ));
        }
        for method in class.methods.iter().filter(|method| method.is_static) {
            let suspend_kw = if method.is_async { "suspend " } else { "" };
            let params = render_params(&method.params);
            let args = render_call_args(method.params.iter().map(|param| param.name.as_str()));
            let alias = symbol_alias(&method.ffi_symbol, platform);
            out.push_str(&format!(
                "        actual {}fun {}({}): {} = {}({})\n",
                suspend_kw, method.name, params, method.return_type, alias, args
            ));
        }
        out.push_str("    }\n");
    }

    out.push_str("}\n");
    out
}

fn render_function_signature(function: &KmpFunction) -> String {
    let suspend_kw = if function.is_async { "suspend " } else { "" };
    format!(
        "{}fun {}({}): {}",
        suspend_kw,
        function.name,
        render_params(&function.params),
        function.return_type
    )
}

fn render_factory_signature(factory: &KmpClassFactory) -> String {
    let suspend_kw = if factory.is_async { "suspend " } else { "" };
    format!(
        "{}fun {}({}): {}",
        suspend_kw,
        factory.name,
        render_params(&factory.params),
        factory.return_type
    )
}

fn render_method_signature(method: &KmpClassMethod, _indent: usize) -> String {
    let suspend_kw = if method.is_async { "suspend " } else { "" };
    format!(
        "{}fun {}({}): {}",
        suspend_kw,
        method.name,
        render_params(&method.params),
        method.return_type
    )
}

fn render_params(params: &[KmpParam]) -> String {
    params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.kotlin_type))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_actual_property_params(params: &[KmpParam]) -> String {
    params
        .iter()
        .map(|param| format!("actual val {}: {}", param.name, param.kotlin_type))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_call_args<'a>(args: impl Iterator<Item = &'a str>) -> String {
    args.collect::<Vec<_>>().join(", ")
}

fn symbol_alias(symbol: &str, platform: Platform) -> String {
    match platform {
        Platform::Jvm => format!("__jvm_{}", symbol),
        Platform::Native => format!("__native_{}", symbol),
    }
}

fn collect_imports(module: &KmpModule, platform: Platform) -> Vec<String> {
    let import_package = match platform {
        Platform::Jvm => &module.jvm_binding_package,
        Platform::Native => &module.native_binding_package,
    };
    let mut imports = BTreeSet::new();

    for function in &module.functions {
        imports.insert(format!(
            "import {}.{} as {}",
            import_package,
            function.ffi_symbol,
            symbol_alias(&function.ffi_symbol, platform)
        ));
    }

    for class in &module.classes {
        for constructor in &class.constructors {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                constructor.ffi_symbol,
                symbol_alias(&constructor.ffi_symbol, platform)
            ));
        }
        for factory in &class.factories {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                factory.ffi_symbol,
                symbol_alias(&factory.ffi_symbol, platform)
            ));
        }
        for method in &class.methods {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                method.ffi_symbol,
                symbol_alias(&method.ffi_symbol, platform)
            ));
        }
    }

    imports.into_iter().collect()
}
