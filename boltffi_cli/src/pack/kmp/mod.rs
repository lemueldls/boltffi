use std::path::Path;

use boltffi_bindgen::render::jni::{JniEmitter, JniLowerer, JvmBindingStyle};

use crate::cli::{CliError, Result};
use crate::commands::generate::run_generate_kmp_with_output_from_source_dir;
use crate::commands::pack::PackKmpOptions;
use crate::config::{Config, Target};
use crate::pack::java::link::{build_jvm_native_library, compile_jni_library_with_output};
use crate::pack::java::outputs::{
    remove_stale_flat_jvm_outputs_if_current_host_unrequested,
    remove_stale_requested_jvm_shared_library_copies_after_success,
    remove_stale_structured_jvm_outputs,
};
use crate::pack::java::plan::{
    PreparedJavaPackaging, prepare_java_packaging, selected_jvm_package_source_directory,
};
use crate::reporter::Reporter;
use crate::target::JavaHostTarget;

pub(crate) fn check_kmp_packaging_prereqs(
    config: &Config,
    release: bool,
    cargo_args: &[String],
    experimental: bool,
) -> Result<()> {
    if config.should_process(Target::Kmp, experimental) {
        prepare_java_packaging(config, release, cargo_args)?;
    }

    Ok(())
}

pub(crate) fn pack_kmp(
    config: &Config,
    options: PackKmpOptions,
    reporter: &Reporter,
) -> Result<()> {
    if !config.should_process(Target::Kmp, options.experimental) {
        return Err(CliError::CommandFailed {
            command: "targets.kmp requires --experimental or [experimental] includes 'kmp'"
                .to_string(),
            status: None,
        });
    }

    ensure_kmp_no_build_supported(
        config,
        options.execution.no_build,
        options.experimental,
        "pack kmp",
    )?;

    reporter.section("🧩", "Packing KMP");

    let step = reporter.step("Validating JVM toolchains");
    let prepared = prepare_java_packaging(
        config,
        options.execution.release,
        &options.execution.cargo_args,
    )?;
    step.finish_success();

    if options.execution.regenerate {
        regenerate_kmp_sources_and_jni(config, &prepared)?;
    }

    let output_root = config.kmp_output();
    let jni_libs_directory = output_root.join("jniLibs");
    std::fs::create_dir_all(&jni_libs_directory).map_err(|source| {
        CliError::CreateDirectoryFailed {
            path: jni_libs_directory.clone(),
            source,
        }
    })?;

    let mut packaged_outputs = Vec::with_capacity(prepared.packaging_targets.len());
    for packaging_target in &prepared.packaging_targets {
        let host_target = packaging_target.cargo_context.host_target;
        let step = reporter.step(&format!(
            "Building Rust library for {}",
            host_target.canonical_name()
        ));
        let build_artifacts =
            build_jvm_native_library(packaging_target, options.execution.release, &step)?;
        step.finish_success();

        let step = reporter.step(&format!(
            "Compiling JNI library for {}",
            host_target.canonical_name()
        ));
        packaged_outputs.push(compile_jni_library_with_output(
            config,
            &output_root,
            packaging_target,
            &build_artifacts,
            &step,
        )?);
        step.finish_success();

        copy_host_jni_library_to_jnilibs(
            &output_root,
            &jni_libs_directory,
            host_target,
            &packaging_target.cargo_context.artifact_name,
        )?;
    }

    let artifact_name = selected_jvm_package_artifact_name(&prepared)?;
    remove_stale_requested_jvm_shared_library_copies_after_success(
        &output_root,
        &packaged_outputs,
        artifact_name,
    )?;
    remove_stale_structured_jvm_outputs(&output_root.join("native"), &prepared.java_host_targets)?;
    remove_stale_flat_jvm_outputs_if_current_host_unrequested(
        &output_root,
        JavaHostTarget::current(),
        &prepared.java_host_targets,
        artifact_name,
    )?;

    reporter.finish();
    Ok(())
}

