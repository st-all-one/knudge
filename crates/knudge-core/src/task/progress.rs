//! Rollup de progresso por épico — derivado, sem verdade nova (D127).
//!
//! A conclusão de um épico é contada sobre os **itens de trabalho**
//! ([`Graph::is_work_item`]: `issue`/`task` com `scope`) no seu subárvore; `done` é quantos
//! estão `closed`. Nada é gravado: o número é recomputado do grafo a cada leitura, como as
//! views `ready`/`blocked` e o `impact`.

use crate::graph::Graph;
use crate::schema::{EdgeKind, Scope, Status};

use super::TaskRef;

/// Progresso de conclusão de um container: `done/total`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    /// Itens concluídos (`closed`).
    pub done: u32,
    /// Itens de trabalho no escopo.
    pub total: u32,
}

impl Progress {
    /// Constrói um progresso.
    #[must_use]
    pub const fn new(done: u32, total: u32) -> Self {
        Self { done, total }
    }

    /// `true` se não há itens de trabalho no escopo.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.total == 0
    }

    /// Fração em `[0, 1]` (0 quando vazio).
    #[must_use]
    pub fn ratio(self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            f64::from(self.done) / f64::from(self.total)
        }
    }

    /// Rótulo canônico `done/total` (ex.: `3/16`).
    #[must_use]
    pub fn label(self) -> String {
        format!("{}/{}", self.done, self.total)
    }
}

/// Épico mais próximo de `id` subindo pelos pais — inclusive se `id` já for épico (D127).
///
/// Limitado à profundidade máxima da hierarquia (`plan ⊃ epic ⊃ issue ⊃ task`).
#[must_use]
pub fn epic_of(graph: &Graph, id: &str) -> Option<String> {
    let mut current = id;
    for _ in 0..4 {
        if graph.scope(current) == Some(Scope::Epic) {
            return Some(current.to_string());
        }
        current = graph.parent(current)?;
    }
    None
}

/// Progresso de conclusão do subárvore de `root` — D127.
///
/// Conta **itens de trabalho folha** (`issue`/`task` sem filhos de trabalho) — a fronteira
/// que o executor realmente pega e conclui. Fechar um `issue` com tarefas abertas não infla
/// o número; esquecer de fechá-lo não o trava. O próprio `root` não entra.
#[must_use]
pub fn progress_of(graph: &Graph, root: &str) -> Progress {
    let mut done: u32 = 0;
    let mut total: u32 = 0;
    let mut stack = vec![root.to_string()];
    while let Some(current) = stack.pop() {
        let children = graph.targets(&current, EdgeKind::ResultsIn);
        for child in children {
            stack.push(child.clone());
        }
        let leaf = children.iter().all(|child| !graph.is_work_item(child));
        if current != root && leaf && graph.is_work_item(&current) {
            total = total.saturating_add(1);
            if graph.status(&current) == Some(Status::Closed) {
                done = done.saturating_add(1);
            }
        }
    }
    Progress::new(done, total)
}

/// Épico de um item + progresso derivado (D127).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpicProgress {
    /// Referência ao épico.
    pub epic: TaskRef,
    /// Progresso de conclusão do épico.
    pub progress: Progress,
}
