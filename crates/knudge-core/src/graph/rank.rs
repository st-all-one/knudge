//! `PageRank` e `Personalized PageRank` sobre as arestas de autoridade (E19-T04/D192).
//!
//! Autoridade é um sinal **derivado** do grafo (D49): a fonte da verdade continua sendo
//! `notas/<id>.md`. A iteração de potência é **determinística** (ordem canônica dos ids, `f64`,
//! tolerância fixa) e o resultado é normalizado (soma 1). Serve ao canal `ppr` do `recall`,
//! semeado pelo *working set*; sem sementes, degenera no `PageRank` global.

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::EdgeKind;

use super::Graph;

/// Arestas que carregam autoridade (D192).
pub const AUTHORITY_EDGES: [EdgeKind; 4] = [
    EdgeKind::References,
    EdgeKind::Supports,
    EdgeKind::Extends,
    EdgeKind::Replaces,
];

/// Amortecimento padrão do `PageRank`.
pub const DAMPING: f64 = 0.85;
/// Iterações máximas da potência (determinístico).
pub const MAX_ITERATIONS: u32 = 32;
/// Tolerância L1 de convergência.
pub const TOLERANCE: f64 = 1e-8;

/// `PageRank` global (semente uniforme sobre todos os nós).
#[must_use]
pub fn pagerank(graph: &Graph) -> BTreeMap<String, f64> {
    personalized_pagerank(graph, &BTreeSet::new())
}

/// `Personalized PageRank`: a massa se concentra na vizinhança das `seeds` (D192).
///
/// `seeds` vazio ⇒ semente uniforme (`PageRank` global). Ids de `seeds` ausentes do grafo são
/// ignorados; se nenhum sobrar, cai no uniforme.
#[must_use]
pub fn personalized_pagerank(graph: &Graph, seeds: &BTreeSet<String>) -> BTreeMap<String, f64> {
    let ids: Vec<&str> = graph.nodes.keys().map(String::as_str).collect();
    if ids.is_empty() {
        return BTreeMap::new();
    }
    let index: BTreeMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(position, id)| (*id, position))
        .collect();
    let out = adjacency(graph, &ids, &index);
    let personal = personalization(&ids, &index, seeds);
    let rank = power_iteration(&out, &personal);
    ids.iter()
        .enumerate()
        .map(|(position, id)| {
            (
                (*id).to_string(),
                rank.get(position).copied().unwrap_or(0.0),
            )
        })
        .collect()
}

/// Adjacência de saída (arestas de autoridade, alvos presentes, sem duplicatas).
fn adjacency(graph: &Graph, ids: &[&str], index: &BTreeMap<&str, usize>) -> Vec<Vec<usize>> {
    ids.iter()
        .map(|id| {
            let mut targets: BTreeSet<usize> = BTreeSet::new();
            if let Some(node) = graph.nodes.get(*id) {
                for kind in AUTHORITY_EDGES {
                    if let Some(list) = node.edges.get(&kind) {
                        for target in list {
                            if let Some(position) = index.get(target.as_str()) {
                                let _ignored = targets.insert(*position);
                            }
                        }
                    }
                }
            }
            targets.into_iter().collect()
        })
        .collect()
}

/// Vetor de personalização (distribuição de probabilidade, soma 1).
fn personalization(
    ids: &[&str],
    index: &BTreeMap<&str, usize>,
    seeds: &BTreeSet<String>,
) -> Vec<f64> {
    let mut personal = vec![0.0_f64; ids.len()];
    let mut selected: Vec<usize> = Vec::new();
    for seed in seeds {
        if let Some(position) = index.get(seed.as_str()) {
            selected.push(*position);
        }
    }
    if selected.is_empty() {
        personal.fill(1.0 / count(ids.len()));
    } else {
        let share = 1.0 / count(selected.len());
        for position in selected {
            if let Some(slot) = personal.get_mut(position) {
                *slot = share;
            }
        }
    }
    personal
}

/// Iteração de potência com redistribuição de *dangling* pela personalização.
#[must_use]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "iteração de potência sobre f64 normalizado (domínio [0,1]); sem overflow"
)]
pub(crate) fn power_iteration(out: &[Vec<usize>], personal: &[f64]) -> Vec<f64> {
    let n = out.len();
    if n == 0 {
        return Vec::new();
    }
    let mut rank = vec![1.0 / count(n); n];
    for _ in 0..MAX_ITERATIONS {
        let mut next: Vec<f64> = personal.iter().map(|p| (1.0 - DAMPING) * p).collect();
        let mut dangling = 0.0_f64;
        for (position, targets) in out.iter().enumerate() {
            let current = rank.get(position).copied().unwrap_or(0.0);
            if targets.is_empty() {
                dangling += current;
                continue;
            }
            let share = DAMPING * current / count(targets.len());
            for &target in targets {
                if let Some(slot) = next.get_mut(target) {
                    *slot += share;
                }
            }
        }
        if dangling > 0.0 {
            let carried = DAMPING * dangling;
            for (position, slot) in next.iter_mut().enumerate() {
                let base = personal.get(position).copied().unwrap_or(0.0);
                *slot = carried.mul_add(base, *slot);
            }
        }
        let delta: f64 = rank
            .iter()
            .zip(next.iter())
            .map(|(before, after)| (after - before).abs())
            .sum();
        rank = next;
        if delta < TOLERANCE {
            break;
        }
    }
    rank
}

/// `usize` → `f64` (sem `as`; a contagem é limitada pelo corpus).
fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}
