//! Testes dos metadados do provedor (E11-T01).

use crate::Result;
use crate::config::Config;
use crate::embeddings::{EmbeddingMeta, Similarity};

#[test]
fn defaults_build_cosine_http_meta() -> Result<()> {
    let meta = EmbeddingMeta::from_config(&Config::defaults())?;
    assert_eq!(meta.provider, "http");
    assert_eq!(meta.dimensions, 384);
    assert_eq!(meta.similarity, Similarity::Cosine);
    Ok(())
}

#[test]
fn fingerprint_changes_with_model_or_revision() -> Result<()> {
    let base = EmbeddingMeta::new("http", "modelo", "a", 384, Similarity::Cosine)?;
    let other = EmbeddingMeta::new("http", "modelo", "b", 384, Similarity::Cosine)?;
    assert_ne!(base.fingerprint(), other.fingerprint());
    assert!(base.matches(&base.clone()));
    assert!(!base.matches(&other));
    Ok(())
}

#[test]
fn rejects_zero_dimensions_and_bad_similarity() {
    assert!(EmbeddingMeta::new("http", "modelo", "a", 0, Similarity::Cosine).is_err());
    assert!(Similarity::parse("manhattan").is_err());
    assert_eq!(Similarity::parse("dot").ok(), Some(Similarity::Dot));
    assert_eq!(Similarity::parse("cosine").ok(), Some(Similarity::Cosine));
}
