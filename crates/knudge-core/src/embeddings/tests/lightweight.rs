//! Testes do embedder determinístico (E11-T08).

use crate::Result;
use crate::embeddings::vector::is_normalized;
use crate::embeddings::{LightweightEmbedder, lightweight_embed};
use crate::ports::Embedder;

#[test]
fn same_text_same_vector() {
    let a = lightweight_embed("nota", 16);
    let b = lightweight_embed("nota", 16);
    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
}

#[test]
fn different_text_differs_and_is_normalized() {
    let a = lightweight_embed("nota a", 16);
    let b = lightweight_embed("nota b", 16);
    assert_ne!(a, b);
    assert!(is_normalized(&a, 1e-5));
    assert!(is_normalized(&b, 1e-5));
}

#[test]
fn embedder_preserves_batch_order() -> Result<()> {
    let embedder = LightweightEmbedder::new(8)?;
    let texts = vec!["a".to_string(), "b".to_string()];
    let vectors = embedder.embed(&texts)?;
    assert_eq!(vectors.len(), 2);
    assert_eq!(vectors.first(), Some(&lightweight_embed("a", 8)));
    assert_eq!(vectors.get(1), Some(&lightweight_embed("b", 8)));
    assert_eq!(embedder.meta().dimensions, 8);
    Ok(())
}

#[test]
fn zero_dimensions_is_rejected() {
    assert!(LightweightEmbedder::new(0).is_err());
}
