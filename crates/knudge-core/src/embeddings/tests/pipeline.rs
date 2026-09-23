//! Testes do worker de dreno (E11-T03/T04/T05/T06/T10).

use std::path::Path;

use crate::Result;
use crate::config::{Config, ConfigValue};
use crate::embeddings::pipeline::{DrainInput, drain, embedding_text};
use crate::embeddings::{EmbeddingIndex, EmbeddingMeta, EmbeddingState};
use crate::ports::fakes::{FakeEmbedder, MemFs};
use crate::ports::{Embedder, Fs};
use crate::store::Store;

use super::{ROOT, note, seeded};

fn config() -> Config {
    Config::defaults()
}

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

#[test]
fn embedding_text_joins_statement_and_body() {
    assert_eq!(embedding_text("afirmação", "corpo"), "afirmação\ncorpo");
    assert_eq!(embedding_text("afirmação", "  "), "afirmação");
}

#[test]
fn drains_pending_and_indexes_all() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo a")?, note("b", "corpo b")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let config = config();

    let outcome = drain(&input(&store, &embedder, &config))?;
    assert_eq!(outcome.indexed, 2);
    assert_eq!(outcome.pending, 0);
    assert!(outcome.warnings.is_empty());

    let mut warnings = Vec::new();
    let loaded = EmbeddingIndex::load(&fs, Path::new(ROOT), embedder.meta(), &mut warnings)?;
    assert_eq!(loaded.map(|index| index.len()), Some(2));
    Ok(())
}

#[test]
fn second_drain_is_noop() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let config = config();

    let _first = drain(&input(&store, &embedder, &config))?;
    let second = drain(&input(&store, &embedder, &config))?;
    assert_eq!(second.indexed, 0);
    assert_eq!(second.pending, 0);
    assert_eq!(embedder.calls(), vec![1]);
    Ok(())
}

#[test]
fn cache_hit_skips_provider() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let config = config();

    let _first = drain(&input(&store, &embedder, &config))?;
    fs.remove_file(&EmbeddingIndex::path(Path::new(ROOT)))?;
    let second = drain(&input(&store, &embedder, &config))?;
    assert_eq!(second.indexed, 1);
    assert_eq!(second.cache_hits, 1);
    assert_eq!(embedder.calls(), vec![1]);
    Ok(())
}

#[test]
fn provider_failure_keeps_pending() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("a", "corpo")?];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    embedder.fail_next(1);
    let config = config();

    let outcome = drain(&input(&store, &embedder, &config))?;
    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.pending, 1);
    assert!(!outcome.warnings.is_empty());

    let retry = drain(&input(&store, &embedder, &config))?;
    assert_eq!(retry.indexed, 1);
    assert_eq!(retry.pending, 0);
    Ok(())
}

#[test]
fn batch_limits_drain_when_not_backlogged() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![
        note("a", "corpo a")?,
        note("b", "corpo b")?,
        note("c", "corpo c")?,
    ];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let mut config = config();
    config.set_value("embeddings.batch", ConfigValue::Int(1))?;

    let outcome = drain(&input(&store, &embedder, &config))?;
    assert_eq!(outcome.indexed, 1);
    assert_eq!(outcome.pending, 2);
    Ok(())
}

#[test]
fn backpressure_forces_catch_up() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![
        note("a", "corpo a")?,
        note("b", "corpo b")?,
        note("c", "corpo c")?,
    ];
    let store = seeded(&fs, &notes)?;
    let embedder = FakeEmbedder::new(8)?;
    let mut config = config();
    config.set_value("embeddings.batch", ConfigValue::Int(1))?;
    config.set_value("embeddings.max_pending", ConfigValue::Int(1))?;

    let outcome = drain(&input(&store, &embedder, &config))?;
    assert_eq!(outcome.indexed, 3);
    assert!(outcome.warnings.iter().any(|w| w.contains("max_pending")));
    Ok(())
}

#[test]
fn stale_note_is_reindexed() -> Result<()> {
    let fs = MemFs::new();
    let note_a = note("a", "corpo")?;
    let id = note_a.id()?.to_string();
    let store = seeded(&fs, &[note_a])?;
    let embedder = FakeEmbedder::new(8)?;
    let config = config();

    let _first = drain(&input(&store, &embedder, &config))?;
    store.update(&id, |note| {
        note.body = "corpo novo".to_string();
        Ok(())
    })?;
    let second = drain(&input(&store, &embedder, &config))?;
    assert_eq!(second.indexed, 1);
    assert_eq!(second.stale, 1);

    let mut warnings = Vec::new();
    let index = EmbeddingIndex::load(&fs, Path::new(ROOT), embedder.meta(), &mut warnings)?
        .unwrap_or_else(|| EmbeddingIndex::new(embedder.meta().clone()));
    assert_eq!(index.state_of(&id, "h"), EmbeddingState::Stale);
    Ok(())
}

#[test]
fn non_embeddable_note_does_not_block_queue() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![
        note("boa", "corpo bom")?,
        note("ruim", "corpo que excede o contexto")?,
    ];
    let store = seeded(&fs, &notes)?;
    let embedder = RejectingEmbedder {
        inner: FakeEmbedder::new(8)?,
        marker: "excede",
    };
    let config = config();

    let outcome = drain(&input(&store, &embedder, &config))?;
    assert_eq!(outcome.indexed, 1);
    assert_eq!(outcome.pending, 1);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.contains("individualmente"))
    );
    Ok(())
}

/// Embedder que rejeita o lote inteiro quando qualquer texto contém `marker`, mas embute os
/// demais isoladamente — reproduz um provedor que estoura contexto em uma nota só.
struct RejectingEmbedder {
    inner: FakeEmbedder,
    marker: &'static str,
}

impl Embedder for RejectingEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        self.inner.meta()
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.iter().any(|text| text.contains(self.marker)) {
            return Err(crate::Error::internal("contexto excedido (teste)"));
        }
        self.inner.embed(texts)
    }
}

#[test]
fn removal_purges_vector() -> Result<()> {
    let fs = MemFs::new();
    let note_a = note("a", "corpo")?;
    let id = note_a.id()?.to_string();
    let store = seeded(&fs, &[note_a])?;
    let embedder = FakeEmbedder::new(8)?;
    let config = config();

    let _first = drain(&input(&store, &embedder, &config))?;
    store.remove(&id)?;
    let mut warnings = Vec::new();
    let index = EmbeddingIndex::load(&fs, Path::new(ROOT), embedder.meta(), &mut warnings)?
        .unwrap_or_else(|| EmbeddingIndex::new(embedder.meta().clone()));
    assert!(!index.contains(&id));
    Ok(())
}
