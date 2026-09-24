//! Cache de embeddings por `(body_hash, modelo)`, com teto e eviction LRU (E11-T02, D83/D148/D153).
//!
//! Um *hit* **pula a inferência**. O cache é derivado e descartável: falha de leitura/parse
//! degrada para *pass-through* (cache vazio + warning), nunca é fatal. Com
//! `embeddings.version_cache` ele passa a ser **versionado** (D148): mora em
//! `.knudge/emb_cache.jsonl`, sem eviction/TTL e com `merge=union` no git. A chave lógica é
//! `(body_hash, modelo)` (D153): entradas de outro modelo são ignoradas no *lookup* e a mesma
//! chave com vetores divergentes desempata por `created_ms` (nunca "o primeiro do arquivo").

mod persist;

use std::cmp::Ordering;

use indexmap::IndexMap;

/// Teto default do cache em bytes (32 MiB).
pub const DEFAULT_MAX_BYTES: usize = 33_554_432;

/// Nome do arquivo do cache (`.idx/emb_cache.jsonl` ou `.knudge/emb_cache.jsonl`).
pub const CACHE_FILE: &str = "emb_cache.jsonl";

/// Chave lógica do cache: `(body_hash, modelo)` (D153).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CacheKey {
    /// `body_hash` da nota.
    pub hash: String,
    /// Modelo/revisão de origem.
    pub model: String,
}

/// Entrada do cache: vetor e instante de criação (desempate determinístico — D153).
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    /// Vetor normalizado.
    pub vector: Vec<f32>,
    /// Instante de criação em ms.
    pub created_ms: i64,
}

/// Cache LRU em memória, persistido como JSONL.
#[derive(Debug, Clone)]
pub struct EmbeddingCache {
    entries: IndexMap<CacheKey, CacheEntry>,
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

    /// Consulta a entrada do **modelo ativo** e a promove a MRU (D153).
    pub fn get(&mut self, hash: &str, model: &str) -> Option<Vec<f32>> {
        let key = CacheKey {
            hash: hash.to_string(),
            model: model.to_string(),
        };
        let entry = self.entries.shift_remove(&key)?;
        let vector = entry.vector.clone();
        self.entries.insert(key, entry);
        self.hits = self.hits.saturating_add(1);
        Some(vector)
    }

    /// Insere (ou mantém o vencedor determinístico) e devolve `true` se coube no teto.
    ///
    /// Mesma `(hash, model)` com vetores divergentes: vence o `created_ms` maior; empate,
    /// o vetor lexicograficamente maior (D153).
    pub fn insert(&mut self, hash: &str, vector: Vec<f32>, model: &str, now_ms: i64) -> bool {
        let incoming = entry_bytes(&vector, model);
        if incoming > self.max_bytes {
            return false;
        }
        let key = CacheKey {
            hash: hash.to_string(),
            model: model.to_string(),
        };
        if let Some(existing) = self.entries.get(&key)
            && (existing.created_ms > now_ms
                || (existing.created_ms == now_ms && vector_ge(&existing.vector, &vector)))
        {
            return true;
        }
        if let Some(previous) = self.entries.shift_remove(&key) {
            self.bytes = self
                .bytes
                .saturating_sub(entry_bytes(&previous.vector, &key.model));
        }
        self.evict_until_fit(incoming);
        self.entries.insert(
            key,
            CacheEntry {
                vector,
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
            self.bytes = self.recompute_bytes();
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

    fn recompute_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|(key, entry)| entry_bytes(&entry.vector, &key.model))
            .sum()
    }

    fn evict_until_fit(&mut self, incoming: usize) {
        while self.bytes.saturating_add(incoming) > self.max_bytes {
            let Some((key, entry)) = self.entries.shift_remove_index(0) else {
                break;
            };
            self.bytes = self
                .bytes
                .saturating_sub(entry_bytes(&entry.vector, &key.model));
        }
    }
}

/// `true` se `left` é lexicograficamente maior (ou igual) a `right` — desempate D153.
fn vector_ge(left: &[f32], right: &[f32]) -> bool {
    for (a, b) in left.iter().zip(right.iter()) {
        match a.total_cmp(b) {
            Ordering::Equal => {}
            Ordering::Greater => return true,
            Ordering::Less => return false,
        }
    }
    left.len() >= right.len()
}

fn entry_bytes(vector: &[f32], model: &str) -> usize {
    vector.len().saturating_mul(4).saturating_add(model.len())
}
