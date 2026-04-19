use std::collections::BTreeSet;

use askama::Template;

use super::plan::{
    KmpClass, KmpClassFactory, KmpClassMethod, KmpEnumKind, KmpEnumVariant, KmpFunction, KmpModule,
    KmpOutputs, KmpParam, KmpRecord,
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
#[template(path = "render_kmp/wire_runtime.txt", escape = "none")]
struct WireRuntimeTemplate;

#[derive(Template)]
#[template(path = "render_kmp/platform_main.txt", escape = "none")]
struct PlatformMainTemplate<'a> {
    module: &'a KmpModule,
    imports: &'a [String],
    alias_prefix: &'a str,
    has_data_classes: bool,
    is_jvm: bool,
}

#[derive(Template)]
#[template(path = "render_kmp/native_def.txt", escape = "none")]
struct NativeDefTemplate<'a> {
    module: &'a KmpModule,
}

#[derive(Template)]
#[template(path = "render_kmp/record_actual.txt", escape = "none")]
struct RecordActualTemplate<'a> {
    record: &'a KmpRecord,
}

#[derive(Template)]
#[template(path = "render_kmp/class_actual.txt", escape = "none")]
struct ClassActualTemplate<'a> {
    class: &'a KmpClass,
    alias_prefix: &'a str,
    has_static_methods: bool,
    is_jvm: bool,
}

#[derive(Template)]
#[template(path = "render_kmp/data_enum_variant_actual.txt", escape = "none")]
struct DataEnumVariantActualTemplate<'a> {
    variant: &'a KmpEnumVariant,
    enum_name: &'a str,
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

impl Platform {
    const fn is_jvm(self) -> bool {
        matches!(self, Self::Jvm)
    }
}

fn render_common_main(module: &KmpModule) -> String {
    CommonMainTemplate { module }.render().unwrap()
}

fn render_wire_runtime() -> String {
    WireRuntimeTemplate.render().unwrap()
}

fn render_platform_main(module: &KmpModule, platform: Platform) -> String {
    let context = PlatformTemplateContext::for_platform(module, platform);
    PlatformMainTemplate {
        module,
        imports: &context.imports,
        alias_prefix: context.alias_prefix,
        has_data_classes: context.has_data_classes,
        is_jvm: platform.is_jvm(),
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
        out.push_str(
            &RecordActualTemplate { record }
                .render()
                .expect("render kmp record actual"),
        );
        out.push('\n');
    }

    for enumeration in &module.enums {
        if !matches!(enumeration.kind, KmpEnumKind::Data) {
            continue;
        }

        for variant in &enumeration.variants {
            if variant.fields.is_empty() {
                continue;
            }

            out.push_str(
                &DataEnumVariantActualTemplate {
                    variant,
                    enum_name: &enumeration.name,
                }
                .render()
                .expect("render data enum variant actual"),
            );
            out.push('\n');
        }
    }

    out
}

fn render_actual_class(class: &KmpClass, platform: Platform) -> String {
    ClassActualTemplate {
        class,
        alias_prefix: alias_prefix(platform),
        has_static_methods: has_static_methods(class),
        is_jvm: platform.is_jvm(),
    }
    .render()
    .expect("render class actual")
}

fn render_actual_class_with_alias_prefix(
    class: &KmpClass,
    alias_prefix: &str,
    is_jvm: &bool,
) -> String {
    ClassActualTemplate {
        class,
        alias_prefix,
        has_static_methods: has_static_methods(class),
        is_jvm: *is_jvm,
    }
    .render()
    .expect("render class actual")
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
    if platform.is_jvm() {
        return Vec::new();
    }

    let import_package = &module.package_name;
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
