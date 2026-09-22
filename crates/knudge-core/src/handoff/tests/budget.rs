//! Orçamento de tokens (E08-T02).

use crate::handoff::budget::{MIN_TAIL, apply, apply_into, estimate_tokens};

#[test]
fn estimate_is_ceil_quarter() {
    assert_eq!(estimate_tokens(""), 0);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcde"), 2);
    assert_eq!(estimate_tokens("abcdefgh"), 2);
}

#[test]
fn items_within_budget_are_kept() {
    let lines = vec!["a".repeat(40), "b".repeat(40)];
    let result = apply(&lines, 100);
    assert_eq!(result.text.lines().count(), 2);
    assert!(!result.truncated);
    assert_eq!(result.dropped, 0);
}

#[test]
fn last_item_is_truncated_when_tail_is_large() {
    let lines = vec!["x".repeat(1000)];
    let result = apply(&lines, 150);
    assert!(result.truncated);
    assert!(result.text.ends_with('…'));
    assert!(estimate_tokens(&result.text) <= 200);
}

#[test]
fn small_tail_drops_item() {
    let lines = vec!["a".repeat(400), "b".repeat(400)];
    let result = apply(&lines, 120);
    assert_eq!(result.text.lines().count(), 1);
    assert_eq!(result.dropped, 1);
    assert!(!result.truncated);
    assert!(120_usize.saturating_sub(estimate_tokens(&result.text)) < MIN_TAIL);
}

#[test]
fn zero_budget_keeps_nothing() {
    let lines = vec!["a".repeat(40)];
    let result = apply(&lines, 0);
    assert!(result.text.is_empty());
    assert_eq!(result.dropped, 1);
}

#[test]
fn apply_into_matches_apply() {
    let lines = vec!["a".repeat(40), "b".repeat(40)];
    let mut out = String::new();
    let summary = apply_into(&lines, 100, &mut out);
    assert_eq!(out, apply(&lines, 100).text);
    assert_eq!(summary.dropped, 0);
}
