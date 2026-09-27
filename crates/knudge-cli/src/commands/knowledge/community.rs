//! Seção de comunidades do `knowledge map` (`GraphRAG` — D193).
//!
//! A detecção vive no core (`lifecycle::communities`); aqui só formatamos as linhas de texto e
//! os nós JSON, e restringimos à vizinhança de `--around`.

use std::collections::BTreeSet;

use knudge_core::graph::Graph;
use knudge_core::lifecycle::{Community, communities_filtered};
use knudge_core::retrieval::{Filter, Index};
use serde_json::json;

/// Linhas de texto + nós JSON + entradas de comunidades.
pub(super) type CommunitySection = (Vec<String>, Vec<serde_json::Value>, Vec<Community>);

/// Seção de comunidades: linhas de texto, nós JSON e entradas.
pub(super) fn community_section(
    index: &Index,
    graph: &Graph,
    filter: &Filter,
    allowed: Option<&BTreeSet<String>>,
) -> CommunitySection {
    let mut found = communities_filtered(index, graph, filter);
    if let Some(allowed) = allowed {
        found = restrict(found, allowed);
    }
    let mut lines = Vec::new();
    let mut data = Vec::new();
    for (position, community) in found.iter().enumerate() {
        lines.push(format!(
            "community|{}|{}|{}",
            position.saturating_add(1),
            community.members.len(),
            community.terms.join(", ")
        ));
        data.push(json!({
            "index": position.saturating_add(1),
            "count": community.members.len(),
            "terms": community.terms,
            "members": community.members,
        }));
    }
    (lines, data, found)
}

/// Restringe os membros das comunidades à vizinhança (`--around`) e descarta vazias.
fn restrict(communities: Vec<Community>, allowed: &BTreeSet<String>) -> Vec<Community> {
    communities
        .into_iter()
        .filter_map(|community| {
            let members: Vec<String> = community
                .members
                .into_iter()
                .filter(|member| allowed.contains(member))
                .collect();
            (!members.is_empty()).then_some(Community {
                members,
                terms: community.terms,
            })
        })
        .collect()
}
