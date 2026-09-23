//! Testes dos clusters semânticos off-path (E10-T07).

use std::slice;

use crate::lifecycle::clusters::{Cluster, ClusterAxis};
use crate::lifecycle::semantic::{
    cluster_by_similarity, semantic_clusters, semantic_phase2, should_run,
};
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
fn complete_link_prevents_chaining() {
    let ids: Vec<String> = ["a", "b", "c"].iter().map(|s| (*s).to_string()).collect();
    // `a`~`b` e `b`~`c`, mas `a` e `c` são distantes: com leader/single-link, `c` entraria.
    let similarity = |left: &str, right: &str| match (left, right) {
        ("a" | "c", "b") | ("b", "a" | "c") => 0.9,
        (left, right) if left == right => 1.0,
        _ => 0.2,
    };
    let clusters = cluster_by_similarity(&ids, 0.8, similarity);
    assert_eq!(clusters.len(), 2);
    assert_eq!(clusters.first().map(Vec::len), Some(2));
    assert_eq!(clusters.get(1).map(Vec::len), Some(1));
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

#[test]
fn semantic_clusters_preserve_parent() {
    let cluster = Cluster {
        axis: ClusterAxis::NoteType(NoteType::Fact),
        members: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
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
    let out = semantic_clusters(slice::from_ref(&cluster), 3, 0.8, similarity);
    assert_eq!(out.len(), 1);
    assert_eq!(out.first().map(|entry| entry.parent.clone()), Some(cluster));
    assert_eq!(out.first().map(|entry| entry.groups.len()), Some(2));
}
