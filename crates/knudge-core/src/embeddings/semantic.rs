//! Consultas semânticas sobre o índice vetorial: vizinhos, duplicatas e links (E11-T09).
//!
//! Tudo é **proposta**: nada é fundido nem ligado sem aceite (D42/D47). As funções são puras e
//! determinísticas (ordenação `score desc, id asc`), consumíveis por `audit`/`compact`/`learn`.

use std::collections::BTreeSet;

use crate::graph::Graph;
use crate::lifecycle::semantic::cluster_by_similarity;
use crate::schema::EdgeKind;

use super::index::EmbeddingIndex;
use super::vector::similarity;

/// Vizinho por similaridade.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    /// Id do vizinho.
    pub id: String,
    /// Similaridade `[0,1]` (cosseno) com a nota de referência.
    pub score: f64,
}

/// Par quase-duplicado (proposta de merge/supersede).
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicatePair {
    /// Primeiro id (menor na ordem canônica).
    pub a: String,
    /// Segundo id.
    pub b: String,
    /// Similaridade `[0,1]`.
    pub score: f64,
}

/// Sugestão de link não-declarado (proposta de aresta).
#[derive(Debug, Clone, PartialEq)]
pub struct LinkSuggestion {
    /// Origem.
    pub from: String,
    /// Destino.
    pub to: String,
    /// Similaridade `[0,1]`.
    pub score: f64,
}

/// Vizinhos de `id` acima de `min_score`, em ordem determinística.
#[must_use]
pub fn neighbors(index: &EmbeddingIndex, id: &str, top_k: usize, min_score: f64) -> Vec<Neighbor> {
    let Some(origin) = index.vector(id) else {
        return Vec::new();
    };
    let metric = index.meta.similarity;
    let mut found: Vec<Neighbor> = index
        .ids()
        .into_iter()
        .filter(|other| *other != id)
        .filter_map(|other| {
            let vector = index.vector(other)?;
            let score = f64::from(similarity(origin, vector, metric));
            (score >= min_score).then(|| Neighbor {
                id: other.to_string(),
                score,
            })
        })
        .collect();
    found.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    if top_k > 0 {
        found.truncate(top_k);
    }
    found
}

/// Pares de notas acima de `threshold` (candidatos a merge/supersede).
#[must_use]
pub fn duplicate_pairs(index: &EmbeddingIndex, threshold: f64) -> Vec<DuplicatePair> {
    let ids = index.ids();
    let metric = index.meta.similarity;
    let mut pairs = Vec::new();
    for (position, a) in ids.iter().enumerate() {
        let Some(va) = index.vector(a) else {
            continue;
        };
        for b in ids.iter().skip(position.saturating_add(1)) {
            let Some(vb) = index.vector(b) else {
                continue;
            };
            let score = f64::from(similarity(va, vb, metric));
            if score >= threshold {
                pairs.push(DuplicatePair {
                    a: (*a).to_string(),
                    b: (*b).to_string(),
                    score,
                });
            }
        }
    }
    pairs.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.a.cmp(&right.a))
            .then_with(|| left.b.cmp(&right.b))
    });
    pairs
}

/// Sugere arestas para vizinhos ainda não ligados (proposta — nada é escrito).
#[must_use]
pub fn link_suggestions(
    index: &EmbeddingIndex,
    graph: &Graph,
    threshold: f64,
    top_k: usize,
) -> Vec<LinkSuggestion> {
    let mut suggestions = Vec::new();
    for id in index.ids() {
        for neighbor in neighbors(index, id, top_k, threshold) {
            if linked(graph, id, &neighbor.id) {
                continue;
            }
            suggestions.push(LinkSuggestion {
                from: id.to_string(),
                to: neighbor.id,
                score: neighbor.score,
            });
        }
    }
    suggestions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.from.cmp(&right.from))
            .then_with(|| left.to.cmp(&right.to))
    });
    suggestions
}

/// Agrupa `ids` por similaridade (fase 2 do `clusters`, D47/E10-T07).
#[must_use]
pub fn clusters(index: &EmbeddingIndex, ids: &[String], threshold: f64) -> Vec<Vec<String>> {
    let metric = index.meta.similarity;
    cluster_by_similarity(ids, threshold, |a, b| {
        match (index.vector(a), index.vector(b)) {
            (Some(va), Some(vb)) => f64::from(similarity(va, vb, metric)),
            _ => 0.0,
        }
    })
}

fn linked(graph: &Graph, from: &str, to: &str) -> bool {
    has_edge(graph, from, to) || has_edge(graph, to, from)
}

fn has_edge(graph: &Graph, from: &str, to: &str) -> bool {
    EdgeKind::ALL
        .iter()
        .any(|kind| graph.targets(from, *kind).iter().any(|target| target == to))
}

/// Ids indexados que ainda não aparecem em nenhuma aresta (auxiliar de `audit`).
#[must_use]
pub fn unlinked_ids(index: &EmbeddingIndex, graph: &Graph) -> BTreeSet<String> {
    index
        .ids()
        .into_iter()
        .filter(|id| {
            EdgeKind::ALL
                .iter()
                .all(|kind| graph.targets(id, *kind).is_empty())
        })
        .map(str::to_string)
        .collect()
}
