//! Hierarquia `epic ⊃ { issue ⊃ task | task }` (D53/D93/D134).
//!
//! O `epic` é a **raiz**; o `issue` é **opcional**. A validação exige que o pai tenha `rank`
//! estritamente menor que o filho; `blocks` é 1-based e exige pai.

use crate::graph::Graph;
use crate::schema::{EdgeKind, Scope};
use crate::{Error, Result};

/// Valida a ordem `blocks` (1-based).
///
/// # Errors
/// Retorna `ErrorKind::Schema` para `blocks = 0`.
pub fn validate_blocks(blocks: Option<u32>) -> Result<()> {
    if blocks == Some(0) {
        return Err(Error::schema("`blocks` é 1-based (D53)"));
    }
    Ok(())
}

/// Escopo imediatamente interno (o filho padrão): sempre a folha `task` (D134).
#[must_use]
pub const fn child(scope: Scope) -> Option<Scope> {
    match scope {
        Scope::Epic | Scope::Issue => Some(Scope::Task),
        Scope::Task => None,
    }
}

/// Valida que `parent_scope` pode ser pai de `child_scope` (D134).
///
/// O pai precisa ter `rank` **estritamente menor**; `epic` é a raiz e não tem pai.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se a hierarquia for inválida ou o filho for `epic`.
pub fn validate_parent(parent_scope: Scope, child_scope: Scope) -> Result<()> {
    if child_scope == Scope::Epic {
        return Err(Error::schema("`epic` é a raiz e não tem pai (D134)"));
    }
    if parent_scope.rank() < child_scope.rank() {
        Ok(())
    } else {
        Err(Error::schema(format!(
            "hierarquia inválida: `{child_scope}` não pode ser filho de `{parent_scope}` (D134)"
        )))
    }
}

/// Filhos diretos de um container/tarefa (aresta `results_in`).
#[must_use]
pub fn children(graph: &Graph, parent: &str) -> Vec<String> {
    graph.targets(parent, EdgeKind::ResultsIn).to_vec()
}
