//! Renderização de `kd doctor`: texto (LLM), envelope JSON (máquina) e sugestões (D163).

use std::collections::BTreeMap;

use knudge_core::health::{AuditReport, DoctorCheck, DoctorReport};
use knudge_core::schema::EdgeKind;
use serde_json::{Value, json};

use super::explain::{self, check_title};

/// Ação acionável exibida em `próximos:` e no JSON.
pub(super) struct Suggestion {
    /// Achado de origem.
    pub(super) category: &'static str,
    /// Comando/ação concreta.
    pub(super) action: String,
}

/// Nível de detalhe do relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Detail {
    /// Resumo (padrão).
    Summary,
    /// `--explain`: detalha cada achado.
    Explained,
}

impl Detail {
    const fn explained(self) -> bool {
        matches!(self, Self::Explained)
    }
}

/// Classificação de saúde: `status` + flags `healthy`/`degraded`.
///
/// `healthy` segue D119 (advisórios não bloqueiam); `degraded` distingue "só advisórios" de
/// "saudável de fato", preservando a garantia sem quebrar a decisão fechada.
fn classify(report: &DoctorReport, audit: &AuditReport) -> (&'static str, bool, bool) {
    let all_ok = report.checks.iter().all(|check| check.ok);
    let hard_fail = report
        .checks
        .iter()
        .any(|check| !check.ok && !check.is_warning());
    let healthy = !hard_fail && audit.is_clean();
    let degraded = healthy && !all_ok;
    let status = if all_ok && audit.is_clean() {
        "healthy"
    } else if degraded {
        "degraded"
    } else {
        "unhealthy"
    };
    (status, healthy, degraded)
}

/// Monta a lista `próximos:` a partir dos checks e da auditoria.
pub(super) fn suggestions(report: &DoctorReport, audit: &AuditReport) -> Vec<Suggestion> {
    let mut by_category: BTreeMap<&'static str, String> = BTreeMap::new();
    for check in &report.checks {
        if check.ok {
            continue;
        }
        let action = if check.fixable {
            "kd doctor --fix"
        } else {
            "kd doctor --explain"
        };
        by_category
            .entry(check.id.as_str())
            .or_insert_with(|| action.to_string());
    }
    if !audit.duplicates.is_empty() {
        by_category.insert(
            "duplicates",
            "kd maintenance compact --universe".to_string(),
        );
    }
    if !audit.broken_anchors.is_empty() {
        by_category.insert("anchors", "kd doctor --fix".to_string());
    }
    if let Some(edge) = audit.missing_edges.first() {
        let target = edge.targets.first().map_or("<TO>", String::as_str);
        by_category.insert(
            "edges",
            format!("kd write --link {}:{}:{target}", edge.id, edge.kind),
        );
    }
    if !audit.stale_locks.is_empty() {
        by_category.insert("locks", "kd doctor --fix".to_string());
    }
    if !audit.integrity.is_empty()
        || !audit.supersession_cycles.is_empty()
        || !audit.dependency_cycles.is_empty()
    {
        by_category.insert("integrity", "kd doctor --explain".to_string());
    }
    by_category
        .into_iter()
        .map(|(category, action)| Suggestion { category, action })
        .collect()
}

/// Texto para o pipe (LLM).
pub(super) fn text(
    report: &DoctorReport,
    audit: &AuditReport,
    suggestions: &[Suggestion],
    detail: Detail,
) -> String {
    let mut lines: Vec<String> = report.checks.iter().map(check_line).collect();
    lines.push(if audit.is_clean() {
        "auditoria: limpo".to_string()
    } else {
        format!("auditoria: {} problema(s)", audit.total())
    });
    if detail.explained() {
        for entry in explain::entries(report, audit) {
            lines.push(String::new());
            lines.push(format!("[{}] {}", entry.kind, entry.title));
            lines.push(format!("  esperado: {}", entry.expected));
            lines.push(format!("  encontrado: {}", entry.found));
            lines.push(format!("  ação: {}", entry.action));
        }
    }
    if !suggestions.is_empty() {
        lines.push(String::new());
        lines.push("próximos:".to_string());
        for suggestion in suggestions {
            lines.push(format!(
                "  - {} ({})",
                suggestion.action, suggestion.category
            ));
        }
    }
    lines.join("\n")
}

/// Envelope JSON de `kd doctor`.
pub(super) fn json(
    report: &DoctorReport,
    audit: &AuditReport,
    suggestions: &[Suggestion],
    detail: Detail,
) -> Value {
    let (status, healthy, degraded) = classify(report, audit);
    let mut data = json!({
        "healthy": healthy,
        "degraded": degraded,
        "status": status,
        "checks": report.checks.iter().map(|check| json!({
            "id": check.id.as_str(),
            "title": check_title(check.id),
            "ok": check.ok,
            "warn": check.is_warning(),
            "detail": check.detail,
            "fixable": check.fixable,
        })).collect::<Vec<_>>(),
        "fixed": report.fixed,
        "audit": audit_json(audit),
        "suggestions": suggestions.iter().map(|item| json!({
            "category": item.category,
            "action": item.action,
        })).collect::<Vec<_>>(),
    });
    if detail.explained()
        && let Some(map) = data.as_object_mut()
    {
        map.insert("explain".to_string(), explain_json(report, audit));
    }
    data
}

fn explain_json(report: &DoctorReport, audit: &AuditReport) -> Value {
    json!(
        explain::entries(report, audit)
            .iter()
            .map(|entry| json!({
                "kind": entry.kind,
                "title": entry.title,
                "expected": entry.expected,
                "found": entry.found,
                "action": entry.action,
            }))
            .collect::<Vec<_>>()
    )
}

fn check_line(check: &DoctorCheck) -> String {
    let status = if check.ok {
        "ok"
    } else if check.is_warning() {
        "warn"
    } else {
        "fail"
    };
    format!("{status} {} {}", check.id.as_str(), check.detail)
}

fn audit_json(report: &AuditReport) -> Value {
    json!({
        "clean": report.is_clean(),
        "total": report.total(),
        "integrity_issues": report.integrity.iter().map(|issue| json!({
            "kind": issue.kind.as_str(),
            "from": issue.from,
            "to": issue.to,
            "edge": issue.edge.map(EdgeKind::as_str),
        })).collect::<Vec<_>>(),
        "supersession_cycle_details": report.supersession_cycles,
        "dependency_cycle_details": report.dependency_cycles,
        "broken_anchor_details": report.broken_anchors.iter().map(|anchor| json!({
            "id": anchor.id,
            "anchor": anchor.anchor,
        })).collect::<Vec<_>>(),
        "duplicate_pairs": report.duplicates.iter().map(|dup| json!({
            "keep": dup.keep,
            "drop": dup.drop,
            "score": dup.score,
        })).collect::<Vec<_>>(),
        "missing_edge_details": report.missing_edges.iter().map(|edge| json!({
            "id": edge.id,
            "kind": edge.kind,
            "targets": edge.targets,
            "reason": edge.reason,
        })).collect::<Vec<_>>(),
        "stale_lock_details": report.stale_locks.iter().map(|lock| json!({
            "path": lock.path,
            "age_ms": lock.age_ms,
        })).collect::<Vec<_>>(),
    })
}
