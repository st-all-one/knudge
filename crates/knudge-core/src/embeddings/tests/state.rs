//! Testes do estado da fila e do modo de digestão (E11-T03).

use crate::embeddings::{EmbeddingMode, EmbeddingState, classify, is_backlogged};

#[test]
fn classify_maps_presence_and_hash() {
    assert_eq!(classify(None), EmbeddingState::Pending);
    assert_eq!(classify(Some(true)), EmbeddingState::Indexed);
    assert_eq!(classify(Some(false)), EmbeddingState::Stale);
}

#[test]
fn mode_parse_and_flags() {
    assert_eq!(EmbeddingMode::parse("lazy").ok(), Some(EmbeddingMode::Lazy));
    assert_eq!(
        EmbeddingMode::parse("manual").ok(),
        Some(EmbeddingMode::Manual)
    );
    assert!(EmbeddingMode::parse("eager").is_err());
    assert!(EmbeddingMode::parse("turbo").is_err());
    assert!(EmbeddingMode::Lazy.drains_on_idle());
    assert!(!EmbeddingMode::Manual.drains_on_idle());
}

#[test]
fn backpressure_threshold() {
    assert!(!is_backlogged(10, 1000));
    assert!(is_backlogged(1001, 1000));
    assert!(!is_backlogged(usize::MAX, 0));
}

#[test]
fn state_labels_are_canonical() {
    assert_eq!(EmbeddingState::Indexed.as_str(), "indexed");
    assert_eq!(EmbeddingState::Pending.as_str(), "pending");
    assert_eq!(EmbeddingState::Stale.as_str(), "stale");
}
