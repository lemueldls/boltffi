use std::collections::BTreeSet;

use askama::Template;

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
            native_def_source: render_native_def(module),
        }
    }
}

#[derive(Template)]
#[template(path = "render_kmp/common_main.txt", escape = "none")]
struct CommonMainTemplate<'a> {
    module: &'a KmpModule,
}

#[derive(Template)]
#[template(path = "render_kmp/platform_main.txt", escape = "none")]
struct PlatformMainTemplate<'a> {
    module: &'a KmpModule,
    imports: &'a [String],
    alias_prefix: &'a str,
    has_data_classes: bool,
}

#[derive(Template)]
#[template(path = "render_kmp/native_def.txt", escape = "none")]
struct NativeDefTemplate<'a> {
    module: &'a KmpModule,
}

struct PlatformTemplateContext {
    imports: Vec<String>,
    alias_prefix: &'static str,
    has_data_classes: bool,
}

impl PlatformTemplateContext {
    fn for_platform(module: &KmpModule, platform: Platform) -> Self {
        Self {
            imports: collect_imports(module, platform),
            alias_prefix: alias_prefix(platform),
            has_data_classes: module_has_data_classes(module),
        }
    }
}

#[derive(Clone, Copy)]
enum Platform {
    Jvm,
    Native,
}

fn render_common_main(module: &KmpModule) -> String {
    CommonMainTemplate { module }.render().unwrap()
}

fn render_platform_main(module: &KmpModule, platform: Platform) -> String {
    let context = PlatformTemplateContext::for_platform(module, platform);
    PlatformMainTemplate {
        module,
        imports: &context.imports,
        alias_prefix: context.alias_prefix,
        has_data_classes: context.has_data_classes,
    }
    .render()
    .unwrap()
}

fn render_native_def(module: &KmpModule) -> String {
    NativeDefTemplate { module }.render().unwrap()
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
    render_actual_class_with_alias_prefix(class, alias_prefix(platform))
}

fn render_actual_class_with_alias_prefix(class: &KmpClass, alias_prefix: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "actual class {} private constructor(private val handle: Long) {{\n",
        class.name
    ));

    for constructor in &class.constructors {
        out.push_str(&format!(
            "    actual constructor({}) : this({}({}))\n",
            render_params(&constructor.params),
            symbol_alias(&constructor.ffi_symbol, alias_prefix),
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
        let alias = symbol_alias(&method.ffi_symbol, alias_prefix);
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
            let alias = symbol_alias(&factory.ffi_symbol, alias_prefix);
            out.push_str(&format!(
                "        actual {}fun {}({}): {} = {}({})\n",
                suspend_kw, factory.name, params, factory.return_type, alias, args
            ));
        }
        for method in class.methods.iter().filter(|method| method.is_static) {
            let suspend_kw = if method.is_async { "suspend " } else { "" };
            let params = render_params(&method.params);
            let args = render_call_args(method.params.iter().map(|param| param.name.as_str()));
            let alias = symbol_alias(&method.ffi_symbol, alias_prefix);
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

fn symbol_alias(symbol: &str, alias_prefix: &str) -> String {
    format!("{}{}", alias_prefix, symbol)
}

fn has_static_methods(class: &KmpClass) -> bool {
    class.methods.iter().any(|method| method.is_static)
}

fn alias_prefix(platform: Platform) -> &'static str {
    match platform {
        Platform::Jvm => "__jvm_",
        Platform::Native => "__native_",
    }
}

fn module_has_data_classes(module: &KmpModule) -> bool {
    !module.records.is_empty()
        || module
            .enums
            .iter()
            .any(|enumeration| matches!(enumeration.kind, KmpEnumKind::Data))
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
            symbol_alias(&function.ffi_symbol, alias_prefix(platform))
        ));
    }

    for class in &module.classes {
        for constructor in &class.constructors {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                constructor.ffi_symbol,
                symbol_alias(&constructor.ffi_symbol, alias_prefix(platform))
            ));
        }
        for factory in &class.factories {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                factory.ffi_symbol,
                symbol_alias(&factory.ffi_symbol, alias_prefix(platform))
            ));
        }
        for method in &class.methods {
            imports.insert(format!(
                "import {}.{} as {}",
                import_package,
                method.ffi_symbol,
                symbol_alias(&method.ffi_symbol, alias_prefix(platform))
            ));
        }
    }

    imports.into_iter().collect()
}
