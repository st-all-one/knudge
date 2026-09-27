//! Tokenização com fold de diacríticos (D36/D172).

use proptest::prelude::*;

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
fn folds_accents_and_lowercases() {
    assert_eq!(
        owned("Rust café RÁPIDO foo_bar"),
        ["rust", "cafe", "rapido", "foo_bar"]
    );
}

#[test]
fn precomposed_and_decomposed_fold_equivalently() {
    // `café` (NFC) e `cafe\u{301}` (NFD) produzem o mesmo termo.
    assert_eq!(owned("café"), owned("cafe\u{301}"));
    assert_eq!(owned("café"), ["cafe"]);
    assert_eq!(
        owned("ação çedilha ñ über"),
        ["acao", "cedilha", "n", "uber"]
    );
}

#[test]
fn non_latin_without_ascii_decomposition_is_separator() {
    // Cirílico/CJK não têm decomposição ASCII: seguem separadores (D36).
    assert_eq!(owned("привет Rust 北京"), ["rust"]);
}

#[test]
fn fold_keeps_word_boundaries() {
    assert_eq!(owned("caf e"), ["caf", "e"]);
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
    // `são` → `sao` (3 chars): não é mais fragmento, mas segue stopword (D173).
    assert_eq!(content("são paulo"), ["paulo"]);
    // `ação` → `acao`: dobra para um termo de conteúdo.
    assert_eq!(content("ação paulo"), ["acao", "paulo"]);
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

proptest! {
    #[test]
    fn fold_is_idempotent(input in "\\PC*") {
        let first = owned(&input);
        let second = owned(&first.join(" "));
        prop_assert_eq!(first, second);
    }

    #[test]
    fn folded_terms_are_ascii_and_lowercase(input in "\\PC*") {
        for term in owned(&input) {
            prop_assert!(term.is_ascii());
            prop_assert_eq!(term.as_str(), term.to_ascii_lowercase());
        }
    }
}
