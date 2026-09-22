//! Testes dos clusters semânticos off-path (E10-T07).

use crate::lifecycle::clusters::{Cluster, ClusterAxis};
use crate::lifecycle::semantic::{cluster_by_similarity, semantic_phase2, should_run};
use crate::schema::NoteType;

#[test]
fn should_run_respects_volume_gate() {
    assert!(!should_run(9, 10));
    assert!(should_run(10, 10));
    assert!(should_run(11, 10));
}

#[test]
fn greedy_clustering_uses_injected_similarity() {
    let ids: Vec<String> = ["a", "b", "c", "d"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let similarity = |left: &str, right: &str| {
        let group = |name: &str| match name {
            "a" | "b" => 0,
            _ => 1,
        };
        if group(left) == group(right) {
            0.9
        } else {
            0.1
        }
    };
    let clusters = cluster_by_similarity(&ids, 0.8, similarity);
    assert_eq!(clusters.len(), 2);
    assert_eq!(clusters.first().map(Vec::len), Some(2));
    assert_eq!(clusters.get(1).map(Vec::len), Some(2));
}

#[test]
fn phase2_only_runs_above_volume() {
    let small = Cluster {
        axis: ClusterAxis::NoteType(NoteType::Fact),
        members: vec!["a".to_string(), "b".to_string()],
    };
    let big = Cluster {
        axis: ClusterAxis::NoteType(NoteType::Decision),
        members: vec!["c".to_string(), "d".to_string(), "e".to_string()],
    };
    let similarity = |_left: &str, _right: &str| 1.0;
    let out = semantic_phase2(&[small, big], 3, 0.8, similarity);
    assert_eq!(out.len(), 1);
    assert_eq!(out.first().map(Vec::len), Some(3));
}
