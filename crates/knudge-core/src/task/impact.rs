//! Impacto derivado: tarefas abertas que dependem de um nó (D106/D109).
//!
//! O impacto é uma **projeção** do grafo (`depends_on` reverso transitivo): quantas tarefas
//! abertas ficam desbloqueadas quando `id` fecha. Nunca é armazenado.

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::schema::{EdgeKind, Status};

/// Número de tarefas **abertas** que dependem de `id` transitivamente (`depends_on` reverso).
///
/// Uma tarefa `t` conta quando existe caminho `t → … → id` por arestas `depends_on`, ou seja,
/// `id` bloqueia `t` direta ou indiretamente. Auto-dependência não conta.
#[must_use]
pub fn impact(graph: &Graph, id: &str) -> usize {
    graph
        .ids()
        .iter()
        .copied()
        .filter(|other| *other != id)
        .filter(|other| graph.is_work_item(other))
        .filter(|other| is_open(graph, other))
        .filter(|other| depends_on_transitively(graph, other, id))
        .count()
}

/// Impacto de **todos** os ids numa passada (E15-T20/O8.2): `id → nº de tarefas abertas que
/// dependem dele`.
///
/// Equivale a chamar [`impact`] para cada id, mas sem varrer o corpus por id: percorre o
/// `depends_on` a partir de cada item de trabalho aberto e credita cada ancestral alcançado.
#[must_use]
pub fn impacts(graph: &Graph) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for source in graph.ids() {
        if !graph.is_work_item(source) || !is_open(graph, source) {
            continue;
        }
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack = vec![source.to_string()];
        while let Some(current) = stack.pop() {
            for dependency in graph.targets(&current, EdgeKind::DependsOn) {
                if dependency.as_str() == source || !seen.insert(dependency.clone()) {
                    continue;
                }
                let count = counts.entry(dependency.clone()).or_default();
                *count = count.saturating_add(1);
                stack.push(dependency.clone());
            }
        }
    }
    counts
}

fn is_open(graph: &Graph, id: &str) -> bool {
    matches!(
        graph.status(id),
        Some(Status::Active | Status::InProgress | Status::Blocked)
    )
}

/// `true` se o status ainda pede ação (não `closed`/`superseded`/`forgotten`).
///
/// Compartilhado por `next:` (D106) e por `task list --sort impact` (D109): a lista por
/// impacto é o **caminho crítico**, não o histórico.
#[must_use]
pub fn is_actionable(status: Option<Status>) -> bool {
    !matches!(
        status,
        Some(Status::Closed | Status::Superseded | Status::Forgotten)
    )
}

fn depends_on_transitively(graph: &Graph, from: &str, target: &str) -> bool {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(current) = stack.pop() {
        for next in graph.targets(&current, EdgeKind::DependsOn) {
            if next == target {
                return true;
            }
            if seen.insert(next.clone()) {
                stack.push(next.clone());
            }
        }
    }
    false
}
