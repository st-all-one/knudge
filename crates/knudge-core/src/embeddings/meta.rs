//! Identidade do provedor e do índice vetorial (E11-T01, D79).

use indexmap::IndexMap;

use crate::config::Config;
use crate::schema::{Value, hash};
use crate::{Error, Result};

/// Métrica de similaridade do provedor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Similarity {
    /// Similaridade de cosseno (default; modelos `-cos`).
    Cosine,
    /// Produto interno.
    Dot,
}

impl Similarity {
    /// Todos os valores, em ordem canônica.
    pub const ALL: [Self; 2] = [Self::Cosine, Self::Dot];

    /// Rótulo canônico (chave de config).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cosine => "cosine",
            Self::Dot => "dot",
        }
    }

    /// Interpreta o rótulo de config.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para valor desconhecido.
    pub fn parse(text: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.as_str() == text)
            .ok_or_else(|| Error::config(format!("embeddings.similarity inválido: `{text}`")))
    }
}

/// Identidade do provedor/modelo. Qualquer mudança **invalida** o índice (D79).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingMeta {
    /// Provedor (`http`, `lightweight`, `none`).
    pub provider: String,
    /// Modelo (ex.: `sentence-transformers/msmarco-MiniLM-L12-cos-v5`).
    pub model: String,
    /// Revisão/commit pinado.
    pub revision: String,
    /// Dimensão dos vetores.
    pub dimensions: usize,
    /// Métrica de similaridade.
    pub similarity: Similarity,
}

impl EmbeddingMeta {
    /// Constrói e valida a identidade.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a dimensão for zero.
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        revision: impl Into<String>,
        dimensions: usize,
        similarity: Similarity,
    ) -> Result<Self> {
        let meta = Self {
            provider: provider.into(),
            model: model.into(),
            revision: revision.into(),
            dimensions,
            similarity,
        };
        meta.validate()?;
        Ok(meta)
    }

    /// Lê a identidade da config efetiva.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para valor inválido.
    pub fn from_config(config: &Config) -> Result<Self> {
        let provider = config.get_str("embeddings.provider").unwrap_or("none");
        let model = config
            .get_str("embeddings.model")
            .unwrap_or("ibm-granite/granite-embedding-97m-multilingual-r2")
            .to_string();
        let revision = config.get_str("embeddings.revision").unwrap_or("main");
        let dimensions = config.get_int("embeddings.dimensions").unwrap_or(384);
        let dimensions = usize::try_from(dimensions)
            .map_err(|_| Error::config(format!("embeddings.dimensions inválido: {dimensions}")))?;
        let similarity =
            Similarity::parse(config.get_str("embeddings.similarity").unwrap_or("cosine"))?;
        Self::new(provider, model, revision, dimensions, similarity)
    }

    /// Valida a identidade.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a dimensão for zero.
    pub fn validate(&self) -> Result<()> {
        if self.dimensions == 0 {
            return Err(Error::config("embeddings.dimensions deve ser > 0"));
        }
        Ok(())
    }

    /// Impressão digital determinística (hex8) da identidade.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        hash::hex8(
            format!(
                "{}|{}|{}|{}|{}",
                self.provider,
                self.model,
                self.revision,
                self.dimensions,
                self.similarity.as_str()
            )
            .as_bytes(),
        )
    }

    /// `true` se duas identidades são compatíveis (índice reutilizável).
    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        self == other
    }
}

/// Serializa a identidade como objeto JSON (cabeçalho do índice).
pub(crate) fn meta_to_value(meta: &EmbeddingMeta) -> Value {
    let mut map = IndexMap::new();
    map.insert("provider".to_string(), Value::Str(meta.provider.clone()));
    map.insert("model".to_string(), Value::Str(meta.model.clone()));
    map.insert("revision".to_string(), Value::Str(meta.revision.clone()));
    map.insert(
        "dimensions".to_string(),
        Value::Int(i64::try_from(meta.dimensions).unwrap_or(i64::MAX)),
    );
    map.insert(
        "similarity".to_string(),
        Value::Str(meta.similarity.as_str().to_string()),
    );
    Value::Map(map)
}

/// Lê a identidade de um cabeçalho JSON.
pub(crate) fn meta_from_value(value: &Value) -> Result<EmbeddingMeta> {
    let map = value
        .as_map()
        .and_then(|outer| outer.get("meta"))
        .and_then(Value::as_map)
        .ok_or_else(|| Error::schema("cabeçalho do índice de embeddings sem `meta`"))?;
    let text = |key: &str| {
        map.get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::schema(format!("meta de embeddings sem `{key}`")))
    };
    let dimensions = map
        .get("dimensions")
        .and_then(Value::as_int)
        .ok_or_else(|| Error::schema("meta de embeddings sem `dimensions`"))?;
    let dimensions = usize::try_from(dimensions)
        .map_err(|_| Error::schema(format!("dimensão inválida: {dimensions}")))?;
    let similarity = Similarity::parse(text("similarity")?)?;
    EmbeddingMeta::new(
        text("provider")?,
        text("model")?,
        text("revision")?,
        dimensions,
        similarity,
    )
}
