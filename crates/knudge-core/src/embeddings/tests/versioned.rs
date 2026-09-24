//! Testes do cache vetorial versionado (D148).

use std::path::Path;

use crate::Result;
use crate::config::{Config, ConfigValue};
use crate::embeddings::pipeline::{DrainInput, drain};
use crate::embeddings::{EmbeddingCache, EmbeddingIndex};
use crate::ports::Embedder;
use crate::ports::Fs;
use crate::ports::fakes::{FakeEmbedder, MemFs};
use crate::store::Store;

use super::{ROOT, note, seeded};

fn input<'a>(
    store: &'a Store<'a>,
    embedder: &'a dyn Embedder,
    config: &'a Config,
) -> DrainInput<'a> {
    DrainInput {
        store,
        embedder,
        config,
        now_ms: super::NOW,
    }
}

/// Com `version_cache=true`, um clone (índice apagado, cache presente) reindexa **sem**
/// inferência e o cache mora fora do `.idx/` (D148).
#[test]
fn version_cache_rebuilds_index_without_inference() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let mut config = Config::defaults();
    config.set_value("embeddings.version_cache", ConfigValue::Bool(true))?;

    let first = drain(&input(&store, &embedder, &config))?;
    assert_eq!(first.indexed, 1);
    assert!(fs.exists(Path::new("/p/.knudge/emb_cache.jsonl")));
    assert!(!fs.exists(Path::new("/p/.knudge/.idx/emb_cache.jsonl")));

    fs.remove_file(&EmbeddingIndex::path(Path::new(ROOT)))?;
    let second = drain(&input(&store, &embedder, &config))?;
    assert_eq!(second.indexed, 1);
    assert_eq!(second.cache_hits, 1);
    assert_eq!(embedder.calls(), vec![1], "clone re-embeddou (D148)");
    Ok(())
}

/// O cache versionado ignora o teto (sem eviction), emitindo só um aviso (D148).
#[test]
fn version_cache_does_not_evict() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo a")?, note("b", "corpo b")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let mut config = Config::defaults();
    config.set_value("embeddings.version_cache", ConfigValue::Bool(true))?;
    config.set_value("embeddings.cache_max_bytes", ConfigValue::Int(1))?;

    let _first = drain(&input(&store, &embedder, &config))?;
    let second = drain(&input(&store, &embedder, &config))?;
    assert!(
        second.warnings.iter().any(|w| w.contains("versionado")),
        "aviso de teto soft ausente: {:?}",
        second.warnings
    );
    let mut warnings = Vec::new();
    let cache = EmbeddingCache::load(
        &fs,
        Path::new("/p/.knudge/emb_cache.jsonl"),
        usize::MAX,
        &mut warnings,
    );
    assert_eq!(cache.len(), 2, "eviction apagou o cache versionado");
    Ok(())
}
