//! Execução de validators de tarefa na borda (E09-T01 + E12-T04, D54).

use knudge_core::Result;
use knudge_core::adapters::ProcessHookRunner;
use knudge_core::health::validator::{ValidatorCatalog, resolve_checks};
use knudge_core::health::{CheckOutcome, CheckResult};
use knudge_core::ports::HookRunner;
use knudge_core::store::Note;

use crate::session::Session;

/// Resultado da execução dos validators de uma tarefa.
pub struct CheckRun {
    /// Evidências por validator.
    pub outcomes: Vec<CheckOutcome>,
    /// Avisos (ex.: checks explícitos ausentes do catálogo).
    pub warnings: Vec<String>,
}

/// Executa os validators resolvidos para a nota (evidência de fechamento).
///
/// # Errors
/// Propaga erros de leitura do catálogo e de execução dos comandos.
pub fn run(session: &Session, note: &Note) -> Result<CheckRun> {
    let catalog = ValidatorCatalog::load(session.fs_dyn(), &session.knowledge_dir())?;
    let explicit: Vec<String> = note
        .frontmatter
        .string_list("checks")?
        .into_iter()
        .map(str::to_string)
        .collect();
    let anchors: Vec<String> = note
        .frontmatter
        .string_list("anchors")?
        .into_iter()
        .map(str::to_string)
        .collect();
    let resolved = resolve_checks(&catalog, &explicit, &anchors);
    let runner = ProcessHookRunner::with_default_timeout(session.project_root());
    let mut outcomes = Vec::new();
    for check in &resolved.checks {
        let Some(validator) = catalog.get(&check.name) else {
            continue;
        };
        let output = runner.run(&validator.cmd, &[])?;
        let result = if output.status == 0 {
            CheckResult::Pass
        } else {
            CheckResult::Fail
        };
        outcomes.push(CheckOutcome {
            name: check.name.clone(),
            result,
            severity: check.severity,
            duration_ms: None,
            output: Some(format!("status {}", output.status)),
        });
    }
    let warnings = resolved
        .missing
        .iter()
        .map(|name| format!("check explícito ausente do catálogo: {name}"))
        .collect();
    Ok(CheckRun { outcomes, warnings })
}
