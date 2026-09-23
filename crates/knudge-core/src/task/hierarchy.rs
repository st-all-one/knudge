//! Hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task` (D53/D93).
//!
//! A profundidade é **intrínseca ao `scope`** (1 a 4); o pai tem sempre o escopo imediatamente
//! externo. `blocks` é 1-based e exige pai. `plan` é a raiz e não tem pai.

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

/// Escopo imediatamente externo esperado para um filho de `scope`.
#[must_use]
pub const fn expected_parent(scope: Scope) -> Option<Scope> {
    scope.parent()
}

/// Escopo imediatamente interno (o filho esperado).
#[must_use]
pub const fn child(scope: Scope) -> Option<Scope> {
    match scope {
        Scope::Plan => Some(Scope::Epic),
        Scope::Epic => Some(Scope::Issue),
        Scope::Issue => Some(Scope::Task),
        Scope::Task => None,
    }
}

/// Valida que `parent_scope` é o pai imediato de `child_scope`.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se a hierarquia for inválida ou o filho for `plan`.
pub fn validate_parent(parent_scope: Scope, child_scope: Scope) -> Result<()> {
    match expected_parent(child_scope) {
        Some(expected) if expected == parent_scope => Ok(()),
        Some(expected) => Err(Error::schema(format!(
            "hierarquia inválida: `{child_scope}` não pode ser filho de `{parent_scope}` (esperado `{expected}`)"
        ))),
        None => Err(Error::schema("`plan` é a raiz e não tem pai (D53)")),
    }
}

/// Filhos diretos de um container/tarefa (aresta `results_in`).
#[must_use]
pub fn children(graph: &Graph, parent: &str) -> Vec<String> {
    graph.targets(parent, EdgeKind::ResultsIn).to_vec()
}
