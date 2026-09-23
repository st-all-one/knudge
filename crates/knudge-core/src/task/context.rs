//! Contexto estrutural de um item de trabalho — visibilidade em um só comando (D125).
//!
//! Resolve ids em referências legíveis (`id|statement`) para que um único `kd task show`
//! responda "onde isto se encaixa, o que o bloqueia e o que ele destrava" — sem puxar a
//! árvore inteira. Relações: pai (marcador do corpo), `depends_on` (saída = bloqueadores,
//! entrada = bloqueados) e filhos (`results_in`).

use crate::graph::Graph;
use crate::schema::{EdgeKind, Scope, Status};
use crate::store::Store;
use crate::{ErrorKind, Result};

use super::progress::{EpicProgress, epic_of, progress_of};

/// Referência resumida a um item de trabalho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRef {
    /// Id.
    pub id: String,
    /// Afirmação (vazia se a nota já não existe — aresta pendente).
    pub statement: String,
    /// Escopo, quando houver.
    pub scope: Option<Scope>,
    /// Estado.
    pub status: Status,
}

/// Contexto estrutural de um item (D125).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskContext {
    /// Pai imediato (marcador do corpo).
    pub parent: Option<TaskRef>,
    /// Dependências (`depends_on` de saída) — o que bloqueia este item.
    pub blocked_by: Vec<TaskRef>,
    /// Quem depende deste item (`depends_on` de entrada) — o que ele destrava.
    pub blocks: Vec<TaskRef>,
    /// Filhos diretos (arestas `results_in`).
    pub children: Vec<TaskRef>,
    /// Épico mais próximo acima + progresso derivado (D127).
    pub epic: Option<EpicProgress>,
}

impl TaskContext {
    /// `true` se não há nenhuma relação a mostrar.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parent.is_none()
            && self.blocked_by.is_empty()
            && self.blocks.is_empty()
            && self.children.is_empty()
            && self.epic.is_none()
    }
}

/// Contexto de `id` no grafo (D125).
///
/// # Errors
/// Propaga erros de leitura do store.
pub fn context_of(store: &Store<'_>, graph: &Graph, id: &str) -> Result<TaskContext> {
    let note = store.read(id)?;
    let parent = super::parent_of(&note)
        .as_deref()
        .map(|parent| reference(store, graph, parent))
        .transpose()?;
    let blocked_by = graph
        .targets(id, EdgeKind::DependsOn)
        .iter()
        .map(|target| reference(store, graph, target))
        .collect::<Result<Vec<_>>>()?;
    let blocks = graph
        .ids()
        .into_iter()
        .filter(|other| *other != id)
        .filter(|other| {
            graph
                .targets(other, EdgeKind::DependsOn)
                .iter()
                .any(|target| target == id)
        })
        .map(|other| reference(store, graph, other))
        .collect::<Result<Vec<_>>>()?;
    let children = graph
        .targets(id, EdgeKind::ResultsIn)
        .iter()
        .map(|child| reference(store, graph, child))
        .collect::<Result<Vec<_>>>()?;
    let epic = epic_of(graph, id)
        .map(|epic_id| {
            Ok(EpicProgress {
                epic: reference(store, graph, &epic_id)?,
                progress: progress_of(graph, &epic_id),
            })
        })
        .transpose()?;
    Ok(TaskContext {
        parent,
        blocked_by,
        blocks,
        children,
        epic,
    })
}

/// Resolve um id em [`TaskRef`]; aresta pendente vira afirmação vazia (R33).
fn reference(store: &Store<'_>, graph: &Graph, id: &str) -> Result<TaskRef> {
    let statement = match store.read(id) {
        Ok(note) => note.frontmatter.statement().unwrap_or_default().to_string(),
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    Ok(TaskRef {
        id: id.to_string(),
        statement,
        scope: graph.scope(id),
        status: graph.status(id).unwrap_or(Status::Active),
    })
}
