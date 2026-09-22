//! Clusters fase 1: estruturais e determinísticos (D47, E10-T06).
//!
//! Agregação barata por `anchor`, `type`, `classification` e container (via `depends_on`
//! transitivo) — sem estatística e sem embeddings. É a base sobre a qual a fase 2 semântica
//! (opcional e off-path) opera.

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::retrieval::Index;
use crate::schema::{Classification, EdgeKind, NoteType};

/// Profundidade máxima da busca pelo container ancestral.
pub const MAX_CONTAINER_DEPTH: u32 = 8;

/// Eixo de agrupamento estrutural.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClusterAxis {
    /// Agrupado por âncora declarada.
    Anchor(String),
    /// Agrupado por tipo de nota.
    NoteType(NoteType),
    /// Agrupado por classificação de maturidade.
    Classification(Classification),
    /// Agrupado por container ancestral.
    Container(String),
}

/// Cluster estrutural: eixo + membros ordenados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cluster {
    /// Eixo de agrupamento.
    pub axis: ClusterAxis,
    /// Ids membros, ordenados.
    pub members: Vec<String>,
}

/// Computa os clusters estruturais de um índice/grafo (determinístico).
#[must_use]
pub fn structural_clusters(index: &Index, graph: &Graph) -> Vec<Cluster> {
    let mut by_axis: BTreeMap<ClusterAxis, BTreeSet<String>> = BTreeMap::new();
    for doc in &index.docs {
        let id = &doc.meta.id;
        for anchor in &doc.meta.anchors {
            by_axis
                .entry(ClusterAxis::Anchor(anchor.clone()))
                .or_default()
                .insert(id.clone());
        }
        by_axis
            .entry(ClusterAxis::NoteType(doc.meta.note_type))
            .or_default()
            .insert(id.clone());
        by_axis
            .entry(ClusterAxis::Classification(doc.meta.classification))
            .or_default()
            .insert(id.clone());
        if let Some(container) = container_of(graph, id) {
            by_axis
                .entry(ClusterAxis::Container(container))
                .or_default()
                .insert(id.clone());
        }
    }
    by_axis
        .into_iter()
        .map(|(axis, members)| Cluster {
            axis,
            members: members.into_iter().collect(),
        })
        .collect()
}

/// Container ancestral mais próximo de `id` (via `depends_on` transitivo), se houver.
#[must_use]
pub fn container_of(graph: &Graph, id: &str) -> Option<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut best: Option<(u32, String)> = None;
    let mut stack: Vec<(String, u32)> = vec![(id.to_string(), 0)];
    while let Some((current, depth)) = stack.pop() {
        if depth >= MAX_CONTAINER_DEPTH {
            continue;
        }
        let mut deps: Vec<String> = graph.targets(&current, EdgeKind::DependsOn).to_vec();
        deps.sort();
        for dep in deps {
            if !visited.insert(dep.clone()) {
                continue;
            }
            let next_depth = depth.saturating_add(1);
            if graph.note_type(&dep) == Some(NoteType::Container) {
                let candidate = (next_depth, dep.clone());
                let replace = match &best {
                    None => true,
                    Some(current) => candidate < *current,
                };
                if replace {
                    best = Some(candidate);
                }
            }
            stack.push((dep, next_depth));
        }
    }
    best.map(|(_, container)| container)
}
