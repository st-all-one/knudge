//! Testes das consultas semânticas (E11-T09).

use crate::Result;
use crate::embeddings::EmbeddingIndex;
use crate::embeddings::meta::{EmbeddingMeta, Similarity};
use crate::embeddings::semantic::{
    clusters, duplicate_pairs, link_suggestions, neighbors, unlinked_ids,
};
use crate::graph::Graph;

use super::note;

struct Fixture {
    index: EmbeddingIndex,
    graph: Graph,
    a: String,
    b: String,
    c: String,
}

fn setup() -> Result<Fixture> {
    let alfa = note("alfa", "")?;
    let beta = note("beta", "")?;
    let gama = note("gama", "")?;
    let a = alfa.id()?.to_string();
    let b = beta.id()?.to_string();
    let c = gama.id()?.to_string();
    let graph = Graph::from_notes(vec![alfa, beta, gama])?;
    let meta = EmbeddingMeta::new("http", "modelo", "1", 2, Similarity::Cosine)?;
    let mut index = EmbeddingIndex::new(meta);
    index.insert(&a, "h", vec![1.0, 0.0]);
    index.insert(&b, "h", vec![0.0, 1.0]);
    index.insert(&c, "h", vec![1.0, 0.0]);
    Ok(Fixture {
        index,
        graph,
        a,
        b,
        c,
    })
}

#[test]
fn neighbors_rank_by_similarity() -> Result<()> {
    let fixture = setup()?;
    let found = neighbors(&fixture.index, &fixture.a, 0, 0.5);
    assert_eq!(
        found.first().map(|n| n.id.as_str()),
        Some(fixture.c.as_str())
    );
    assert!(!found.iter().any(|n| n.id == fixture.b));
    Ok(())
}

#[test]
fn duplicate_pairs_find_identical_vectors() -> Result<()> {
    let fixture = setup()?;
    let pairs = duplicate_pairs(&fixture.index, 0.99);
    assert_eq!(pairs.len(), 1);
    assert!(pairs.first().is_some_and(|pair| {
        let mut got = [pair.a.clone(), pair.b.clone()];
        got.sort();
        let mut want = [fixture.a.clone(), fixture.c.clone()];
        want.sort();
        got == want
    }));
    Ok(())
}

#[test]
fn link_suggestions_are_proposals_only() -> Result<()> {
    let fixture = setup()?;
    let suggestions = link_suggestions(&fixture.index, &fixture.graph, 0.9, 1);
    assert!(
        suggestions
            .iter()
            .any(|s| s.from == fixture.a && s.to == fixture.c)
    );
    assert!(
        suggestions
            .iter()
            .any(|s| s.from == fixture.c && s.to == fixture.a)
    );
    Ok(())
}

#[test]
fn clusters_group_by_similarity() -> Result<()> {
    let fixture = setup()?;
    let ids = vec![fixture.a.clone(), fixture.b.clone(), fixture.c.clone()];
    let grouped = clusters(&fixture.index, &ids, 0.9);
    assert_eq!(grouped.len(), 2);
    assert!(
        grouped
            .iter()
            .any(|cluster| cluster.contains(&fixture.a) && cluster.contains(&fixture.c))
    );
    assert!(
        grouped
            .iter()
            .any(|cluster| cluster == &vec![fixture.b.clone()])
    );
    Ok(())
}

#[test]
fn unlinked_ids_lists_isolated_notes() -> Result<()> {
    let fixture = setup()?;
    let isolated = unlinked_ids(&fixture.index, &fixture.graph);
    assert!(isolated.contains(&fixture.a));
    assert!(isolated.contains(&fixture.b));
    assert!(isolated.contains(&fixture.c));
    Ok(())
}
