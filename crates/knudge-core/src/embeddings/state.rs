//! Estado derivado da fila de embeddings e modo de digestão (E11-T03, D80/D83).

use crate::{Error, Result};

/// Estado de embedding de uma nota — **derivado**, nunca no frontmatter (D80).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingState {
    /// Vetor presente e atual.
    Indexed,
    /// Sem vetor (nunca digerida ou provedor falhou).
    Pending,
    /// Vetor desatualizado (o corpo mudou desde a última digestão).
    Stale,
}

impl EmbeddingState {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Indexed => "indexed",
            Self::Pending => "pending",
            Self::Stale => "stale",
        }
    }
}

/// Modo de digestão (config `embeddings.mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingMode {
    /// Digere no fim de cada invocação do CLI (default; E11-T03).
    Lazy,
    /// Só via `kd maintenance index --drain`.
    Manual,
}

impl EmbeddingMode {
    /// Todos os modos, em ordem canônica.
    pub const ALL: [Self; 2] = [Self::Lazy, Self::Manual];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lazy => "lazy",
            Self::Manual => "manual",
        }
    }

    /// Interpreta o rótulo de config.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para valor desconhecido.
    pub fn parse(text: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|mode| mode.as_str() == text)
            .ok_or_else(|| Error::config(format!("embeddings.mode inválido: `{text}`")))
    }

    /// `true` se o modo digere ociosamente (no fim da invocação do CLI).
    #[must_use]
    pub const fn drains_on_idle(self) -> bool {
        matches!(self, Self::Lazy)
    }
}

/// Classifica o estado a partir do vetor indexado: `None` = ausente, `Some(true)` = atual,
/// `Some(false)` = desatualizado (o corpo mudou).
#[must_use]
pub fn classify(indexed: Option<bool>) -> EmbeddingState {
    match indexed {
        None => EmbeddingState::Pending,
        Some(true) => EmbeddingState::Indexed,
        Some(false) => EmbeddingState::Stale,
    }
}

/// `true` se a fila passou do teto de backpressure (`max_pending`; 0 = ilimitado).
#[must_use]
pub fn is_backlogged(pending: usize, max_pending: usize) -> bool {
    max_pending > 0 && pending > max_pending
}
