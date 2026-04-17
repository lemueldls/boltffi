use boltffi_bindgen::{
    CHeaderLowerer, FactoryStyle, KotlinApiStyle, KotlinOptions,
    render::{
        jni::{JniEmitter, JniLowerer, JvmBindingStyle},
        kmp::{KmpEmitter, KmpOptions},
        kotlin::{KotlinEmitter, KotlinLowerer},
    },
};

use crate::{
    cli::{CliError, Result},
    commands::generate::generator::{GenerateRequest, LanguageGenerator, ScanPointerWidth},
    config::Target,
};

pub struct KmpGenerator;

impl KmpGenerator {
    #[cfg(test)]
    pub fn generate_from_source_directory(
        config: &crate::config::Config,
        output_override: Option<std::path::PathBuf>,
        source_directory: &std::path::Path,
        crate_name: &str,
    ) -> Result<()> {
        let request = GenerateRequest::new(
            config,
            output_override,
            crate::commands::generate::generator::SourceCrate::new(source_directory, crate_name),
        );

        Self::generate(&request)
    }
}

impl LanguageGenerator for KmpGenerator {
    const TARGET: Target = Target::Kmp;

    fn generate(request: &GenerateRequest<'_>) -> Result<()> {
        if !request.config().is_kmp_enabled() {
            return Err(CliError::CommandFailed {
                command: "targets.kmp.enabled = false".to_string(),
                status: None,
            });
        }

        let output_directory = request
            .output_override()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| request.config().kmp_output());

        let package_name = request.config().kmp_package();
        let module_name = request.config().kmp_module_name();
        let library_name = request
            .config()
            .kmp_library_name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| request.config().crate_artifact_name());

        let package_path = package_name.replace('.', "/");
        let common_directory = output_directory
            .join("commonMain")
            .join("kotlin")
            .join(&package_path);
        let jvm_directory = output_directory
            .join("jvmMain")
            .join("kotlin")
            .join(&package_path);
        let native_directory = output_directory
            .join("nativeMain")
            .join("kotlin")
            .join(&package_path);
        let include_directory = output_directory.join("include");

        let jvm_binding_package = package_name.clone();
        let jvm_binding_module_name = format!("{}JvmFfi", module_name);
        let jvm_binding_path = jvm_binding_package.replace('.', "/");
        let jvm_binding_directory = output_directory
            .join("jvmMain")
            .join("kotlin")
            .join(&jvm_binding_path);
        let jni_directory = output_directory.join("jvmMain").join("jni");

        request.ensure_output_directory(&common_directory)?;
        request.ensure_output_directory(&jvm_directory)?;
        request.ensure_output_directory(&native_directory)?;
        request.ensure_output_directory(&include_directory)?;
        request.ensure_output_directory(&jvm_binding_directory)?;
        request.ensure_output_directory(&jni_directory)?;

        let lowered_crate = request.lowered_crate(ScanPointerWidth::Flexible)?;

        let header_path = include_directory.join(format!("{}.h", request.config().library_name()));
        let header_source =
            CHeaderLowerer::new(&lowered_crate.ffi_contract, &lowered_crate.abi_contract)
                .generate();
        request.write_output(&header_path, header_source)?;

        let kmp_outputs = KmpEmitter::emit(
            &lowered_crate.ffi_contract,
            &lowered_crate.abi_contract,
            &KmpOptions {
                package_name: package_name.clone(),
                module_name: module_name.clone(),
                jvm_binding_package: jvm_binding_package.clone(),
                native_binding_package: format!("{}.native", package_name),
                header_file_name: format!("{}.h", request.config().library_name()),
                library_name: library_name.clone(),
            },
        );

        let common_path = common_directory.join(format!("{module_name}.kt"));
        request.write_output(&common_path, &kmp_outputs.common_main_source)?;

        let jvm_path = jvm_directory.join(format!("{module_name}.kt"));
        request.write_output(&jvm_path, &kmp_outputs.jvm_main_source)?;

        let native_path = native_directory.join(format!("{module_name}.kt"));
        request.write_output(&native_path, &kmp_outputs.native_main_source)?;

        let def_path = include_directory.join(format!("{}.def", request.config().library_name()));
        request.write_output(&def_path, &kmp_outputs.cinterop_def)?;

        let jvm_binding_module = KotlinLowerer::new(
            &lowered_crate.ffi_contract,
            &lowered_crate.abi_contract,
            jvm_binding_package.clone(),
            jvm_binding_module_name.clone(),
            KotlinOptions {
                factory_style: FactoryStyle::Constructors,
                api_style: KotlinApiStyle::TopLevel,
                module_object_name: Some(jvm_binding_module_name.clone()),
                library_name: Some(boltffi_bindgen::library_name(library_name.as_str())),
                desktop_loader: true,
            },
        )
        .lower();
        let jvm_binding_source = KotlinEmitter::emit(&jvm_binding_module);
        let jvm_binding_path = jvm_binding_directory.join(format!("{jvm_binding_module_name}.kt"));
        request.write_output(&jvm_binding_path, jvm_binding_source)?;

        let jni_module = JniLowerer::new(
            &lowered_crate.ffi_contract,
            &lowered_crate.abi_contract,
            jvm_binding_package,
            jvm_binding_module_name,
        )
        .with_jvm_binding_style(JvmBindingStyle::Kotlin)
        .lower();
        let jni_source = JniEmitter::emit(&jni_module);
        let jni_path = jni_directory.join("jni_glue.c");

        request.write_output(&jni_path, jni_source)
    }
}
