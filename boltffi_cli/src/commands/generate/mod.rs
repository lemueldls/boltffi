mod generator;
mod header;
mod languages;

use std::path::{Path, PathBuf};

use generator::{GenerateRequest, run_generator};
use header::HeaderGenerator;
use languages::{
    DartGenerator, JavaGenerator, KmpGenerator, KotlinGenerator, PythonGenerator, SwiftGenerator,
    TypeScriptGenerator,
};

use crate::cli::Result;
use crate::config::{Config, Target};

pub enum GenerateTarget {
    Swift,
    Kotlin,
    Kmp,
    Java,
    Header,
    Typescript,
    Dart,
    Python,
    All,
}

pub struct GenerateOptions {
    pub target: GenerateTarget,
    pub output: Option<PathBuf>,
    pub experimental: bool,
}

pub fn run_generate_with_output(config: &Config, options: GenerateOptions) -> Result<()> {
    let request = GenerateRequest::for_current_crate(config, options.output);

    match options.target {
        GenerateTarget::Swift => run_generator::<SwiftGenerator>(&request, options.experimental),
        GenerateTarget::Kotlin => run_generator::<KotlinGenerator>(&request, options.experimental),
        GenerateTarget::Kmp => run_generator::<KmpGenerator>(&request, options.experimental),
        GenerateTarget::Java => run_generator::<JavaGenerator>(&request, options.experimental),
        GenerateTarget::Header => run_generator::<HeaderGenerator>(&request, options.experimental),
        GenerateTarget::Typescript => {
            run_generator::<TypeScriptGenerator>(&request, options.experimental)
        }
        GenerateTarget::Dart => run_generator::<DartGenerator>(&request, options.experimental),
        GenerateTarget::Python => run_generator::<PythonGenerator>(&request, options.experimental),
        GenerateTarget::All => {
            if config.should_process(Target::Swift, options.experimental) {
                run_generator::<SwiftGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Kotlin, options.experimental) {
                run_generator::<KotlinGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Kmp, options.experimental) {
                run_generator::<KmpGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Java, options.experimental) {
                run_generator::<JavaGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Header, options.experimental) {
                run_generator::<HeaderGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::TypeScript, options.experimental) {
                run_generator::<TypeScriptGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Dart, options.experimental) {
                run_generator::<DartGenerator>(&request, options.experimental)?;
            }

            if config.should_process(Target::Python, options.experimental) {
                run_generator::<PythonGenerator>(&request, options.experimental)?;
            }

            Ok(())
        }
    }
}

pub fn run_generate_java_with_output_from_source_dir(
    config: &Config,
    output: Option<PathBuf>,
    source_directory: &Path,
    crate_name: &str,
) -> Result<()> {
    JavaGenerator::generate_from_source_directory(config, output, source_directory, crate_name)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::languages::KmpGenerator;
    use super::languages::PythonGenerator;
    use crate::config::Config;

    fn parse_config(input: &str) -> Config {
        let parsed: Config = toml::from_str(input).expect("toml parse failed");
        parsed.validate().expect("config validation failed");
        parsed
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{unique_suffix}"))
    }

    fn demo_source_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/demo")
    }

    #[test]
    fn python_generate_writes_scalar_package_files() {
        let output_directory = unique_temp_dir("boltffi-python-generate-test");
        let config = parse_config(
            r#"
[package]
name = "demo"
version = "0.1.0"

[targets.python]
enabled = true
"#,
        );

        PythonGenerator::generate_from_source_directory(
            &config,
            Some(output_directory.clone()),
            &demo_source_directory(),
            "demo",
        )
        .expect("python generate should succeed");

        let generated_init_path = output_directory.join("demo/__init__.py");
        let generated_stub_path = output_directory.join("demo/__init__.pyi");
        let generated_native_path = output_directory.join("demo/_native.c");
        let generated_pyproject_path = output_directory.join("pyproject.toml");
        let generated_setup_path = output_directory.join("setup.py");
        let generated_init = fs::read_to_string(&generated_init_path)
            .expect("generated python init should be readable");
        let generated_stub = fs::read_to_string(&generated_stub_path)
            .expect("generated python typing stub should be readable");
        let generated_native = fs::read_to_string(&generated_native_path)
            .expect("generated native bridge should be readable");
        let generated_pyproject = fs::read_to_string(&generated_pyproject_path)
            .expect("generated pyproject should be readable");
        let generated_setup = fs::read_to_string(&generated_setup_path)
            .expect("generated setup.py should be readable");

        assert!(generated_init.contains("from pathlib import Path"));
        assert!(generated_init.contains("from . import _native"));
        assert!(generated_init.contains("_native._initialize_loader"));
        assert!(generated_init.contains("echo_i32"));
        assert!(generated_init.contains("PACKAGE_NAME = \"demo\""));
        assert!(generated_stub.contains("def echo_i32"));
        assert!(!generated_stub.contains("def echo_string"));
        assert!(!generated_stub.contains("def echo_vec_i32"));
        assert!(generated_pyproject.contains("setuptools.build_meta"));
        assert!(generated_setup.contains("Extension("));
        assert!(generated_setup.contains("\"demo._native\""));
        assert!(generated_native.contains("boltffi_python_echo_i32_symbol_fn"));
        assert!(!generated_native.contains("boltffi_echo_string"));
        assert!(!generated_native.contains("boltffi_echo_vec_i32"));
        assert!(generated_native.contains("boltffi_python_initialize_loader"));
        assert!(generated_native.contains("PyInit__native"));

        fs::remove_dir_all(output_directory).expect("cleanup generated output");
    }

    #[test]
    fn kmp_generate_writes_expected_artifacts() {
        let output_directory = unique_temp_dir("boltffi-kmp-generate-test");
        let config = parse_config(
            r#"
[package]
name = "demo"
version = "0.1.0"

[targets.kmp]
enabled = true
package = "com.example.demo"
module_name = "DemoKmp"
"#,
        );

        KmpGenerator::generate_from_source_directory(
            &config,
            Some(output_directory.clone()),
            &demo_source_directory(),
            "demo",
        )
        .expect("kmp generate should succeed");

        let common_path = output_directory
            .join("commonMain/kotlin/com/example/demo/DemoKmp.kt");
        let jvm_path = output_directory.join("jvmMain/kotlin/com/example/demo/DemoKmp.kt");
        let native_path = output_directory
            .join("nativeMain/kotlin/com/example/demo/DemoKmp.kt");
        let def_path = output_directory.join("include/demo.def");
        let header_path = output_directory.join("include/demo.h");
        let jni_path = output_directory.join("jvmMain/jni/jni_glue.c");
        let jvm_binding_path = output_directory
            .join("jvmMain/kotlin/com/example/demo/jvmffi/DemoKmpJvmFfi.kt");

        assert!(common_path.exists());
        assert!(jvm_path.exists());
        assert!(native_path.exists());
        assert!(def_path.exists());
        assert!(header_path.exists());
        assert!(jni_path.exists());
        assert!(jvm_binding_path.exists());

        let common_source = fs::read_to_string(&common_path).expect("common source readable");
        let def_source = fs::read_to_string(&def_path).expect("def source readable");
        assert!(common_source.contains("expect object DemoKmp"));
        assert!(common_source.contains("Skipped functions"));
        assert!(def_source.contains("headers = demo.h"));
        assert!(def_source.contains("package = com.example.demo.native"));

        fs::remove_dir_all(output_directory).expect("cleanup generated output");
    }
}
