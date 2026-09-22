//! Embedder determinístico por hash, para testes/CI/offline (E11-T08, D89).
//!
//! Sem pesos, sem download e sem rede: cada dimensão é derivada de `SHA-256(texto ‖ índice)`,
//! mapeada para `[-1, 1]` e o vetor é normalizado. É **determinístico** — o mesmo texto produz
//! sempre o mesmo vetor —, o que torna a fila/cache/reconcile testáveis sem provedor externo.

use sha2::{Digest, Sha256};

use super::meta::{EmbeddingMeta, Similarity};
use super::vector::{from_f64, normalize};
use crate::Result;
use crate::ports::Embedder;

/// Dimensão default (mesma dos modelos MS MARCO — D79).
pub const DEFAULT_DIMENSIONS: usize = 384;

/// Gera um vetor determinístico e normalizado a partir do texto.
#[must_use]
pub fn embed(text: &str, dimensions: usize) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dimensions);
    for index in 0..dimensions {
        let counter = u32::try_from(index).unwrap_or(u32::MAX);
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.update(counter.to_le_bytes());
        let digest = hasher.finalize();
        let bits = digest.first_chunk::<4>().copied().unwrap_or([0; 4]);
        let value = f64::from(i32::from_le_bytes(bits)) / f64::from(i32::MAX);
        vector.push(from_f64(value));
    }
    normalize(&mut vector);
    vector
}

/// Embedder `lightweight` (hash SHA-256; D89).
#[derive(Debug, Clone)]
pub struct LightweightEmbedder {
    meta: EmbeddingMeta,
}

impl LightweightEmbedder {
    /// Cria com a dimensão dada.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a dimensão for zero.
    pub fn new(dimensions: usize) -> Result<Self> {
        Ok(Self {
            meta: EmbeddingMeta::new(
                "lightweight",
                "hash-sha256",
                "1",
                dimensions,
                Similarity::Cosine,
            )?,
        })
    }

    /// Envolve uma identidade já construída.
    #[must_use]
    pub const fn with_meta(meta: EmbeddingMeta) -> Self {
        Self { meta }
    }
}

impl Embedder for LightweightEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|text| embed(text, self.meta.dimensions))
            .collect())
    }
}
