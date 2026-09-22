//! Supersessão e demolição com proteção de ciclos (D45, E10-T04).
//!
//! A demolição por decay/shelf-life **nunca** derruba um membro de ciclo de supersessão ou de
//! dependência (D45): o ciclo indica conhecimento ainda vivo, então os membros são **protegidos**.
//! Fora do ciclo, a demolição segue o caminho normal (`supersede`/`forget`).

use std::collections::BTreeSet;

use crate::Result;
use crate::graph::Graph;
use crate::write::{WriteContext, forget};

/// Membros de todos os ciclos (supersessão + dependência), protegidos contra demolição.
#[must_use]
pub fn cycle_members(graph: &Graph) -> BTreeSet<String> {
    graph.cycle_members()
}

/// `true` se o id está protegido por participar de um ciclo.
#[must_use]
pub fn protected(graph: &Graph, id: &str) -> bool {
    graph.cycle_members().contains(id)
}

/// Remove de `ids` os protegidos por ciclo, preservando a ordem.
#[must_use]
pub fn filter_protected(graph: &Graph, ids: &[String]) -> Vec<String> {
    let members = graph.cycle_members();
    ids.iter()
        .filter(|id| !members.contains(*id))
        .cloned()
        .collect()
}

/// Demove (soft-archive via `forget`) uma nota, respeitando a proteção de ciclo.
///
/// Devolve `None` quando o id é protegido (nada muda); `Some(revision)` ao demover.
///
/// # Errors
/// Propaga erros de leitura/transição/escrita; `ErrorKind::InvalidInput` se já estiver `forgotten`.
pub fn demote(
    ctx: &WriteContext<'_>,
    graph: &Graph,
    id: &str,
    reason: &str,
) -> Result<Option<u32>> {
    if protected(graph, id) {
        return Ok(None);
    }
    forget(ctx, id, Some(reason)).map(Some)
}
