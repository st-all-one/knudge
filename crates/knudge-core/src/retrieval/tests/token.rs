//! Tokenização ASCII (D36).

use crate::retrieval::token::{query_terms, tokenize};

fn owned(input: &str) -> Vec<String> {
    tokenize(input)
        .into_iter()
        .map(std::borrow::Cow::into_owned)
        .collect()
}

#[test]
fn splits_on_non_ascii_and_lowercases() {
    assert_eq!(
        owned("Rust café RÁPIDO foo_bar"),
        ["rust", "caf", "r", "pido", "foo_bar"]
    );
}

#[test]
fn non_word_bytes_are_separators() {
    assert_eq!(owned("a-b.c/d"), ["a", "b", "c", "d"]);
    assert!(tokenize("   ---  ").is_empty());
}

#[test]
fn query_terms_dedup_preserving_order() {
    let terms: Vec<String> = query_terms("Rust rust RUST json")
        .into_iter()
        .map(std::borrow::Cow::into_owned)
        .collect();
    assert_eq!(terms, ["rust", "json"]);
}
