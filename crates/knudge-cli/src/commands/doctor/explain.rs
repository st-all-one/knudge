//! Achados detalhados de `kd doctor --explain` (`esperado` × `encontrado` × `ação`) (D163).

use knudge_core::health::{AuditReport, CheckId, DoctorReport};

/// Achado detalhado.
pub(super) struct Entry {
    pub(super) kind: &'static str,
    pub(super) title: &'static str,
    pub(super) expected: String,
    pub(super) found: String,
    pub(super) action: String,
}

/// Todos os achados: checks que falharam + problemas da auditoria.
pub(super) fn entries(report: &DoctorReport, audit: &AuditReport) -> Vec<Entry> {
    let mut entries = check_entries(report);
    audit_entries(audit, &mut entries);
    entries
}

fn check_entries(report: &DoctorReport) -> Vec<Entry> {
    report
        .checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| Entry {
            kind: check.id.as_str(),
            title: check_title(check.id),
            expected: "check ok".to_string(),
            found: check.detail.clone(),
            action: if check.fixable {
                "kd doctor --fix".to_string()
            } else {
                "kd doctor --explain".to_string()
            },
        })
        .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "builder linear de achados; cada bloco é um problema distinto da auditoria"
)]
fn audit_entries(audit: &AuditReport, entries: &mut Vec<Entry>) {
    if !audit.duplicates.is_empty() {
        entries.push(entry(
            "duplicates",
            "Quase-duplicatas",
            "nenhum par",
            format!("{} par(es)", audit.duplicates.len()),
            "kd maintenance compact --universe",
        ));
    }
    if !audit.broken_anchors.is_empty() {
        entries.push(entry(
            "anchors",
            "Âncoras quebradas",
            "nenhuma",
            format!("{} âncora(s)", audit.broken_anchors.len()),
            "kd doctor --fix",
        ));
    }
    if !audit.missing_edges.is_empty() {
        entries.push(entry(
            "edges",
            "Arestas sugeridas",
            "nenhuma pendente",
            format!("{} aresta(s)", audit.missing_edges.len()),
            "kd write --link <FROM:ARESTA:TO>",
        ));
    }
    if !audit.stale_locks.is_empty() {
        entries.push(entry(
            "locks",
            "Locks abandonados",
            "nenhum",
            format!("{} lock(s)", audit.stale_locks.len()),
            "kd doctor --fix",
        ));
    }
    if !audit.integrity.is_empty()
        || !audit.supersession_cycles.is_empty()
        || !audit.dependency_cycles.is_empty()
    {
        let total = audit
            .integrity
            .len()
            .saturating_add(audit.supersession_cycles.len())
            .saturating_add(audit.dependency_cycles.len());
        entries.push(entry(
            "integrity",
            "Integridade/ciclos",
            "nenhum",
            format!("{total} problema(s)"),
            "kd doctor --explain",
        ));
    }
}

fn entry(
    kind: &'static str,
    title: &'static str,
    expected: &'static str,
    found: String,
    action: &'static str,
) -> Entry {
    Entry {
        kind,
        title,
        expected: expected.to_string(),
        found,
        action: action.to_string(),
    }
}

/// Título humano de um check.
pub(super) const fn check_title(id: CheckId) -> &'static str {
    match id {
        CheckId::Schema => "Schema/TOON",
        CheckId::Integrity => "Integridade do grafo",
        CheckId::Cycles => "Ciclos",
        CheckId::Anchors => "Âncoras",
        CheckId::ProgramAnchor => "Âncora de programa",
        CheckId::Duplicates => "Quase-duplicatas",
        CheckId::Locks => "Locks",
        CheckId::Config => "Configuração",
        CheckId::BodyHash => "body_hash",
        CheckId::Events => "Eventos",
        CheckId::Derived => "Canônico × derivado",
        CheckId::Embeddings => "Embeddings",
        CheckId::Body => "Corpo",
    }
}
