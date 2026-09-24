//! Portão de evidência de propostas na borda (D156).
//!
//! Lê os gates configurados (`proposals.gate`), executa cada comando do catálogo
//! (`validators.toml`, `kind = "gate"`) passando `{op, before, after}` em stdin e interpreta o
//! JSON de saída (`{passed, score_before, score_after}`). A decisão pura fica em
//! [`knudge_core::health::gate`]; aqui só há E/S. Sem shell (R12).

use knudge_core::Result;
use knudge_core::adapters::ProcessHookRunner;
use knudge_core::health::gate::{GateOutcome, accept};
use knudge_core::health::validator::ValidatorCatalog;
use knudge_core::ports::HookRunner;
use serde_json::{Value, json};

use crate::session::Session;

/// Estado do portão para uma transformação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    /// Nenhum gate configurado.
    Disabled,
    /// Todos aprovaram.
    Passed,
    /// Ao menos um reprovou.
    Failed,
}

/// Relatório do portão para uma transformação.
pub struct GateReport {
    /// Estado geral.
    pub status: GateStatus,
    /// Veredito por gate.
    pub outcomes: Vec<(String, GateOutcome)>,
    /// Avisos de degradação (gate ausente, JSON inválido, timeout).
    pub warnings: Vec<String>,
}

impl GateReport {
    /// Havia gates configurados.
    #[must_use]
    pub fn configured(&self) -> bool {
        !matches!(self.status, GateStatus::Disabled)
    }

    /// Todos aprovaram (respeitando `min_delta`).
    #[must_use]
    pub fn passed(&self) -> bool {
        matches!(self.status, GateStatus::Passed)
    }

    /// Nomes dos gates que reprovaram.
    #[must_use]
    pub fn failed(&self) -> Vec<String> {
        self.outcomes
            .iter()
            .filter(|(_, outcome)| !outcome.passed)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Representação JSON do relatório.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "configured": self.configured(),
            "passed": self.passed(),
            "outcomes": self.outcomes.iter().map(|(name, outcome)| json!({
                "name": name,
                "passed": outcome.passed,
                "score_before": outcome.score_before,
                "score_after": outcome.score_after,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Nomes dos gates configurados em `proposals.gate` (lista separada por vírgula).
#[must_use]
pub fn gate_names(session: &Session) -> Vec<String> {
    session
        .config()
        .get_str("proposals.gate")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

/// `true` se o portão está em modo de bloqueio (`proposals.enforce`).
#[must_use]
pub fn enforced(session: &Session) -> bool {
    session
        .config()
        .get_bool("proposals.enforce")
        .unwrap_or(false)
        && !gate_names(session).is_empty()
}

/// `min_delta` configurado.
fn min_delta(session: &Session) -> f64 {
    session
        .config()
        .get_float("proposals.min_delta")
        .unwrap_or(0.0)
}

/// Avalia os gates configurados sobre `{op, before, after}`.
///
/// # Errors
/// Propaga erro de leitura do catálogo; falha de execução vira veredito reprovado + aviso
/// (degradação graciosa, R33).
pub fn evaluate(
    session: &Session,
    op: &str,
    before: Option<&Value>,
    after: &Value,
) -> Result<GateReport> {
    let names = gate_names(session);
    if names.is_empty() {
        return Ok(GateReport {
            status: GateStatus::Disabled,
            outcomes: Vec::new(),
            warnings: Vec::new(),
        });
    }
    let catalog = ValidatorCatalog::load(session.fs_dyn(), &session.knowledge_dir())?;
    let runner = ProcessHookRunner::with_default_timeout(session.project_root());
    let input = serde_json::to_vec(&json!({
        "op": op,
        "before": before,
        "after": after,
    }))
    .map_err(|error| knudge_core::Error::internal(format!("entrada do gate: {error}")))?;
    let min = min_delta(session);
    let mut outcomes = Vec::new();
    let mut warnings = Vec::new();
    for name in names {
        let Some(validator) = catalog.get(&name) else {
            warnings.push(format!("gate ausente do catálogo: {name}"));
            outcomes.push((name, rejected()));
            continue;
        };
        match run_one(&runner, &validator.cmd, &input) {
            Ok(outcome) => outcomes.push((name, outcome)),
            Err(error) => {
                warnings.push(format!("gate {name}: {error}"));
                outcomes.push((name, rejected()));
            }
        }
    }
    let passed = !outcomes.is_empty() && outcomes.iter().all(|(_, outcome)| accept(outcome, min));
    Ok(GateReport {
        status: if passed {
            GateStatus::Passed
        } else {
            GateStatus::Failed
        },
        outcomes,
        warnings,
    })
}

fn run_one(runner: &dyn HookRunner, cmd: &str, input: &[u8]) -> Result<GateOutcome> {
    let output = runner.run(cmd, input)?;
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(GateOutcome {
            passed: output.status == 0,
            score_before: 0.0,
            score_after: 0.0,
        });
    }
    let value: Value = serde_json::from_str(trimmed).map_err(|error| {
        knudge_core::Error::invalid_input(format!("gate devolveu JSON inválido: {error}"))
    })?;
    let passed = value
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or(output.status == 0);
    let score_before = value
        .get("score_before")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let score_after = value
        .get("score_after")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    Ok(GateOutcome {
        passed,
        score_before,
        score_after,
    })
}

/// Veredito reprovado e neutro (usado em degradação graciosa).
fn rejected() -> GateOutcome {
    GateOutcome {
        passed: false,
        score_before: 0.0,
        score_after: 0.0,
    }
}
