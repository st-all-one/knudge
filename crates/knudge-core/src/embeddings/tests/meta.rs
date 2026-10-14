//! Testes dos metadados do provedor (E11-T01).

use crate::Result;
use crate::config::Config;
use crate::embeddings::{EmbeddingMeta, Prefix, Similarity};

#[test]
fn defaults_build_cosine_http_meta() -> Result<()> {
    let meta = EmbeddingMeta::from_config(&Config::defaults())?;
    assert_eq!(meta.provider, "http");
    assert_eq!(meta.model, "google/embeddinggemma-2");
    assert_eq!(meta.dimensions, 768);
    assert_eq!(meta.similarity, Similarity::Cosine);
    assert_eq!(meta.prefix, Prefix::EmbeddingGemma);
    Ok(())
}

#[test]
fn fingerprint_changes_with_prefix() -> Result<()> {
    let base = EmbeddingMeta::new(
        "http",
        "google/embeddinggemma-2",
        "a",
        768,
        Similarity::Cosine,
        Prefix::EmbeddingGemma,
    )?;
    let other = EmbeddingMeta::new(
        "http",
        "google/embeddinggemma-2",
        "a",
        768,
        Similarity::Cosine,
        Prefix::None,
    )?;
    assert_ne!(base.fingerprint(), other.fingerprint());
    assert!(!base.matches(&other));
    assert_ne!(base.cache_key(), other.cache_key());
    Ok(())
}

#[test]
fn fingerprint_changes_with_model_or_revision() -> Result<()> {
    let base = EmbeddingMeta::new("http", "modelo", "a", 384, Similarity::Cosine, Prefix::None)?;
    let other = EmbeddingMeta::new("http", "modelo", "b", 384, Similarity::Cosine, Prefix::None)?;
    assert_ne!(base.fingerprint(), other.fingerprint());
    assert!(base.matches(&base.clone()));
    assert!(!base.matches(&other));
    Ok(())
}

#[test]
fn rejects_zero_dimensions_and_bad_similarity() {
    assert!(
        EmbeddingMeta::new("http", "modelo", "a", 0, Similarity::Cosine, Prefix::None).is_err()
    );
    assert!(Similarity::parse("manhattan").is_err());
    assert_eq!(Similarity::parse("dot").ok(), Some(Similarity::Dot));
    assert_eq!(Similarity::parse("cosine").ok(), Some(Similarity::Cosine));
}
