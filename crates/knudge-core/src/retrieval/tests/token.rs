//! Tokenização ASCII (D36).

use crate::retrieval::token::{STOPWORDS, content_terms, is_stopword, query_terms, tokenize};

fn owned(input: &str) -> Vec<String> {
    tokenize(input)
        .into_iter()
        .map(std::borrow::Cow::into_owned)
        .collect()
}

fn content(input: &str) -> Vec<String> {
    content_terms(input)
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

#[test]
fn content_terms_drop_stopwords_and_short_fragments() {
    assert_eq!(
        content("tempestade de requisicoes"),
        ["tempestade", "requisicoes"]
    );
    // `são` → `s`,`o`: o fragmento de 1 char e a stopword somem.
    assert_eq!(content("são paulo"), ["paulo"]);
    assert!(content("de a o").is_empty());
}

#[test]
fn stopwords_are_sorted_for_binary_search() {
    let mut sorted = STOPWORDS.to_vec();
    sorted.sort_unstable();
    assert_eq!(
        STOPWORDS,
        sorted.as_slice(),
        "STOPWORDS deve estar ordenada"
    );
    assert!(is_stopword("de"));
    assert!(!is_stopword("retry"));
}
