//! Fusão RRF (D81/D123).

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::retrieval::rrf::{Channel, fuse};

#[test]
fn fuses_by_reciprocal_rank() {
    let first = vec!["x".to_string(), "y".to_string()];
    let second = vec!["y".to_string(), "z".to_string()];
    let fused = fuse(&[Channel::uniform(&first), Channel::uniform(&second)], 60);
    let ids: Vec<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, ["y", "x", "z"]);
    assert_eq!(fused.first().map(|hit| hit.channels), Some(2));
}

#[test]
fn ties_break_by_id() {
    let first = vec!["b".to_string()];
    let second = vec!["a".to_string()];
    let fused = fuse(&[Channel::uniform(&first), Channel::uniform(&second)], 60);
    let ids: Vec<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
}

#[test]
fn union_is_preserved() {
    let first = vec!["x".to_string(), "y".to_string()];
    let second = vec!["z".to_string()];
    let fused = fuse(&[Channel::uniform(&first), Channel::uniform(&second)], 60);
    let ids: BTreeSet<&str> = fused.iter().map(|hit| hit.id.as_str()).collect();
    assert_eq!(ids, BTreeSet::from(["x", "y", "z"]));
}

#[test]
fn empty_channels_are_fine() {
    assert!(fuse(&[], 60).is_empty());
    let empty: Vec<String> = Vec::new();
    assert!(fuse(&[Channel::uniform(&empty)], 60).is_empty());
}

#[test]
fn weight_scales_channel_contribution() {
    let ids = vec!["a".to_string()];
    let one = fuse(&[Channel::new(&ids, 1.0)], 60);
    let two = fuse(&[Channel::new(&ids, 2.0)], 60);
    let single = one.first().map(|hit| hit.score).unwrap_or_default();
    let doubled = two.first().map(|hit| hit.score).unwrap_or_default();
    assert!((doubled - single * 2.0).abs() < f64::EPSILON);
}

/// As parcelas por canal somam exatamente o score fundido (D151).
#[test]
fn contributions_sum_to_score() {
    let first = vec!["x".to_string(), "y".to_string()];
    let second = vec!["y".to_string(), "z".to_string()];
    let fused = fuse(&[Channel::new(&first, 1.0), Channel::new(&second, 2.0)], 60);
    for hit in &fused {
        let sum: f64 = hit.contribs.iter().sum();
        assert!(
            (sum - hit.score).abs() < 1e-12,
            "parcelas != score: {sum} != {}",
            hit.score
        );
    }
    assert!(
        fused.iter().all(|hit| hit.contribs.len() == 2),
        "parcelas por canal ausentes"
    );
}

#[test]
fn semantic_weight_can_flip_the_winner() {
    let lexical = vec!["lex".to_string()];
    let semantic = vec!["sem".to_string()];
    let neutral = fuse(
        &[Channel::uniform(&lexical), Channel::uniform(&semantic)],
        60,
    );
    // Empate neutro → desempate determinístico por id (`lex` < `sem`).
    assert_eq!(neutral.first().map(|hit| hit.id.as_str()), Some("lex"));
    // Peso maior no canal vetorial inverte o vencedor.
    let weighted = fuse(
        &[Channel::new(&lexical, 1.0), Channel::new(&semantic, 2.0)],
        60,
    );
    assert_eq!(weighted.first().map(|hit| hit.id.as_str()), Some("sem"));
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
        let refs: Vec<Channel<'_>> = channels
            .iter()
            .map(|channel| Channel::uniform(channel.as_slice()))
            .collect();
        prop_assert_eq!(fuse(&refs, k), fuse(&refs, k));
    }

    #[test]
    fn rank_weight_is_monotonic(
        size in 1_usize..8,
    ) {
        let channel: Vec<String> = (0..size).map(|index| format!("id{index}")).collect();
        let fused = fuse(&[Channel::uniform(&channel)], 60);
        let scores: Vec<f64> = fused.iter().map(|hit| hit.score).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| b.total_cmp(a));
        prop_assert_eq!(scores, sorted);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// A fusão devolve exatamente a **união** dos ids dos canais, sem repetição.
    #[test]
    fn fuse_contains_union_of_ids(
        channels in prop::collection::vec(prop::collection::vec("[a-e][0-9]", 0..6), 0..3),
    ) {
        let refs: Vec<Channel<'_>> = channels
            .iter()
            .map(|channel| Channel::uniform(channel.as_slice()))
            .collect();
        let fused = fuse(&refs, 60);
        let mut expected: BTreeSet<String> = BTreeSet::new();
        for channel in &channels {
            for id in channel {
                expected.insert(id.clone());
            }
        }
        let got: BTreeSet<String> = fused.iter().map(|hit| hit.id.clone()).collect();
        prop_assert_eq!(got, expected);
    }
}
