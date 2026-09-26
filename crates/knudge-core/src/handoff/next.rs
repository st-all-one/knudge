//! Linhas dinâmicas do manifest: `next:` e `fresh:` (D106).
//!
//! O `prime` continua **estático** (D57); estas linhas só aparecem no `rewind` e são derivadas
//! do índice/grafo + shelf-life. `next` lista as tarefas `ready` de maior impacto; `fresh`
//! resume o frescor do corpus.

use std::collections::BTreeMap;

use crate::graph::Graph;
use crate::lifecycle::Freshness;
use crate::retrieval::views::compute_views;
use crate::retrieval::{Index, Meta};
use crate::task::{impact, is_actionable};

use super::CorpusScope;
use super::manifest::{manifest_text, sanitize};

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
    let views = compute_views(graph);
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
    let mut ready: Vec<&str> = views
        .ready
        .iter()
        .map(String::as_str)
        .filter(|id| is_actionable(graph.status(id)))
        .filter(|id| metas.get(id).is_some_and(|meta| scope.matches(meta)))
        .collect();
    ready.sort_unstable_by(|left, right| {
        impact(graph, right)
            .cmp(&impact(graph, left))
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
        .map(|id| NextTask {
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
    let ready = next_tasks(index, graph, usize::MAX, scope);
    let shown = ready.len().min(limit);
    let mut text = manifest_text(index, graph, changed_paths, scope);
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
