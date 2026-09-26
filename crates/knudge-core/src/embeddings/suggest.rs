//! Sugestões semânticas de aresta/contradição (D158).
//!
//! Read-only: nada vira aresta. Pares são classificados em `duplicate`/`contradiction`/`link`
//! a partir do índice vetorial e do grafo (D49/D50).

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;

use super::index::EmbeddingIndex;
use super::semantic::{linked, neighbors};

/// Relação sugerida entre duas notas (D158).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relation {
    /// Quase-duplicata (candidata a merge/supersede).
    Duplicate,
    /// Mesmo tópico, possível contradição (advisory).
    Contradiction,
    /// Relacionadas, sem aresta (candidata a `references`/`extends`).
    Link,
}

impl Relation {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Contradiction => "contradiction",
            Self::Link => "link",
        }
    }
}

/// Limiares da classificação semântica (D158).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuggestionPolicy {
    /// Similaridade de quase-duplicata (merge).
    pub duplicate: f64,
    /// Piso da banda advisory (contradição/link).
    pub low: f64,
    /// Teto da banda advisory.
    pub high: f64,
}

/// Sugestão semântica de aresta (proposta — nunca vira aresta sozinha, D49).
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSuggestion {
    /// Origem (menor id na ordem canônica).
    pub from: String,
    /// Destino.
    pub to: String,
    /// Relação sugerida.
    pub relation: Relation,
    /// Similaridade `[0,1]`.
    pub score: f64,
}

/// Contexto de um par para a classificação (D158): sem `bool` na assinatura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    /// Já existe aresta (em qualquer direção).
    Linked,
    /// Não existe aresta.
    Unlinked,
}

/// Estado de compartilhamento de âncoras entre as notas do par (D158).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorShare {
    /// Compartilham ao menos uma âncora.
    Shared,
    /// Não compartilham âncora.
    Disjoint,
}

/// Classifica um par (puro) a partir da similaridade e do contexto (D158).
///
/// Par já ligado nunca sugere. Par `Shared` na banda advisory tende a `Link` (relacionadas);
/// `Disjoint` tende a `Contradiction` (mesmo tópico, sem âncora comum).
#[must_use]
pub fn classify_pair(
    score: f64,
    edge: EdgeState,
    anchor: AnchorShare,
    policy: &SuggestionPolicy,
) -> Option<Relation> {
    if edge == EdgeState::Linked {
        return None;
    }
    if score >= policy.duplicate {
        return Some(Relation::Duplicate);
    }
    if score >= policy.high {
        return Some(Relation::Link);
    }
    if score >= policy.low {
        return Some(if anchor == AnchorShare::Shared {
            Relation::Link
        } else {
            Relation::Contradiction
        });
    }
    None
}

/// Sugestões semânticas determinísticas sobre o índice vetorial (D158).
///
/// `anchors_of` mapeia id → âncoras declaradas. Pares são deduplicados (`from < to`) e
/// ordenados por `(score desc, from asc, to asc)`.
#[must_use]
pub fn semantic_suggestions(
    index: &EmbeddingIndex,
    graph: &Graph,
    anchors_of: &BTreeMap<String, Vec<String>>,
    policy: &SuggestionPolicy,
    top_k: usize,
) -> Vec<SemanticSuggestion> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut out = Vec::new();
    for id in index.ids() {
        for neighbor in neighbors(index, id, top_k, policy.low) {
            let (from, to) = if id <= neighbor.id.as_str() {
                (id.to_string(), neighbor.id.clone())
            } else {
                (neighbor.id.clone(), id.to_string())
            };
            if !seen.insert((from.clone(), to.clone())) {
                continue;
            }
            let shares_anchor = if shares_any(anchors_of.get(&from), anchors_of.get(&to)) {
                AnchorShare::Shared
            } else {
                AnchorShare::Disjoint
            };
            let edge = if linked(graph, &from, &to) {
                EdgeState::Linked
            } else {
                EdgeState::Unlinked
            };
            if let Some(relation) = classify_pair(neighbor.score, edge, shares_anchor, policy) {
                out.push(SemanticSuggestion {
                    from,
                    to,
                    relation,
                    score: neighbor.score,
                });
            }
        }
    }
    out.sort_unstable_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.from.cmp(&right.from))
            .then_with(|| left.to.cmp(&right.to))
    });
    out
}

fn shares_any(left: Option<&Vec<String>>, right: Option<&Vec<String>>) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };
    left.iter().any(|anchor| right.contains(anchor))
}
