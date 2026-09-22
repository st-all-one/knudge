//! Cache de embeddings por `body_hash`, com teto e eviction LRU (E11-T02, D83/R14).
//!
//! Um *hit* **pula a inferência**. O cache é derivado e descartável: falha de leitura/parse
//! degrada para *pass-through* (cache vazio + warning), nunca é fatal.

use std::path::Path;

use indexmap::IndexMap;

use crate::Result;
use crate::jsonl;
use crate::jsonl::json;
use crate::ports::Fs;
use crate::schema::Value;

use super::vector::{from_f64, to_f64};

/// Teto default do cache em bytes (32 MiB).
pub const DEFAULT_MAX_BYTES: usize = 33_554_432;

/// Nome do arquivo do cache dentro de `.idx/`.
pub const CACHE_FILE: &str = "emb_cache.jsonl";

/// Entrada do cache: vetor e o modelo que o produziu.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    /// Vetor normalizado.
    pub vector: Vec<f32>,
    /// Modelo/revisão de origem.
    pub model: String,
    /// Instante de criação em ms (para o TTL).
    pub created_ms: i64,
}

/// Cache LRU em memória, persistido como JSONL.
#[derive(Debug, Clone)]
pub struct EmbeddingCache {
    entries: IndexMap<String, CacheEntry>,
    max_bytes: usize,
    bytes: usize,
    hits: u64,
    misses: u64,
}

impl EmbeddingCache {
    /// Cache vazio com o teto dado.
    #[must_use]
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: IndexMap::new(),
            max_bytes,
            bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Consulta e promove a entrada para MRU.
    pub fn get(&mut self, key: &str) -> Option<Vec<f32>> {
        let entry = self.entries.shift_remove(key)?;
        let vector = entry.vector.clone();
        self.entries.insert(key.to_string(), entry);
        self.hits = self.hits.saturating_add(1);
        Some(vector)
    }

    /// Insere (substituindo) e devolve `true` se a entrada coube no teto.
    pub fn insert(&mut self, key: &str, vector: Vec<f32>, model: &str, now_ms: i64) -> bool {
        let incoming = entry_bytes(&vector, model);
        if incoming > self.max_bytes {
            return false;
        }
        if let Some(previous) = self.entries.shift_remove(key) {
            self.bytes = self
                .bytes
                .saturating_sub(entry_bytes(&previous.vector, &previous.model));
        }
        self.evict_until_fit(incoming);
        self.entries.insert(
            key.to_string(),
            CacheEntry {
                vector,
                model: model.to_string(),
                created_ms: now_ms,
            },
        );
        self.bytes = self.bytes.saturating_add(incoming);
        true
    }

    /// Remove entradas mais velhas que `ttl_ms` (0 = sem TTL); devolve quantas saíram.
    pub fn prune_expired(&mut self, now_ms: i64, ttl_ms: i64) -> usize {
        if ttl_ms <= 0 {
            return 0;
        }
        let mut removed = 0_usize;
        self.entries.retain(|_, entry| {
            let expired = now_ms.saturating_sub(entry.created_ms) > ttl_ms;
            if expired {
                removed = removed.saturating_add(1);
            }
            !expired
        });
        if removed > 0 {
            self.bytes = self
                .entries
                .values()
                .map(|entry| entry_bytes(&entry.vector, &entry.model))
                .sum();
        }
        removed
    }

    /// Registra um *miss* (métrica).
    pub const fn record_miss(&mut self) {
        self.misses = self.misses.saturating_add(1);
    }

    /// Número de entradas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` se vazio.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Bytes ocupados.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// Teto em bytes.
    #[must_use]
    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Contador de *hits*.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }

    /// Contador de *misses*.
    #[must_use]
    pub const fn misses(&self) -> u64 {
        self.misses
    }

    fn evict_until_fit(&mut self, incoming: usize) {
        while self.bytes.saturating_add(incoming) > self.max_bytes {
            let Some((_, entry)) = self.entries.shift_remove_index(0) else {
                break;
            };
            self.bytes = self
                .bytes
                .saturating_sub(entry_bytes(&entry.vector, &entry.model));
        }
    }

    /// Serializa em JSONL canônico (uma linha por entrada, ordenado por chave).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum float não for codificável.
    pub fn serialize(&self) -> Result<String> {
        let mut keys: Vec<&String> = self.entries.keys().collect();
        keys.sort();
        let mut out = String::new();
        for key in keys {
            let Some(entry) = self.entries.get(key) else {
                continue;
            };
            let mut map = IndexMap::new();
            map.insert("hash".to_string(), Value::Str(key.clone()));
            map.insert("model".to_string(), Value::Str(entry.model.clone()));
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

    /// Lê de JSONL, aplicando o teto e a eviction.
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

fn entry_bytes(vector: &[f32], model: &str) -> usize {
    vector.len().saturating_mul(4).saturating_add(model.len())
}
