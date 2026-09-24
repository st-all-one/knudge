//! Testes do cache por `(body_hash, modelo)` (E11-T02, D153).

use std::path::Path;

use crate::Result;
use crate::embeddings::{CACHE_DEFAULT_MAX_BYTES, EmbeddingCache};
use crate::ports::fakes::MemFs;

#[test]
fn hit_returns_vector_and_counts() {
    let mut cache = EmbeddingCache::new(1024);
    assert!(cache.insert("hash", vec![1.0, 0.0], "modelo", 0));
    assert_eq!(cache.get("hash", "modelo"), Some(vec![1.0_f32, 0.0]));
    assert_eq!(cache.hits(), 1);
    assert!(cache.get("ausente", "modelo").is_none());
    assert_eq!(cache.len(), 1);
}

#[test]
fn respects_ceiling_with_lru_eviction() {
    let mut cache = EmbeddingCache::new(8);
    assert!(cache.insert("a", vec![1.0, 0.0], "", 0));
    assert!(cache.insert("b", vec![0.0, 1.0], "", 0));
    assert_eq!(cache.len(), 1);
    assert!(cache.get("a", "").is_none());
    assert!(cache.get("b", "").is_some());
    assert!(cache.bytes() <= cache.max_bytes());
}

#[test]
fn oversized_vector_is_rejected() {
    let mut cache = EmbeddingCache::new(4);
    assert!(!cache.insert("a", vec![1.0, 0.0], "", 0));
    assert!(cache.is_empty());
}

#[test]
fn serialize_parse_round_trip() -> Result<()> {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("a", vec![1.0, 0.5], "modelo", 0);
    cache.insert("b", vec![0.0, 1.0], "modelo", 0);
    let text = cache.serialize()?;
    let mut parsed = EmbeddingCache::parse(&text, 1024)?;
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed.get("a", "modelo"), Some(vec![1.0_f32, 0.5]));
    Ok(())
}

#[test]
fn load_tolerates_corruption() {
    let fs = MemFs::new();
    let path = Path::new("/p/.knudge/.idx/emb_cache.jsonl");
    fs.insert(path, "{lixo");
    let mut warnings = Vec::new();
    let cache = EmbeddingCache::load(&fs, path, CACHE_DEFAULT_MAX_BYTES, &mut warnings);
    assert!(cache.is_empty());
    assert!(!warnings.is_empty());
}

#[test]
fn save_and_load_round_trip() -> Result<()> {
    let fs = MemFs::new();
    let path = Path::new("/p/.knudge/.idx/emb_cache.jsonl");
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("a", vec![1.0, 0.0], "modelo", 0);
    cache.save(&fs, path)?;
    let mut warnings = Vec::new();
    let mut loaded = EmbeddingCache::load(&fs, path, 1024, &mut warnings);
    assert_eq!(loaded.get("a", "modelo"), Some(vec![1.0_f32, 0.0]));
    assert!(warnings.is_empty());
    Ok(())
}

#[test]
fn prune_expired_removes_old_entries() {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("old", vec![1.0, 0.0], "modelo", 0);
    cache.insert("new", vec![0.0, 1.0], "modelo", 1800);
    let removed = cache.prune_expired(2000, 500);
    assert_eq!(removed, 1);
    assert!(cache.get("old", "modelo").is_none());
    assert!(cache.get("new", "modelo").is_some());
    assert_eq!(cache.prune_expired(2000, 0), 0);
}

/// A mesma `body_hash` em dois modelos convive: a chave é `(hash, model)` (D153).
#[test]
fn dedupes_by_hash_and_model() {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("hash", vec![1.0, 0.0], "modelo-a", 0);
    cache.insert("hash", vec![0.0, 1.0], "modelo-b", 0);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get("hash", "modelo-a"), Some(vec![1.0_f32, 0.0]));
    assert_eq!(cache.get("hash", "modelo-b"), Some(vec![0.0_f32, 1.0]));
}

/// O *lookup* é model-aware: entrada de modelo inativo vira *miss* (D153).
#[test]
fn ignores_inactive_model() {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("hash", vec![1.0, 0.0], "modelo-a", 0);
    assert!(cache.get("hash", "modelo-b").is_none());
    assert_eq!(cache.len(), 1);
}

/// Mesma chave com vetores divergentes: vence o `created_ms` maior (D153).
#[test]
fn same_key_resolves_by_created_ms() {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("hash", vec![1.0, 0.0], "modelo", 500);
    cache.insert("hash", vec![0.0, 1.0], "modelo", 100);
    assert_eq!(cache.get("hash", "modelo"), Some(vec![1.0_f32, 0.0]));
    cache.insert("hash", vec![0.5, 0.5], "modelo", 900);
    assert_eq!(cache.get("hash", "modelo"), Some(vec![0.5_f32, 0.5]));
}

/// Empate no `created_ms`: vence o vetor lexicograficamente maior (D153).
#[test]
fn tie_breaks_by_lexicographic_vector() {
    let mut cache = EmbeddingCache::new(1024);
    cache.insert("hash", vec![1.0, 0.0], "modelo", 0);
    cache.insert("hash", vec![0.5, 0.0], "modelo", 0);
    assert_eq!(cache.get("hash", "modelo"), Some(vec![1.0_f32, 0.0]));
    cache.insert("hash", vec![2.0, 0.0], "modelo", 0);
    assert_eq!(cache.get("hash", "modelo"), Some(vec![2.0_f32, 0.0]));
}

/// `merge=union` do git concatena dois JSONL; o loader dedupa por chave (D153).
#[test]
fn union_merge_dedupes_by_key() -> Result<()> {
    let mut left = EmbeddingCache::new(1024);
    left.insert("a", vec![1.0, 0.0], "modelo", 100);
    left.insert("b", vec![0.0, 1.0], "modelo", 100);
    let mut right = EmbeddingCache::new(1024);
    right.insert("a", vec![1.0, 0.0], "modelo", 100);
    right.insert("c", vec![1.0, 1.0], "modelo", 100);
    let merged = format!("{}{}", left.serialize()?, right.serialize()?);
    let cache = EmbeddingCache::parse(&merged, 1024)?;
    assert_eq!(cache.len(), 3);
    Ok(())
}
