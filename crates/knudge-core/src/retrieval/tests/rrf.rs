//! Fusão RRF (D81).

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::retrieval::rrf::fuse;

#[test]
fn fuses_by_reciprocal_rank() {
    let first = vec!["x".to_string(), "y".to_string()];
    let second = vec!["y".to_string(), "z".to_string()];
    let fused = fuse(&[first.as_slice(), second.as_slice()], 60);
    let ids: Vec<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, ["y", "x", "z"]);
    assert_eq!(fused.first().map(|hit| hit.channels), Some(2));
}

#[test]
fn ties_break_by_id() {
    let first = vec!["b".to_string()];
    let second = vec!["a".to_string()];
    let fused = fuse(&[first.as_slice(), second.as_slice()], 60);
    let ids: Vec<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
}

#[test]
fn union_is_preserved() {
    let first = vec!["x".to_string(), "y".to_string()];
    let second = vec!["z".to_string()];
    let fused = fuse(&[first.as_slice(), second.as_slice()], 60);
    let ids: BTreeSet<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, BTreeSet::from(["x", "y", "z"]));
}

#[test]
fn empty_channels_are_fine() {
    assert!(fuse(&[], 60).is_empty());
    let empty: Vec<String> = Vec::new();
    assert!(fuse(&[empty.as_slice()], 60).is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn fuse_is_deterministic(
        channels in prop::collection::vec(
            prop::collection::vec("[a-e][0-9]", 0..6),
            0..3,
        ),
        k in 0_u32..200,
    ) {
        let refs: Vec<&[String]> = channels.iter().map(Vec::as_slice).collect();
        prop_assert_eq!(fuse(&refs, k), fuse(&refs, k));
    }

    #[test]
    fn rank_weight_is_monotonic(
        size in 1_usize..8,
    ) {
        let channel: Vec<String> = (0..size).map(|index| format!("id{index}")).collect();
        let fused = fuse(&[channel.as_slice()], 60);
        let scores: Vec<f64> = fused.iter().map(|hit| hit.score).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| b.total_cmp(a));
        prop_assert_eq!(scores, sorted);
    }
}
