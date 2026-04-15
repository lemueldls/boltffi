use crate::cli::{CliError, Result};
use crate::commands::generate::generator::{GenerateRequest, LanguageGenerator};
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

        Err(CliError::CommandFailed {
            command: "kmp generation is not implemented yet".to_string(),
            status: None,
        })
    }
}
