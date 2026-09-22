//! Views `ready`/`blocked` (D53).
//!
//! São **computadas** a partir do `depends_on` transitivo — não são tools. Uma tarefa está
//! `ready` se toda a sua dependência transitiva está encerrada (`closed`/`superseded`); caso
//! contrário (ou se participa de um ciclo de dependência) está `blocked`.

use std::collections::BTreeSet;

use crate::graph::Graph;
use crate::schema::{EdgeKind, NoteType, Status};

/// Views de trabalho derivadas do grafo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Views {
    /// Tarefas prontas (todas as dependências resolvidas).
    pub ready: BTreeSet<String>,
    /// Tarefas bloqueadas (dependência aberta ou ciclo).
    pub blocked: BTreeSet<String>,
}

/// Computa as views `ready`/`blocked`.
#[must_use]
pub fn compute_views(graph: &Graph) -> Views {
    let cyclic: BTreeSet<String> = graph.dependency_cycles().into_iter().flatten().collect();
    let mut views = Views::default();
    for id in graph.ids() {
        if graph.note_type(id) != Some(NoteType::Task) {
            continue;
        }
        if cyclic.contains(id) || !dependencies_satisfied(graph, id) {
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
