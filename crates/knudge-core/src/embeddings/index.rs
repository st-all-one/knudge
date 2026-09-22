//! Índice vetorial derivado (`.idx/embeddings.jsonl`) e invalidação por modelo (E11-T01/T03).
//!
//! O índice é **derivado** de `notas/` + provedor: reconstruível e descartável (D15/D84). A
//! primeira linha é o cabeçalho de identidade (`meta`); se ele não casar com o provedor atual, o
//! índice é **invalidado** (re-embed completo — D79). A remoção de notas é coberta por
//! [`crate::store::purge_derived`], que apaga registros com `id` (T05).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

use super::meta::{EmbeddingMeta, meta_from_value, meta_to_value};
use super::state::{EmbeddingState, classify};
use super::vector::{from_f64, to_f64};

/// Nome do arquivo do índice vetorial dentro de `.idx/`.
pub const INDEX_FILE: &str = "embeddings.jsonl";

/// Acima deste tamanho (bytes) o índice emite aviso (R14).
pub const WARN_BYTES: u64 = 67_108_864;

/// Vetor indexado de uma nota.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedVector {
    /// `body_hash` do conteúdo que gerou o vetor.
    pub body_hash: String,
    /// Vetor normalizado.
    pub vector: Vec<f32>,
}

/// Índice vetorial: identidade + entradas por id (ordenadas).
#[derive(Debug, Clone)]
pub struct EmbeddingIndex {
    /// Identidade do provedor que gerou o índice.
    pub meta: EmbeddingMeta,
    entries: BTreeMap<String, IndexedVector>,
}

impl EmbeddingIndex {
    /// Índice vazio para a identidade dada.
    #[must_use]
    pub const fn new(meta: EmbeddingMeta) -> Self {
        Self {
            meta,
            entries: BTreeMap::new(),
        }
    }

    /// Caminho do arquivo do índice.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(".idx").join(INDEX_FILE)
    }

    /// Número de vetores indexados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` se não há vetores.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `true` se a nota tem vetor.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    /// `body_hash` do vetor da nota, se houver.
    #[must_use]
    pub fn body_hash(&self, id: &str) -> Option<&str> {
        self.entries.get(id).map(|entry| entry.body_hash.as_str())
    }

    /// Vetor da nota, se houver.
    #[must_use]
    pub fn vector(&self, id: &str) -> Option<&[f32]> {
        self.entries.get(id).map(|entry| entry.vector.as_slice())
    }

    /// Ids indexados, em ordem canônica.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Insere/substitui o vetor de uma nota.
    pub fn insert(&mut self, id: &str, body_hash: &str, vector: Vec<f32>) {
        self.entries.insert(
            id.to_string(),
            IndexedVector {
                body_hash: body_hash.to_string(),
                vector,
            },
        );
    }

    /// Remove o vetor de uma nota; devolve `true` se existia.
    pub fn purge(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Estado de embedding da nota dado o `body_hash` atual.
    #[must_use]
    pub fn state_of(&self, id: &str, body_hash: &str) -> EmbeddingState {
        classify(
            self.entries
                .get(id)
                .map(|entry| entry.body_hash == body_hash),
        )
    }

    /// Serializa: cabeçalho `meta` + uma linha por entrada.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum valor não for codificável.
    pub fn serialize(&self) -> Result<String> {
        let mut out = String::new();
        let mut header = IndexMap::new();
        header.insert("meta".to_string(), meta_to_value(&self.meta));
        out.push_str(&json::encode(&Value::Map(header))?);
        out.push('\n');
        for (id, entry) in &self.entries {
            let mut map = IndexMap::new();
            map.insert("id".to_string(), Value::Str(id.clone()));
            map.insert("body_hash".to_string(), Value::Str(entry.body_hash.clone()));
            map.insert(
                "vector".to_string(),
                Value::List(
                    entry
                        .vector
                        .iter()
                        .map(|v| Value::Float(to_f64(*v)))
                        .collect(),
                ),
            );
            out.push_str(&json::encode(&Value::Map(map))?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Lê o índice; devolve `None` quando a identidade **não casa** (invalidação — D79).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput`/`Schema` para linha malformada.
    pub fn parse(text: &str, expected: &EmbeddingMeta) -> Result<Option<Self>> {
        let mut lines = jsonl::lines(text);
        let Some(first) = lines.next() else {
            return Ok(Some(Self::new(expected.clone())));
        };
        let header = json::decode(first)?;
        let meta = meta_from_value(&header)?;
        if !meta.matches(expected) {
            return Ok(None);
        }
        let mut index = Self::new(meta);
        for line in lines {
            let value = json::decode(line)?;
            let Some(map) = value.as_map() else {
                continue;
            };
            let Some(id) = map.get("id").and_then(Value::as_str) else {
                continue;
            };
            let body_hash = map
                .get("body_hash")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let vector = map
                .get("vector")
                .and_then(Value::as_list)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_f64)
                        .map(from_f64)
                        .collect()
                })
                .unwrap_or_default();
            index.insert(id, &body_hash, vector);
        }
        Ok(Some(index))
    }

    /// Grava atomicamente, avisando se ultrapassar o teto.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, fs: &dyn Fs, root: &Path, warnings: &mut Vec<String>) -> Result<()> {
        let path = Self::path(root);
        if let Some(parent) = path.parent() {
            fs.create_dir_all(parent)?;
        }
        let data = self.serialize()?;
        if let Some(warning) = size_warning(data.len()) {
            warnings.push(warning);
        }
        fs.write_atomic(&path, data.as_bytes())
    }

    /// Carrega o índice, invalidando (com warning) se o modelo/dimensão mudou.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a leitura falhar.
    pub fn load(
        fs: &dyn Fs,
        root: &Path,
        expected: &EmbeddingMeta,
        warnings: &mut Vec<String>,
    ) -> Result<Option<Self>> {
        let path = Self::path(root);
        if !fs.exists(&path) {
            return Ok(None);
        }
        let bytes = fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        if text.trim().is_empty() {
            return Ok(None);
        }
        match Self::parse(text, expected) {
            Ok(Some(index)) => Ok(Some(index)),
            Ok(None) => {
                warnings.push(
                    "índice de embeddings invalidado (modelo/dimensão mudou): re-embed completo"
                        .to_string(),
                );
                Ok(None)
            }
            Err(error) => {
                warnings.push(format!("índice de embeddings inválido: {error}"));
                Ok(None)
            }
        }
    }
}

/// Aviso quando o índice passa de [`WARN_BYTES`] (R14).
#[must_use]
pub fn size_warning(len: usize) -> Option<String> {
    let limit = usize::try_from(WARN_BYTES).unwrap_or(usize::MAX);
    (len > limit).then(|| {
        format!(
            "índice de embeddings de {len} bytes acima do teto de {WARN_BYTES}; considere rebuild"
        )
    })
}
