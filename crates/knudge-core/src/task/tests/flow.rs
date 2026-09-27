//! Métricas de fluxo e caminho crítico (E19/T11/R7/D205).

use std::collections::BTreeMap;

use proptest::prelude::*;

use crate::Result;
use crate::schema::Value;
use crate::store::Event;
use crate::task::flow::{CriticalPath, ThroughputBucket, critical_path, task_flows, throughput};

use super::impact::{graph_with, task_id};

fn task_event(action: &str, at: i64, id: &str) -> Event {
    Event::new("task", at)
        .with_note_id(id)
        .with_data("action", Value::Str(action.to_string()))
}

#[test]
fn task_flows_tracks_create_and_review() {
    let events = vec![
        task_event("submit", 100, "task_a"),
        task_event("review", 300, "task_a"),
        Event::new("write", 50).with_note_id("task_b"),
    ];
    let flows = task_flows(&events);
    let a = flows.get("task_a").copied().unwrap_or_default();
    assert_eq!(a.created_ms, 100);
    assert_eq!(a.closed_ms, Some(300));
    assert_eq!(a.cycle_ms(), Some(200));
    assert_eq!(a.lead_ms(999), 200);
    assert!(a.is_closed());
    let b = flows.get("task_b").copied().unwrap_or_default();
    assert_eq!(b.created_ms, 50);
    assert_eq!(b.cycle_ms(), None);
    assert_eq!(b.lead_ms(150), 100, "tarefa aberta conta o tempo em voo");
}

#[test]
fn throughput_groups_reviews_by_window() {
    let events = vec![
        task_event("review", 10, "a"),
        task_event("review", 20, "b"),
        task_event("submit", 30, "c"),
        task_event("review", 1_000, "c"),
        task_event("review", 1_010, "d"),
    ];
    let buckets = throughput(&events, 1_000);
    assert_eq!(
        buckets,
        vec![
            ThroughputBucket {
                start_ms: 0,
                closed: 2
            },
            ThroughputBucket {
                start_ms: 1_000,
                closed: 2
            },
        ]
    );
}

#[test]
fn critical_path_follows_the_dependency_chain() -> Result<()> {
    // t1 depende de t0, t2 de t1, t3 de t2; durações crescentes.
    let graph = graph_with(&[(1, 0), (2, 1), (3, 2)])?;
    let mut durations: BTreeMap<String, i64> = BTreeMap::new();
    for (index, weight) in [10_i64, 20, 30, 40].iter().enumerate() {
        let _ignored = durations.insert(task_id(index)?, *weight);
    }
    let path = critical_path(&graph, &durations);
    assert_eq!(path.total_ms, 100);
    assert_eq!(path.ids.len(), 4);
    assert_eq!(
        path.ids.first().map(String::as_str),
        Some(task_id(3)?.as_str())
    );
    Ok(())
}

#[test]
fn critical_path_without_weights_is_deterministic() {
    let Ok(graph) = graph_with(&[(1, 0)]) else {
        return;
    };
    let path = critical_path(&graph, &BTreeMap::new());
    assert_eq!(path.total_ms, 0);
    assert_eq!(path.ids.len(), 2, "caminho de 2 nós mesmo com peso zero");
}

/// Maior caminho ponderado por força bruta (arestas `from < to` ⇒ DAG em índice).
fn brute_force_longest(edges: &[(usize, usize)], weights: &[i64]) -> i64 {
    let mut best = [0_i64; 4];
    for from in (0..4).rev() {
        let mut tail = 0_i64;
        for &(edge_from, to) in edges {
            if edge_from == from {
                tail = tail.max(best.get(to).copied().unwrap_or(0));
            }
        }
        let weight = weights.get(from).copied().unwrap_or(0);
        if let Some(slot) = best.get_mut(from) {
            *slot = weight.saturating_add(tail);
        }
    }
    best.into_iter().max().unwrap_or(0)
}

proptest! {
    /// O caminho crítico é o maior caminho ponderado do DAG `depends_on`.
    #[test]
    fn critical_path_is_the_longest_weighted_path(
        raw in prop::collection::vec((0usize..4, 0usize..4), 0..6),
        weights in prop::collection::vec(0_i64..10, 4),
    ) {
        let edges: Vec<(usize, usize)> =
            raw.into_iter().filter(|(from, to)| from < to).collect();
        if let Ok(graph) = graph_with(&edges) {
            let mut durations: BTreeMap<String, i64> = BTreeMap::new();
            for index in 0..4 {
                if let Ok(id) = task_id(index) {
                    let _ignored = durations.insert(id, weights.get(index).copied().unwrap_or(0));
                }
            }
            let path: CriticalPath = critical_path(&graph, &durations);
            prop_assert_eq!(path.total_ms, brute_force_longest(&edges, &weights));
        }
    }
}
