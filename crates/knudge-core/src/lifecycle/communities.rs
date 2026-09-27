//! Comunidades (`GraphRAG`) sobre arestas explícitas + âncoras compartilhadas (E19-T05/D193).
//!
//! A detecção roda sobre um grafo ponderado não-dirigido: cada aresta explícita pesa `1,0` e
//! cada âncora compartilhada liga os membros (clique; estrela acima de [`MAX_ANCHOR_CLIQUE`]).
//! Cada comunidade ganha um **resumo local** (termos mais frequentes) para o `kd map`.
//! Off-path: só roda no `kd map` (nunca no `ask`).

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::graph::communities::{WeightedGraph, louvain};
use crate::retrieval::token::content_terms;
use crate::retrieval::{Filter, Index, NoteDoc};
use crate::schema::EdgeKind;

/// Arestas que entram na comunidade (todas as relações explícitas).
pub const COMMUNITY_EDGES: [EdgeKind; 12] = EdgeKind::ALL;
/// Acima deste nº de membros, uma âncora vira estrela em vez de clique (perf O(k²) → O(k)).
pub const MAX_ANCHOR_CLIQUE: usize = 64;
/// Nº de termos do resumo de uma comunidade.
pub const SUMMARY_TERMS: usize = 8;
/// Peso de cada aresta explícita.
const EDGE_WEIGHT: f64 = 1.0;
/// Peso de cada aresta de âncora compartilhada.
const ANCHOR_WEIGHT: f64 = 1.0;

/// Comunidade: membros ordenados + resumo (termos mais frequentes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Community {
    /// Ids membros, ordenados.
    pub members: Vec<String>,
    /// Termos mais frequentes do resumo local (até [`SUMMARY_TERMS`]).
    pub terms: Vec<String>,
}

/// Detecta comunidades sobre o corpus (arestas explícitas + âncoras compartilhadas).
#[must_use]
pub fn communities(index: &Index, graph: &Graph) -> Vec<Community> {
    communities_filtered(index, graph, &Filter::new())
}

/// Como [`communities`], restringindo o corpus ao filtro dado (D143).
///
/// O filtro roda **antes** de montar o grafo: uma nota fora do escopo não entra na partição.
#[must_use]
pub fn communities_filtered(index: &Index, graph: &Graph, filter: &Filter) -> Vec<Community> {
    let docs: Vec<&NoteDoc> = index
        .docs
        .iter()
        .filter(|doc| filter.matches(&doc.meta))
        .collect();
    if docs.is_empty() {
        return Vec::new();
    }
    let allowed: BTreeSet<&str> = docs.iter().map(|doc| doc.meta.id.as_str()).collect();
    let statements: BTreeMap<&str, &str> = docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc.statement.as_str()))
        .collect();
    let mut weighted = WeightedGraph::new(allowed.iter().copied());
    for &id in &allowed {
        for kind in COMMUNITY_EDGES {
            for target in graph.targets(id, kind) {
                if allowed.contains(target.as_str()) {
                    weighted.add_edge(id, target, EDGE_WEIGHT);
                }
            }
        }
    }
    add_anchor_edges(&docs, &allowed, &mut weighted);
    louvain(&weighted)
        .into_iter()
        .map(|members| {
            let terms = top_terms(
                members
                    .iter()
                    .filter_map(|id| statements.get(id.as_str()).copied()),
            );
            Community { members, terms }
        })
        .collect()
}

/// Resumo **global**: termos mais frequentes de todo o corpus filtrado.
#[must_use]
pub fn global_terms(index: &Index, filter: &Filter) -> Vec<String> {
    top_terms(
        index
            .docs
            .iter()
            .filter(|doc| filter.matches(&doc.meta))
            .map(|doc| doc.statement.as_str()),
    )
}

/// Liga os membros de cada âncora compartilhada (clique ou estrela — determinístico).
fn add_anchor_edges(docs: &[&NoteDoc], allowed: &BTreeSet<&str>, weighted: &mut WeightedGraph) {
    let mut by_anchor: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for doc in docs {
        for anchor in &doc.meta.anchors {
            by_anchor
                .entry(anchor.as_str())
                .or_default()
                .push(doc.meta.id.as_str());
        }
    }
    for (_anchor, mut members) in by_anchor {
        members.retain(|id| allowed.contains(id));
        members.sort_unstable();
        members.dedup();
        if members.len() < 2 {
            continue;
        }
        if members.len() <= MAX_ANCHOR_CLIQUE {
            for (i, &a) in members.iter().enumerate() {
                for &b in members.iter().skip(i.saturating_add(1)) {
                    weighted.add_edge(a, b, ANCHOR_WEIGHT);
                }
            }
        } else if let Some((&hub, rest)) = members.split_first() {
            for &member in rest {
                weighted.add_edge(hub, member, ANCHOR_WEIGHT);
            }
        }
    }
}

/// Termos mais frequentes (count desc, termo asc), até [`SUMMARY_TERMS`].
fn top_terms<'a>(texts: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for text in texts {
        for term in content_terms(text) {
            let entry = counts.entry(term.into_owned()).or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
    let mut ranked: Vec<(String, u32)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(SUMMARY_TERMS)
        .map(|(term, _)| term)
        .collect()
}
