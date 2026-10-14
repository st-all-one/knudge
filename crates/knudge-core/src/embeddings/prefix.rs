//! Prefixos de instrução de tarefa por família de modelo (E11, D216).
//!
//! Alguns modelos (`EmbeddingGemma`) são treinados com prefixos **assimétricos**: o documento é
//! formatado com título (`title: … | text: …`) e a consulta com a tarefa
//! (`task: search result | query: …`). Outros (granite) não pedem prefixo.
//!
//! `embeddings.prefix = auto` (default) resolve pelo nome do modelo; `none` desliga;
//! `embeddinggemma` força. O prefixo **resolvido** entra na identidade (`EmbeddingMeta`), então
//! mudá-lo invalida o índice (D79) — nunca se comparam vetores de prefixos diferentes.

use crate::{Error, Result};

/// Prefixo de tarefa resolvido (propriedade do modelo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prefix {
    /// Sem prefixo — modelos que não pedem instrução (ex.: granite).
    None,
    /// Prefixos do `EmbeddingGemma` (documento com título, consulta com `SearchQuery`).
    EmbeddingGemma,
}

impl Prefix {
    /// Valores persistidos, em ordem canônica.
    pub const ALL: [Self; 2] = [Self::None, Self::EmbeddingGemma];

    /// Rótulos aceitos em `embeddings.prefix` (inclui `auto`, que não é persistido).
    pub const CONFIG_VALUES: [&'static str; 3] = ["auto", "none", "embeddinggemma"];

    /// Rótulo canônico do prefixo resolvido.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::EmbeddingGemma => "embeddinggemma",
        }
    }

    /// Resolve o valor de config contra o modelo ativo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para valor desconhecido.
    pub fn from_config(value: &str, model: &str) -> Result<Self> {
        match value {
            "auto" => Ok(Self::for_model(model)),
            "none" => Ok(Self::None),
            "embeddinggemma" => Ok(Self::EmbeddingGemma),
            other => Err(Error::config(format!(
                "embeddings.prefix inválido: `{other}` (use auto|none|embeddinggemma)"
            ))),
        }
    }

    /// Heurística determinística modelo→prefixo (usada por `auto`).
    #[must_use]
    pub fn for_model(model: &str) -> Self {
        if model.to_ascii_lowercase().contains("embeddinggemma") {
            Self::EmbeddingGemma
        } else {
            Self::None
        }
    }

    /// Texto do documento (lado corpus).
    #[must_use]
    pub fn document(self, statement: &str, body: &str) -> String {
        match self {
            Self::None => super::pipeline::embedding_text(statement, body),
            Self::EmbeddingGemma => {
                if body.trim().is_empty() {
                    format!("title: {statement} | text: {statement}")
                } else {
                    format!("title: {statement} | text: {}", body.trim())
                }
            }
        }
    }

    /// Texto da consulta (lado query).
    #[must_use]
    pub fn query(self, text: &str) -> String {
        match self {
            Self::None => text.to_string(),
            Self::EmbeddingGemma => format!("task: search result | query: {text}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Prefix;

    #[test]
    fn auto_resolves_by_model_family() {
        assert_eq!(
            Prefix::from_config("auto", "google/embeddinggemma-2").ok(),
            Some(Prefix::EmbeddingGemma)
        );
        assert_eq!(
            Prefix::from_config("auto", "ibm-granite/granite-embedding-97m-multilingual-r2").ok(),
            Some(Prefix::None)
        );
        assert_eq!(
            Prefix::from_config("auto", "x/EmbeddingGemma-300m").ok(),
            Some(Prefix::EmbeddingGemma)
        );
    }

    #[test]
    fn explicit_values_win_over_model() {
        assert_eq!(
            Prefix::from_config("none", "google/embeddinggemma-2").ok(),
            Some(Prefix::None)
        );
        assert_eq!(
            Prefix::from_config("embeddinggemma", "outro/modelo").ok(),
            Some(Prefix::EmbeddingGemma)
        );
        assert!(Prefix::from_config("nope", "m").is_err());
    }

    #[test]
    fn none_prefix_is_the_canonical_embedding_text() {
        assert_eq!(
            Prefix::None.document("afirmação", "corpo"),
            "afirmação\ncorpo"
        );
        assert_eq!(Prefix::None.document("afirmação", "  "), "afirmação");
        assert_eq!(Prefix::None.query("busca"), "busca");
    }

    #[test]
    fn embeddinggemma_document_uses_title_and_body() {
        assert_eq!(
            Prefix::EmbeddingGemma.document("título", "corpo"),
            "title: título | text: corpo"
        );
        assert_eq!(
            Prefix::EmbeddingGemma.document("título", "  "),
            "title: título | text: título"
        );
        assert_eq!(
            Prefix::EmbeddingGemma.query("pergunta"),
            "task: search result | query: pergunta"
        );
    }

    #[test]
    fn labels_are_canonical() {
        assert_eq!(Prefix::None.as_str(), "none");
        assert_eq!(Prefix::EmbeddingGemma.as_str(), "embeddinggemma");
        assert_eq!(Prefix::ALL.len(), 2);
    }
}
