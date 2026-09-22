//! Testes das métricas de avaliação (E11-T07).

use crate::embeddings::{GoldenCase, Winner, ab_compare, evaluate, mrr, ndcg_at_k, recall_at_k};

fn ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

#[test]
fn recall_at_k_counts_relevant() {
    let ranked = ids(&["a", "b", "c"]);
    let relevant = ids(&["b", "d"]);
    assert!(close(recall_at_k(&ranked, &relevant, 3), 0.5));
    assert!(close(recall_at_k(&ranked, &relevant, 1), 0.0));
    assert!(close(recall_at_k(&ranked, &[], 3), 0.0));
}

#[test]
fn mrr_uses_first_relevant_rank() {
    let ranked = ids(&["a", "b", "c"]);
    assert!(close(mrr(&ranked, &ids(&["b"])), 0.5));
    assert!(close(mrr(&ranked, &ids(&["c"])), 1.0 / 3.0));
    assert!(close(mrr(&ranked, &ids(&["z"])), 0.0));
}

#[test]
fn ndcg_is_one_for_perfect_ranking() {
    let case = GoldenCase::new("q".to_string(), ids(&["a", "b"]));
    let ranked = ids(&["a", "b", "c"]);
    assert!((ndcg_at_k(&ranked, &case, 3) - 1.0).abs() <= 1e-9);
}

#[test]
fn ndcg_rewards_higher_gain_first() {
    let case = GoldenCase::new("q".to_string(), ids(&["a", "b"]))
        .with_gain("a", 3.0)
        .with_gain("b", 1.0);
    let good = ndcg_at_k(&ids(&["a", "b"]), &case, 2);
    let bad = ndcg_at_k(&ids(&["b", "a"]), &case, 2);
    assert!(good > bad);
}

#[test]
fn evaluate_averages_over_cases() {
    let cases = vec![
        GoldenCase::new("a".to_string(), ids(&["x"])),
        GoldenCase::new("b".to_string(), ids(&["y"])),
    ];
    let metrics = evaluate(&cases, 3, |query| match query {
        "a" => ids(&["x"]),
        _ => ids(&["z"]),
    });
    assert_eq!(metrics.cases, 2);
    assert!(close(metrics.recall, 0.5));
    assert!(close(metrics.mrr, 0.5));
}

#[test]
fn ab_compare_picks_best_ndcg() {
    let cases = vec![GoldenCase::new("a".to_string(), ids(&["x"]))];
    let a = evaluate(&cases, 1, |_| ids(&["z"]));
    let b = evaluate(&cases, 1, |_| ids(&["x"]));
    let report = ab_compare(a.clone(), b);
    assert_eq!(report.winner, Winner::B);
    let tie = ab_compare(a.clone(), a);
    assert_eq!(tie.winner, Winner::Tie);
}
