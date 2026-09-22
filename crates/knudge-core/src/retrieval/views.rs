//! Views `ready`/`blocked` (D53).
//!
//! São **computadas** a partir do `depends_on` transitivo — não são tools. Uma tarefa está
//! `ready` se toda a sua dependência transitiva está encerrada (`closed`/`superseded`) e o
//! agendamento `not_before` já venceu (D56); caso contrário (ou se participa de um ciclo de
//! dependência) está `blocked`.
//!
//! [`compute_views`] é **estática** (ignora `not_before`) para preservar o `prime` byte-idêntico
//! (D57); [`compute_views_at`] considera o relógio e é usada na leitura dinâmica.

use std::collections::BTreeSet;

use crate::graph::Graph;
use crate::schema::{EdgeKind, NoteType, Status};

/// Views de trabalho derivadas do grafo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Views {
    /// Tarefas prontas (todas as dependências resolvidas).
    pub ready: BTreeSet<String>,
    /// Tarefas bloqueadas (dependência aberta, ciclo ou agendamento futuro).
    pub blocked: BTreeSet<String>,
}

/// Motivo pelo qual uma tarefa está bloqueada (D104).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// Dependência aberta (menor id pendente na travessia transitória).
    Dependency(String),
    /// Agendamento futuro (`not_before`, ms desde a época — D56).
    Scheduled(i64),
    /// Ciclo de dependência (D45).
    Cycle,
}

/// Motivo do bloqueio de `id` em `now_ms`; `None` se `ready` ou não for tarefa (D104).
#[must_use]
pub fn block_reason(graph: &Graph, id: &str, now_ms: i64) -> Option<BlockReason> {
    if graph.note_type(id) != Some(NoteType::Task) {
        return None;
    }
    let cyclic: BTreeSet<String> = graph.dependency_cycles().into_iter().flatten().collect();
    if cyclic.contains(id) {
        return Some(BlockReason::Cycle);
    }
    if let Some(not_before) = graph.not_before(id)
        && now_ms < not_before
    {
        return Some(BlockReason::Scheduled(not_before));
    }
    pending_dependency(graph, id).map(BlockReason::Dependency)
}

fn pending_dependency(graph: &Graph, id: &str) -> Option<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut pending: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![id.to_string()];
    while let Some(current) = stack.pop() {
        for dependency in graph.targets(&current, EdgeKind::DependsOn) {
            if !seen.insert(dependency.clone()) {
                continue;
            }
            match graph.status(dependency) {
                Some(Status::Closed | Status::Superseded) => {}
                _ => {
                    pending.insert(dependency.clone());
                }
            }
            stack.push(dependency.clone());
        }
    }
    pending.into_iter().next()
}

/// Computa as views `ready`/`blocked` **sem** considerar `not_before` (estático — D57).
#[must_use]
pub fn compute_views(graph: &Graph) -> Views {
    compute_views_at(graph, i64::MAX)
}

/// Computa as views `ready`/`blocked` no instante `now_ms` (considera `not_before` — D56).
#[must_use]
pub fn compute_views_at(graph: &Graph, now_ms: i64) -> Views {
    let cyclic: BTreeSet<String> = graph.dependency_cycles().into_iter().flatten().collect();
    let mut views = Views::default();
    for id in graph.ids() {
        if graph.note_type(id) != Some(NoteType::Task) {
            continue;
        }
        let scheduled = graph
            .not_before(id)
            .is_some_and(|not_before| now_ms < not_before);
        if cyclic.contains(id) || scheduled || !dependencies_satisfied(graph, id) {
            views.blocked.insert(id.to_string());
        } else {
            views.ready.insert(id.to_string());
        }
    }
    views
}

fn dependencies_satisfied(graph: &Graph, id: &str) -> bool {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![id.to_string()];
    while let Some(current) = stack.pop() {
        for dependency in graph.targets(&current, EdgeKind::DependsOn) {
            if !seen.insert(dependency.clone()) {
                continue;
            }
            match graph.status(dependency) {
                Some(Status::Closed | Status::Superseded) => {}
                _ => return false,
            }
            stack.push(dependency.clone());
        }
    }
    true
}
