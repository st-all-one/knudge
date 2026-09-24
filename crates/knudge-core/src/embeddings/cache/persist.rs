//! Persistência JSONL do cache de embeddings (E11-T02, D148/D153).
//!
//! `merge=union` no git pode concatenar dois arquivos; o `parse` dedupa por `(hash, model)`
//! via [`EmbeddingCache::insert`] (desempate determinístico por `created_ms`).

use std::path::Path;

use indexmap::IndexMap;

use crate::Result;
use crate::jsonl;
use crate::jsonl::json;
use crate::ports::Fs;
use crate::schema::Value;

use super::super::vector::{from_f64, to_f64};
use super::{CacheKey, EmbeddingCache};

impl EmbeddingCache {
    /// Serializa em JSONL canônico (uma linha por entrada, ordenado por chave).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum float não for codificável.
    pub fn serialize(&self) -> Result<String> {
        let mut keys: Vec<&CacheKey> = self.entries.keys().collect();
        keys.sort();
        let mut out = String::new();
        for key in keys {
            let Some(entry) = self.entries.get(key) else {
                continue;
            };
            let mut map = IndexMap::new();
            map.insert("hash".to_string(), Value::Str(key.hash.clone()));
            map.insert("model".to_string(), Value::Str(key.model.clone()));
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
            map.insert("created_ms".to_string(), Value::Int(entry.created_ms));
            out.push_str(&json::encode(&Value::Map(map))?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Lê de JSONL, aplicando o teto e a eviction (dedup por `(hash, model)` — D153).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` para linha malformada.
    pub fn parse(text: &str, max_bytes: usize) -> Result<Self> {
        let mut cache = Self::new(max_bytes);
        for line in jsonl::lines(text) {
            let value = json::decode(line)?;
            let Some(map) = value.as_map() else {
                continue;
            };
            let Some(hash) = map.get("hash").and_then(Value::as_str) else {
                continue;
            };
            let model = map
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let created_ms = map.get("created_ms").and_then(Value::as_int).unwrap_or(0);
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
            cache.insert(hash, vector, &model, created_ms);
        }
        Ok(cache)
    }

    /// Grava atomicamente, criando o diretório pai.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, fs: &dyn Fs, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs.create_dir_all(parent)?;
        }
        fs.write_atomic(path, self.serialize()?.as_bytes())
    }

    /// Carrega se existir; qualquer falha degrada para cache vazio + warning (D83/R33).
    #[must_use]
    pub fn load(fs: &dyn Fs, path: &Path, max_bytes: usize, warnings: &mut Vec<String>) -> Self {
        if !fs.exists(path) {
            return Self::new(max_bytes);
        }
        let bytes = match fs.read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                warnings.push(format!("cache de embeddings ilegível: {error}"));
                return Self::new(max_bytes);
            }
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            warnings.push(format!(
                "cache de embeddings não é UTF-8: {}",
                path.display()
            ));
            return Self::new(max_bytes);
        };
        match Self::parse(text, max_bytes) {
            Ok(cache) => cache,
            Err(error) => {
                warnings.push(format!("cache de embeddings inválido: {error}"));
                Self::new(max_bytes)
            }
        }
    }
}
