//! `kd init` — funda `.knudge/` e emite o prompt inicial (E12-T06, D57/D60).

use knudge_core::Result;
use knudge_core::git::{OnboardOptions, onboard};
use serde_json::json;

use crate::cli::InitArgs;
use crate::commands::prime;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd init` (idempotente; `--force` sobrescreve a config).
///
/// # Errors
/// Propaga erros de resolução de projeto, I/O e config.
pub fn run(session: &Session, args: &InitArgs) -> Result<Output> {
    let report = onboard(
        session.fs_dyn(),
        session.git(),
        session.env(),
        OnboardOptions { force: args.force },
    )?;
    let prompt = if args.no_prompt {
        String::new()
    } else {
        prime::run(prime::PrimeFormat::Short).text
    };
    let text = if prompt.is_empty() {
        format!(
            "projeto {} fundado em {}\n",
            report.name,
            report.knowledge_dir.display()
        )
    } else {
        format!(
            "projeto {} fundado em {}\n\n{prompt}",
            report.name,
            report.knowledge_dir.display()
        )
    };
    let data = json!({
        "project": report.name,
        "root": report.root,
        "knowledge_dir": report.knowledge_dir,
        "in_repo": report.in_repo,
        "config_written": report.config_written,
        "exclude_changed": report.exclude_changed,
        "attributes_changed": report.attributes_changed,
        "agents_changed": report.agents_changed,
    });
    Ok(Output::new(text, data))
}
