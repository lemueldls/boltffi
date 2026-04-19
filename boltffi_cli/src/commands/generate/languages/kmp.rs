use boltffi_bindgen::render::c::CHeaderLowerer;
use boltffi_bindgen::render::kmp::{KmpEmitter, KmpLowerer};

use crate::cli::{CliError, Result};
use crate::commands::generate::generator::{GenerateRequest, LanguageGenerator, ScanPointerWidth};
use crate::config::Target;

pub struct KmpGenerator;

impl LanguageGenerator for KmpGenerator {
    const TARGET: Target = Target::Kmp;

    fn generate(request: &GenerateRequest<'_>) -> Result<()> {
        if !request.config().is_kmp_enabled() {
            return Err(CliError::CommandFailed {
                command: "targets.kmp.enabled = false".to_string(),
                status: None,
            });
        }

        let package_name = request.config().kmp_package();
        let package_path = package_name.replace('.', "/");
        let module_name = request.config().kmp_module_name();
        let library_name = request
            .config()
            .kmp_library_name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| request.config().crate_artifact_name());

        let output_root = request
            .output_override()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| request.config().kmp_output());

        let common_directory = output_root.join("commonMain/kotlin").join(&package_path);
        let jvm_directory = output_root.join("jvmMain/kotlin").join(&package_path);
        let native_directory = output_root.join("nativeMain/kotlin").join(package_path);
        let include_directory = output_root.join("include");

        request.ensure_output_directory(&common_directory)?;
        request.ensure_output_directory(&jvm_directory)?;
        request.ensure_output_directory(&native_directory)?;
        request.ensure_output_directory(&include_directory)?;

        let lowered_crate = request.lowered_crate(ScanPointerWidth::Flexible)?;
        let header_source =
            CHeaderLowerer::new(&lowered_crate.ffi_contract, &lowered_crate.abi_contract)
                .generate();
        let module = KmpLowerer::new(
            &lowered_crate.ffi_contract,
            &lowered_crate.abi_contract,
            package_name,
            module_name.clone(),
            library_name.clone(),
        )
        .lower();
        let outputs = KmpEmitter::emit(&module);

        let common_output_file_name = format!("{module_name}.kt");
        let jvm_output_file_name = format!("{module_name}Jvm.kt");
        let native_output_file_name = format!("{module_name}Native.kt");

        request.write_output(
            &common_directory.join(&common_output_file_name),
            outputs.common_main_source,
        )?;
        request.write_output(
            &jvm_directory.join(&jvm_output_file_name),
            outputs.jvm_main_source,
        )?;
        request.write_output(
            &native_directory.join(native_output_file_name),
            outputs.native_main_source,
        )?;
        request.write_output(
            &include_directory.join(format!("{library_name}.h")),
            header_source,
        )?;
        request.write_output(
            &include_directory.join(format!("{library_name}.def")),
            outputs.native_def_source,
        )
    }
}
