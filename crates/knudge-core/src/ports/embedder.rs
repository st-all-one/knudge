//! Porta do provedor de embedding (E11-T01, D79).
//!
//! O domínio **nunca** fala HTTP nem carrega modelo: depende desta trait. As implementações
//! reais ficam na borda — [`crate::adapters::http::HttpEmbedder`] (servidor local
//! OpenAI-compatible, ex.: `llama-server` com o GGUF) e
//! [`crate::embeddings::LightweightEmbedder`] (determinístico, para testes/CI — D89). Os testes
//! do núcleo usam [`crate::ports::fakes::FakeEmbedder`].

use crate::Result;
use crate::embeddings::EmbeddingMeta;

/// Provedor de vetores plugável via `config [embeddings]` (D79).
pub trait Embedder: Send + Sync {
    /// Identidade do provedor/modelo; mudança **invalida** o índice (D79).
    fn meta(&self) -> &EmbeddingMeta;

    /// Embute um lote de textos, preservando a ordem de entrada.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Timeout`/`Internal` em falha do provedor. O chamador marca as notas
    /// como `pending` e tenta de novo depois — **nunca** descarta a nota (D83).
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
