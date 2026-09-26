//! Saída comum e portão de evidência de `learn`/`compact` (D156/D165).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use knudge_core::Result;
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::super::gate;

/// Relatórios de gate paralelos às propostas.
pub(super) type Reports = Vec<Option<gate::GateReport>>;

/// Sufixo de uma linha de proposta com o veredito do gate (D156).
fn gate_suffix(base: String, report: Option<&gate::GateReport>) -> String {
    match report {
        Some(report) => format!(
            "{base}|gate={}",
            if report.passed() { "passed" } else { "failed" }
        ),
        None => base,
    }
}

/// Modo de verificação do portão (D156) — enum para não usar `bool` em parâmetro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyMode {
    /// Não roda o portão.
    Skip,
    /// Roda o portão (read-only) sobre cada proposta.
    Run,
}

/// Constrói os relatórios de gate (paralelos às propostas) e os avisos de degradação.
pub(super) fn build_reports<T, F>(
    session: &Session,
    verify: VerifyMode,
    proposals: &[T],
    mut describe: F,
) -> Result<(Reports, Vec<String>)>
where
    F: FnMut(&T) -> (String, serde_json::Value),
{
    if verify == VerifyMode::Skip {
        return Ok(((0..proposals.len()).map(|_| None).collect(), Vec::new()));
    }
    let mut reports = Vec::with_capacity(proposals.len());
    let mut warnings = Vec::new();
    for proposal in proposals {
        let (op, after) = describe(proposal);
        let report = gate::evaluate(session, &op, None, &after)?;
        warnings.extend(report.warnings.iter().cloned());
        reports.push(Some(report));
    }
    Ok((reports, warnings))
}

/// Renderiza a saída comum de `learn`/`compact` (texto + JSON + gate + avisos).
pub(super) fn render_proposals<T, FT, FJ>(
    proposals: &[T],
    reports: &[Option<gate::GateReport>],
    warnings: Vec<String>,
    base: FT,
    object: FJ,
) -> Output
where
    FT: Fn(&T) -> String,
    FJ: Fn(&T) -> serde_json::Value,
{
    let text = proposals
        .iter()
        .zip(reports)
        .map(|(proposal, report)| gate_suffix(base(proposal), report.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");
    let items: Vec<serde_json::Value> = proposals
        .iter()
        .zip(reports)
        .map(|(proposal, report)| {
            let mut value = object(proposal);
            if let Some(report) = report
                && let Some(map) = value.as_object_mut()
            {
                let _ignored = map.insert("gate".to_string(), report.to_json());
            }
            value
        })
        .collect();
    Output::new(text, json!({ "proposals": items })).with_warnings(warnings)
}

/// Contagem determinística por rótulo (ordem de `BTreeMap`).
pub(super) fn count_kinds<T, F>(items: &[T], label: F) -> Vec<(&'static str, usize)>
where
    F: Fn(&T) -> &'static str,
{
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for item in items {
        let entry = counts.entry(label(item)).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    counts.into_iter().collect()
}

/// Acrescenta a contagem por `kind` e o `próximos:` ao texto (D165); o JSON não muda.
pub(super) fn annotate(mut output: Output, counts: &[(&'static str, usize)], next: &str) -> Output {
    let summary = if counts.is_empty() {
        "nenhuma".to_string()
    } else {
        counts
            .iter()
            .map(|(kind, total)| format!("{kind}={total}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if !output.text.is_empty() {
        output.text.push('\n');
    }
    let _ignored = write!(output.text, "propostas: {summary}\npróximos: {next}");
    output
}
