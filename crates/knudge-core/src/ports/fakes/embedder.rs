//! Fake do provedor de embeddings (E11).

use std::sync::Mutex;

use crate::Result;
use crate::embeddings::{EmbeddingMeta, Similarity, lightweight_embed};
use crate::error::lock_or_recover;
use crate::ports::Embedder;

/// Embedder falso determinístico (hash SHA-256) com falhas injetáveis.
#[derive(Debug)]
pub struct FakeEmbedder {
    /// Identidade (dimensão vem daqui).
    meta: EmbeddingMeta,
    /// Tamanhos de lote por chamada.
    calls: Mutex<Vec<usize>>,
    /// Falhas restantes a injetar.
    fail_next: Mutex<usize>,
}

impl FakeEmbedder {
    /// Cria com a dimensão dada.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a dimensão for zero.
    pub fn new(dimensions: usize) -> Result<Self> {
        Ok(Self {
            meta: EmbeddingMeta::new(
                "lightweight",
                "fake-sha256",
                "test",
                dimensions,
                Similarity::Cosine,
            )?,
            calls: Mutex::new(Vec::new()),
            fail_next: Mutex::new(0),
        })
    }

    /// Faz as próximas `count` chamadas falharem (marca `pending`).
    pub fn fail_next(&self, count: usize) {
        *lock_or_recover(&self.fail_next) = count;
    }

    /// Tamanhos de lote chamados, em ordem.
    #[must_use]
    pub fn calls(&self) -> Vec<usize> {
        lock_or_recover(&self.calls).clone()
    }
}

impl Embedder for FakeEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        lock_or_recover(&self.calls).push(texts.len());
        {
            let mut fail = lock_or_recover(&self.fail_next);
            if *fail > 0 {
                *fail = fail.saturating_sub(1);
                return Err(crate::Error::internal("provedor indisponível (fake)"));
            }
        }
        Ok(texts
            .iter()
            .map(|text| lightweight_embed(text, self.meta.dimensions))
            .collect())
    }
}
