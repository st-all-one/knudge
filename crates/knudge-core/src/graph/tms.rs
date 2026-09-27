//! TMS/ATMS e derrota de crenças (E19-T10/D208).
//!
//! Um **Truth Maintenance System** rastreia de quais **premissas** cada crença depende. Quando
//! uma premissa é **retratada** (status `forgotten`/`superseded`, ou derrota explícita por
//! `replaces`), os dependentes que só se sustentam nela caem. Aqui a retratação é **derivada**:
//! nada é gravado — a crença continua existindo (não se apaga, D14), mas é sinalizada como
//! derrotada para revisão (`prune`/`doctor`).
//!
//! A clausura usa o índice reverso de `depends_on` (alvo → dependentes), em ordem canônica
//! (`BTreeSet`), determinística (D92). A **derrota** (defeasible) também considera
//! `contradicts` + `replaces`: o lado substituído/contradito é derrotado.

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::schema::{EdgeKind, Status};

/// Ids de notas **retratadas** por status (`forgotten`/`superseded`).
#[must_use]
pub fn retracted(graph: &Graph) -> BTreeSet<String> {
    graph
        .ids()
        .into_iter()
        .filter(|id| {
            matches!(
                graph.status(id),
                Some(Status::Forgotten | Status::Superseded)
            )
        })
        .map(str::to_string)
        .collect()
}

/// Dependentes transitivos (via `depends_on`) de premissas retratadas.
///
/// Não inclui as próprias premissas retratadas. Determinístico e O(V+E).
#[must_use]
pub fn defeated_dependents(graph: &Graph, retracted: &BTreeSet<String>) -> BTreeSet<String> {
    let dependents = reverse_depends_on(graph);
    let mut defeated = BTreeSet::new();
    let mut stack: Vec<String> = retracted.iter().cloned().collect();
    while let Some(current) = stack.pop() {
        let Some(next) = dependents.get(&current) else {
            continue;
        };
        for dependent in next {
            if !retracted.contains(dependent) && defeated.insert(dependent.clone()) {
                stack.push(dependent.clone());
            }
        }
    }
    defeated
}

/// Ids derrotados por **substituição**: alvo de `replaces` (derrota explícita).
///
/// A nota substituta vence sem apagar a antiga (D14); a antiga é sinalizada como derrotada.
#[must_use]
pub fn defeated_by_replacement(graph: &Graph) -> BTreeSet<String> {
    let mut losers = BTreeSet::new();
    for id in graph.ids() {
        for target in graph.targets(id, EdgeKind::Replaces) {
            losers.insert(target.clone());
        }
    }
    losers
}

/// Índice reverso de `depends_on` (alvo → dependentes que dependem dele).
fn reverse_depends_on(graph: &Graph) -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for id in graph.ids() {
        for target in graph.targets(id, EdgeKind::DependsOn) {
            map.entry(target.clone())
                .or_default()
                .insert(id.to_string());
        }
    }
    map
}
