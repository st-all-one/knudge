//! Linhas dinâmicas do manifest: `next:` e `fresh:` (D106).
//!
//! O `prime` continua **estático** (D57); estas linhas só aparecem no `rewind` e são derivadas
//! do índice/grafo + shelf-life. `next` lista as tarefas `ready` de maior impacto; `fresh`
//! resume o frescor do corpus.

use std::collections::BTreeMap;

use crate::graph::Graph;
use crate::lifecycle::Freshness;
use crate::retrieval::views::{Views, compute_views};
use crate::retrieval::{Index, Meta};
use crate::task::{impacts, is_actionable};

use super::CorpusScope;
use super::manifest::{manifest_text_in, sanitize};

/// Máximo de tarefas `ready` listadas em `next:`.
pub const MAX_NEXT: usize = 5;

/// Tokens (estimados) reservados por item de `next` para derivar `K` do orçamento.
const NEXT_BUDGET_DIVISOR: usize = 200;

/// Tarefa `ready` para a linha `next:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextTask {
    /// Id.
    pub id: String,
    /// `statement` normalizado (sem `|`/quebras).
    pub statement: String,
}

/// Até `limit` tarefas `ready` e **abertas** por `(impacto desc, created asc, id asc)` (D106/D109).
#[must_use]
pub fn next_tasks(
    index: &Index,
    graph: &Graph,
    limit: usize,
    scope: &CorpusScope,
) -> Vec<NextTask> {
    next_tasks_in(&compute_views(graph), index, graph, limit, scope)
}

/// Como [`next_tasks`], mas com as `views` já computadas (evita recomputar o SCC — O5.2).
fn next_tasks_in(
    views: &Views,
    index: &Index,
    graph: &Graph,
    limit: usize,
    scope: &CorpusScope,
) -> Vec<NextTask> {
    let created: BTreeMap<&str, i64> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc.meta.created_ms))
        .collect();
    let statements: BTreeMap<&str, &str> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc.statement.as_str()))
        .collect();
    let metas: BTreeMap<&str, &Meta> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), &doc.meta))
        .collect();
    // Impacto é pré-computado **numa passada** (antes era chamado por id — O5.3/O8.2).
    let impact_by_id = impacts(graph);
    let mut ready: Vec<(&str, usize)> = views
        .ready
        .iter()
        .map(String::as_str)
        .filter(|id| is_actionable(graph.status(id)))
        .filter(|id| metas.get(id).is_some_and(|meta| scope.matches(meta)))
        .map(|id| (id, impact_by_id.get(id).copied().unwrap_or(0)))
        .collect();
    ready.sort_unstable_by(|(left, left_impact), (right, right_impact)| {
        right_impact
            .cmp(left_impact)
            .then_with(|| {
                created
                    .get(left)
                    .copied()
                    .unwrap_or(0)
                    .cmp(&created.get(right).copied().unwrap_or(0))
            })
            .then_with(|| left.cmp(right))
    });
    ready
        .into_iter()
        .take(limit)
        .map(|(id, _impact)| NextTask {
            id: id.to_string(),
            statement: sanitize(statements.get(id).copied().unwrap_or("")),
        })
        .collect()
}

/// Manifest dinâmico: contadores + `next:` + `fresh:`; retorna `(texto, descartados)`.
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "`manifest_at` espelha as entradas derivadas do `RewindInput` (D106/D143)"
)]
pub fn manifest_at(
    index: &Index,
    graph: &Graph,
    changed_paths: &[String],
    freshness: &Freshness,
    budget: usize,
    scope: &CorpusScope,
) -> (String, usize) {
    let limit = (budget / NEXT_BUDGET_DIVISOR).clamp(1, MAX_NEXT);
    // Views computadas uma única vez para `next` + contadores do manifest (O5.2).
    let views = compute_views(graph);
    let ready = next_tasks_in(&views, index, graph, usize::MAX, scope);
    let shown = ready.len().min(limit);
    let mut text = manifest_text_in(&views, index, graph, changed_paths, scope);
    if shown > 0 {
        let items: Vec<String> = ready
            .iter()
            .take(shown)
            .map(|task| format!("{}|{}", task.id, task.statement))
            .collect();
        text.push_str("\nnext: ");
        text.push_str(&items.join(" "));
    }
    text.push_str("\nfresh: ");
    text.push_str(&freshness.render());
    (text, ready.len().saturating_sub(shown))
}
