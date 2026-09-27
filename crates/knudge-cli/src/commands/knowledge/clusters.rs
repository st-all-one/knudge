//! Filtragem dos clusters estruturais do `knowledge map` (D143).
//!
//! Isola a montagem do `CorpusScope` e os filtros de eixo/escopo/`--around`, deixando o
//! `map.rs` só com a orquestração e a renderização.

use std::collections::BTreeSet;

use knudge_core::graph::Graph;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{Cluster, structural_clusters_filtered};
use knudge_core::retrieval::Index;
use knudge_core::{Error, Result};

use crate::cli::KnowledgeMapArgs;

use super::super::corpus::{CorpusScope, Selection};

/// Monta o `CorpusScope` a partir das flags de filtro de `knowledge map`.
pub(super) fn corpus_scope(args: &KnowledgeMapArgs) -> CorpusScope {
    CorpusScope {
        types: args.types.clone(),
        classes: args.classes.clone(),
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
        around: args.around.clone(),
        depth: args.depth,
        universe: args.universe,
    }
}

/// Filtra os clusters estruturais (eixos, escopo e vizinhança de `--around`).
pub(super) fn selected_clusters(
    index: &Index,
    graph: &Graph,
    selection: &Selection<'_>,
    axis: Option<&str>,
    scope: Option<&str>,
) -> Vec<Cluster> {
    let mut clusters = structural_clusters_filtered(index, graph, selection.filter());
    if let Some(allowed) = selection.allowed() {
        clusters = restrict(clusters, allowed);
    }
    if let Some(axis) = axis {
        clusters.retain(|cluster| cluster.axis.axis() == axis);
    }
    if let Some(scope) = scope {
        clusters = scope_clusters(clusters, graph, scope);
    }
    clusters
}

/// Restringe os membros dos clusters à vizinhança (`--around`) e descarta clusters vazios.
fn restrict(clusters: Vec<Cluster>, allowed: &BTreeSet<String>) -> Vec<Cluster> {
    clusters
        .into_iter()
        .filter_map(|cluster| {
            let members: Vec<String> = cluster
                .members
                .into_iter()
                .filter(|member| allowed.contains(member))
                .collect();
            (!members.is_empty()).then_some(Cluster {
                axis: cluster.axis,
                members,
            })
        })
        .collect()
}

/// Valida o nome do eixo.
pub(super) fn validate_axis(axis: &str) -> Result<()> {
    const AXES: [&str; 4] = ["anchor", "type", "classification", "scope"];
    if AXES.contains(&axis) {
        Ok(())
    } else {
        Err(Error::invalid_input(format!(
            "eixo desconhecido: {axis:?} (use anchor|type|classification|scope)"
        )))
    }
}

/// Restringe os membros aos que pertencem ao container dado (`belongs_to`).
fn scope_clusters(clusters: Vec<Cluster>, graph: &Graph, scope: &str) -> Vec<Cluster> {
    clusters
        .into_iter()
        .filter_map(|cluster| {
            let members: Vec<String> = cluster
                .members
                .into_iter()
                .filter(|member| belongs_to(graph, member, scope))
                .collect();
            (!members.is_empty()).then_some(Cluster {
                axis: cluster.axis,
                members,
            })
        })
        .collect()
}