fn regenerate_kmp_sources_and_jni(config: &Config, prepared: &PreparedJavaPackaging) -> Result<()> {
    let source_directory = selected_jvm_package_source_directory(&prepared.packaging_targets)?;
    let artifact_name = selected_jvm_package_artifact_name(prepared)?;

    run_generate_kmp_with_output_from_source_dir(
        config,
        Some(config.kmp_output()),
        &source_directory,
        artifact_name,
    )?;

    generate_kmp_jni_sources(config, &source_directory, artifact_name)
}

fn generate_kmp_jni_sources(
    config: &Config,
    source_directory: &Path,
    crate_name: &str,
) -> Result<()> {
    use boltffi_bindgen::{CHeaderLowerer, ir, scan_crate_with_pointer_width};

    let output_directory = config.kmp_output().join("jni");
    std::fs::create_dir_all(&output_directory).map_err(|source| {
        CliError::CreateDirectoryFailed {
            path: output_directory.clone(),
            source,
        }
    })?;

    let host_pointer_width_bits = match usize::BITS {
        32 => Some(32),
        64 => Some(64),
        _ => None,
    };

    let mut module =
        scan_crate_with_pointer_width(source_directory, crate_name, host_pointer_width_bits)
            .map_err(|error| CliError::CommandFailed {
                command: format!("scan_crate: {}", error),
                status: None,
            })?;

    let contract = ir::build_contract(&mut module);
    let abi = ir::Lowerer::new(&contract).to_abi_contract();

    let header_code = CHeaderLowerer::new(&contract, &abi).generate();
    let header_path = output_directory.join(format!("{crate_name}.h"));
    std::fs::write(&header_path, header_code).map_err(|source| CliError::WriteFailed {
        path: header_path,
        source,
    })?;

    let jni_module = JniLowerer::new(
        &contract,
        &abi,
        config.kmp_package(),
        config.kmp_module_name(),
    )
    .with_jvm_binding_style(JvmBindingStyle::Kotlin)
    .lower();
    let jni_source = JniEmitter::emit(&jni_module);
    let jni_path = output_directory.join("jni_glue.c");
    std::fs::write(&jni_path, jni_source).map_err(|source| CliError::WriteFailed {
        path: jni_path,
        source,
    })
}

fn copy_host_jni_library_to_jnilibs(
    output_root: &Path,
    jni_libs_directory: &Path,
    host_target: JavaHostTarget,
    artifact_name: &str,
) -> Result<()> {
    let file_name = host_target.jni_library_filename(artifact_name);
    let source = output_root
        .join("native")
        .join(host_target.canonical_name())
        .join(&file_name);

    let target_directory = jni_libs_directory.join(host_target.canonical_name());
    std::fs::create_dir_all(&target_directory).map_err(|source_error| {
        CliError::CreateDirectoryFailed {
            path: target_directory.clone(),
            source: source_error,
        }
    })?;

    let destination = target_directory.join(file_name);
    std::fs::copy(&source, &destination).map_err(|source_error| CliError::CopyFailed {
        from: source,
        to: destination,
        source: source_error,
    })?;

    Ok(())
}

fn ensure_kmp_no_build_supported(
    config: &Config,
    no_build: bool,
    experimental: bool,
    command_name: &str,
) -> Result<()> {
    if no_build && config.should_process(Target::Kmp, experimental) {
        return Err(CliError::CommandFailed {
            command: format!(
                "{command_name} --no-build is unsupported when KMP JNI packaging is enabled; rerun without --no-build"
            ),
            status: None,
        });
    }

    Ok(())
}

fn selected_jvm_package_artifact_name(prepared: &PreparedJavaPackaging) -> Result<&str> {
    prepared
        .packaging_targets
        .first()
        .map(|target| target.cargo_context.artifact_name.as_str())
        .ok_or_else(|| CliError::CommandFailed {
            command: "could not resolve selected Cargo package artifact name for KMP packaging"
                .to_string(),
            status: None,
        })
}
