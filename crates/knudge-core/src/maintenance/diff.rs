//! `diff` sobre eventos (E08-T05).
//!
//! O passado é derivado da **auditoria** (`eventos/`), não do git global (D21/D33). O intervalo
//! `[since, until]` é opcional e o escopo filtra por pertencimento ao container.

use crate::graph::Graph;
use crate::handoff::manifest::belongs_to;
use crate::store::Event;

/// Entrada de `diff`: uma operação sobre uma nota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    /// Operação (`write`, `update`, `supersede`, …).
    pub op: String,
    /// Nota afetada.
    pub id: String,
    /// Instante (ms).
    pub at: i64,
}

/// Filtra e ordena os eventos por intervalo e escopo.
#[must_use]
pub fn diff(
    events: &[Event],
    since: Option<i64>,
    until: Option<i64>,
    scope: Option<&str>,
    graph: &Graph,
) -> Vec<DiffEntry> {
    let mut entries = Vec::new();
    for event in events {
        if since.is_some_and(|start| event.at < start) {
            continue;
        }
        if until.is_some_and(|end| event.at > end) {
            continue;
        }
        let Some(id) = &event.note_id else {
            continue;
        };
        if let Some(container) = scope
            && !belongs_to(graph, id, container)
        {
            continue;
        }
        entries.push(DiffEntry {
            op: event.op.clone(),
            id: id.clone(),
            at: event.at,
        });
    }
    entries.sort_by(|left, right| {
        left.at
            .cmp(&right.at)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.op.cmp(&right.op))
    });
    entries.dedup();
    entries
}
