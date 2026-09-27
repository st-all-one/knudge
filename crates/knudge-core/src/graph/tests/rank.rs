//! `PageRank` / `Personalized PageRank` (E19-T04/D192).

use std::collections::{BTreeMap, BTreeSet};

use proptest::collection;
use proptest::prelude::{ProptestConfig, prop_assert, proptest};

use crate::Result;
use crate::graph::Graph;
use crate::graph::rank::{pagerank, personalized_pagerank, power_iteration};
use crate::schema::{EdgeKind, NoteType, id};

use super::note;

/// Id de um `fact` sintético.
fn fact(statement: &str) -> String {
    id::note_id(NoteType::Fact, statement)
}

/// Rank de um id (0.0 se ausente).
fn rank_of(ranks: &BTreeMap<String, f64>, id: &str) -> f64 {
    ranks.get(id).copied().unwrap_or(0.0)
}

/// `usize` → `f64` (sem `as`).
fn to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

#[test]
fn pagerank_rewards_the_hub_and_sums_to_one() -> Result<()> {
    let a = fact("A");
    let c = fact("C");
    let notes = vec![
        note(NoteType::Fact, "A", &[(EdgeKind::References, &c)])?,
        note(NoteType::Fact, "B", &[(EdgeKind::References, &c)])?,
        note(NoteType::Fact, "C", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let ranks = pagerank(&graph);
    let sum: f64 = ranks.values().sum();
    assert!((sum - 1.0).abs() < 1e-6, "soma: {sum}");
    assert!(
        rank_of(&ranks, &c) > rank_of(&ranks, &a),
        "C (hub) deveria ser mais autoritativo que A: {ranks:?}"
    );
    Ok(())
}

#[test]
fn isolated_graph_is_uniform() -> Result<()> {
    let notes = vec![
        note(NoteType::Fact, "A", &[])?,
        note(NoteType::Fact, "B", &[])?,
        note(NoteType::Fact, "C", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let ranks = pagerank(&graph);
    for id in graph.ids() {
        assert!(
            (rank_of(&ranks, id) - 1.0 / 3.0).abs() < 1e-9,
            "{id}: {ranks:?}"
        );
    }
    Ok(())
}

#[test]
fn personalized_pagerank_concentrates_on_the_seed_neighborhood() -> Result<()> {
    let a = fact("A");
    let b = fact("B");
    let c = fact("C");
    let notes = vec![
        note(NoteType::Fact, "A", &[(EdgeKind::References, &c)])?,
        note(NoteType::Fact, "B", &[(EdgeKind::References, &c)])?,
        note(NoteType::Fact, "C", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let seeds: BTreeSet<String> = BTreeSet::from([b.clone()]);
    let ranks = personalized_pagerank(&graph, &seeds);
    // Semeado em B, a vizinhança {B, C} domina A.
    assert!(
        rank_of(&ranks, &b) > rank_of(&ranks, &a),
        "seed B deveria dominar A: {ranks:?}"
    );
    assert!(
        rank_of(&ranks, &c) > rank_of(&ranks, &a),
        "vizinhança de B (C) deveria dominar A: {ranks:?}"
    );
    Ok(())
}

#[test]
fn unknown_seed_falls_back_to_uniform() -> Result<()> {
    let notes = vec![
        note(NoteType::Fact, "A", &[])?,
        note(NoteType::Fact, "B", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let seeds: BTreeSet<String> = BTreeSet::from(["fact_00000000".to_string()]);
    let personalized = personalized_pagerank(&graph, &seeds);
    let global = pagerank(&graph);
    for id in graph.ids() {
        assert!((rank_of(&personalized, id) - rank_of(&global, id)).abs() < 1e-12);
    }
    Ok(())
}

#[test]
fn pagerank_is_invariant_under_relabeling() -> Result<()> {
    let a1 = fact("A");
    let b1 = fact("B");
    let c1 = fact("C");
    let left = Graph::from_notes(vec![
        note(NoteType::Fact, "A", &[(EdgeKind::References, &c1)])?,
        note(NoteType::Fact, "B", &[(EdgeKind::References, &c1)])?,
        note(NoteType::Fact, "C", &[])?,
    ])?;
    let a2 = fact("X");
    let b2 = fact("Y");
    let c2 = fact("Z");
    let right = Graph::from_notes(vec![
        note(NoteType::Fact, "X", &[(EdgeKind::References, &c2)])?,
        note(NoteType::Fact, "Y", &[(EdgeKind::References, &c2)])?,
        note(NoteType::Fact, "Z", &[])?,
    ])?;
    let r1 = pagerank(&left);
    let r2 = pagerank(&right);
    for (left_id, right_id) in [(&a1, &a2), (&b1, &b2), (&c1, &c2)] {
        assert!(
            (rank_of(&r1, left_id) - rank_of(&r2, right_id)).abs() < 1e-9,
            "{left_id} vs {right_id}"
        );
    }
    Ok(())
}

#[test]
fn dangling_nodes_keep_the_mass() -> Result<()> {
    // A→B e C isolado (dangling); a soma continua 1.
    let a = fact("A");
    let b = fact("B");
    let notes = vec![
        note(NoteType::Fact, "A", &[(EdgeKind::Supports, &b)])?,
        note(NoteType::Fact, "B", &[])?,
        note(NoteType::Fact, "C", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let ranks = pagerank(&graph);
    let sum: f64 = ranks.values().sum();
    assert!((sum - 1.0).abs() < 1e-6, "soma: {sum}");
    assert!(rank_of(&ranks, &a) > 0.0);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn power_iteration_sums_to_one(
        edges in collection::vec(collection::vec(0_usize..8, 0..4), 1..8)
    ) {
        let n = edges.len();
        let out: Vec<Vec<usize>> = edges
            .iter()
            .map(|targets| {
                let mut set: BTreeSet<usize> = BTreeSet::new();
                for &target in targets {
                    if target < n {
                        let _ignored = set.insert(target);
                    }
                }
                set.into_iter().collect()
            })
            .collect();
        let personal = vec![1.0 / to_f64(n); n];
        let ranks = power_iteration(&out, &personal);
        let sum: f64 = ranks.iter().sum();
        prop_assert!((sum - 1.0).abs() < 1e-6, "soma {sum}");
        prop_assert!(ranks.iter().all(|rank| rank.is_finite() && *rank >= 0.0));
    }

    #[test]
    fn power_iteration_is_deterministic(
        edges in collection::vec(collection::vec(0_usize..8, 0..4), 1..8)
    ) {
        let n = edges.len();
        let out: Vec<Vec<usize>> = edges
            .iter()
            .map(|targets| {
                let mut set: BTreeSet<usize> = BTreeSet::new();
                for &target in targets {
                    if target < n {
                        let _ignored = set.insert(target);
                    }
                }
                set.into_iter().collect()
            })
            .collect();
        let personal = vec![1.0 / to_f64(n); n];
        let first = power_iteration(&out, &personal);
        let second = power_iteration(&out, &personal);
        prop_assert!(first.len() == second.len());
        for (left, right) in first.iter().zip(second.iter()) {
            prop_assert!((left - right).abs() < 1e-15);
        }
    }
}
