//! Fechamento por evidência e `outcomes[]` (D48/D55, E09-T02).
//!
//! Fechar uma task deixa de ser declaração: os validators são executados (borda) e o
//! resultado volta como **evidência**. O `outcome` é **inferido** da severidade das falhas e a
//! confirmação é sempre **derivada** de `outcomes[]` — nunca armazenada (D48).

use crate::schema::{Status, Value};
use crate::store::Note;
use crate::task::{OutcomeStatus, is_task, validate_transition};
use crate::time::Timestamp;
use crate::write::{WriteAction, WriteContext, event};
use crate::{Error, Result};

use super::validator::Severity;

/// Resultado de um validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckResult {
    /// Passou.
    Pass,
    /// Falhou.
    Fail,
    /// Pulado.
    Skip,
}

impl CheckResult {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Skip => "skip",
        }
    }
}

/// Resultado de um validator com metadados de execução.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckOutcome {
    /// Nome do validator.
    pub name: String,
    /// Resultado.
    pub result: CheckResult,
    /// Severidade herdada do catálogo.
    pub severity: Severity,
    /// Duração da execução (ms), quando medida.
    pub duration_ms: Option<u64>,
    /// Saída relevante (nunca o corpo de uma nota).
    pub output: Option<String>,
}

/// Resultado do fechamento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseOutcome {
    /// Id da task.
    pub id: String,
    /// `outcome` inferido.
    pub status: OutcomeStatus,
    /// Nova revisão.
    pub revision: u32,
}

/// Infere o `outcome` a partir da severidade das falhas (D55).
#[must_use]
pub fn infer_outcome(results: &[CheckOutcome]) -> OutcomeStatus {
    if results.is_empty() {
        return OutcomeStatus::Abandoned;
    }
    if results
        .iter()
        .all(|check| check.result == CheckResult::Pass)
    {
        return OutcomeStatus::Success;
    }
    if results
        .iter()
        .any(|check| check.result == CheckResult::Fail && check.severity == Severity::Error)
    {
        return OutcomeStatus::Failure;
    }
    OutcomeStatus::Partial
}

/// Fecha a task gravando `outcomes[]` + `evidence` e inferindo o resultado.
///
/// # Errors
/// - `ErrorKind::InvalidInput` se não houver evidência ou a transição for proibida;
/// - `ErrorKind::Schema` se a nota não for tarefa;
/// - propaga erros de I/O.
pub fn close_task(
    ctx: &WriteContext<'_>,
    id: &str,
    results: &[CheckOutcome],
    agent: Option<&str>,
) -> Result<CloseOutcome> {
    if results.is_empty() {
        return Err(Error::invalid_input(
            "fechamento exige evidência de validators (D55)",
        ));
    }
    let mut note = ctx.store().read(id)?;
    if !is_task(&note) {
        return Err(Error::schema(format!("`{id}` não é tarefa/container")));
    }
    validate_transition(note.frontmatter.status()?, Status::Closed)?;

    let status = infer_outcome(results);
    let recorded_at = Timestamp::from_millis(ctx.now_ms()).to_rfc3339();
    append_outcome(&mut note, status, results, agent, &recorded_at)?;
    merge_evidence(&mut note, results, &recorded_at)?;
    note.frontmatter
        .set("status", Value::Str(Status::Closed.as_str().to_string()))?;

    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    let record = event("task", id, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("action", Value::Str("close".to_string()))
        .with_data("outcome", Value::Str(status.as_str().to_string()));
    ctx.events().append(&record)?;
    Ok(CloseOutcome {
        id: id.to_string(),
        status,
        revision,
    })
}

fn append_outcome(
    note: &mut Note,
    status: OutcomeStatus,
    results: &[CheckOutcome],
    agent: Option<&str>,
    recorded_at: &str,
) -> Result<()> {
    let mut items = match note.frontmatter.get("outcomes") {
        Some(Value::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    let mut entry = vec![
        (
            "status".to_string(),
            Value::Str(status.as_str().to_string()),
        ),
        (
            "recorded_at".to_string(),
            Value::Str(recorded_at.to_string()),
        ),
    ];
    let duration: u64 = results
        .iter()
        .filter_map(|check| check.duration_ms)
        .fold(0_u64, u64::saturating_add);
    if duration > 0 {
        entry.push((
            "duration".to_string(),
            Value::Int(i64::try_from(duration).unwrap_or(i64::MAX)),
        ));
    }
    if let Some(agent) = agent {
        entry.push(("agent".to_string(), Value::Str(agent.to_string())));
    }
    entry.push(("notes".to_string(), Value::Str(summary(results))));
    items.push(Value::map(entry));
    note.frontmatter.set("outcomes", Value::List(items))
}

fn merge_evidence(note: &mut Note, results: &[CheckOutcome], recorded_at: &str) -> Result<()> {
    let mut map = match note.frontmatter.get("evidence") {
        Some(Value::Map(map)) => map.clone(),
        _ => indexmap::IndexMap::new(),
    };
    for check in results {
        map.insert(
            check.name.clone(),
            Value::Str(format!("{}@{recorded_at}", check.result.as_str())),
        );
    }
    note.frontmatter.set("evidence", Value::Map(map))
}

fn summary(results: &[CheckOutcome]) -> String {
    let mut parts: Vec<String> = results
        .iter()
        .map(|check| format!("{}={}", check.name, check.result.as_str()))
        .collect();
    parts.sort();
    format!("checks: {}", parts.join(", "))
}
