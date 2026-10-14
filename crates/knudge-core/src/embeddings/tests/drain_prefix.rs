//! Prefixos de tarefa aplicados no dreno (D216).

use std::sync::Mutex;

use crate::Result;
use crate::config::Config;
use crate::embeddings::pipeline::{DrainInput, drain};
use crate::embeddings::{EmbeddingMeta, Prefix, Similarity};
use crate::error::lock_or_recover;
use crate::ports::Embedder;
use crate::ports::fakes::MemFs;
use crate::store::Store;

use super::{NOW, note, seeded};

fn input<'a>(
    store: &'a Store<'a>,
    embedder: &'a dyn Embedder,
    config: &'a Config,
) -> DrainInput<'a> {
    DrainInput {
        store,
        embedder,
        config,
        now_ms: NOW,
    }
}

/// Embedder que grava os textos enviados ao provedor.
struct RecordingEmbedder {
    meta: EmbeddingMeta,
    texts: Mutex<Vec<String>>,
}

impl RecordingEmbedder {
    fn new(meta: EmbeddingMeta) -> Self {
        Self {
            meta,
            texts: Mutex::new(Vec::new()),
        }
    }

    fn texts(&self) -> Vec<String> {
        lock_or_recover(&self.texts).clone()
    }
}

impl Embedder for RecordingEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        lock_or_recover(&self.texts).extend(texts.iter().cloned());
        Ok(texts.iter().map(|_| vec![1.0_f32, 0.0, 0.0, 0.0]).collect())
    }
}

#[test]
fn drain_applies_embeddinggemma_document_prefix() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("afirmação curta", "corpo da nota")?];
    let store = seeded(&fs, &notes)?;
    let meta = EmbeddingMeta::new(
        "http",
        "google/embeddinggemma-2",
        "1",
        4,
        Similarity::Cosine,
        Prefix::EmbeddingGemma,
    )?;
    let embedder = RecordingEmbedder::new(meta);
    let config = Config::defaults();

    let _outcome = drain(&input(&store, &embedder, &config))?;
    let seen = embedder.texts();
    assert_eq!(seen.len(), 1);
    assert!(
        seen.first()
            .is_some_and(|text| text == "title: afirmação curta | text: corpo da nota")
    );
    Ok(())
}

#[test]
fn drain_without_prefix_sends_plain_text() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note("afirmação curta", "corpo da nota")?];
    let store = seeded(&fs, &notes)?;
    let meta = EmbeddingMeta::new(
        "http",
        "ibm-granite/granite-embedding-97m-multilingual-r2",
        "1",
        4,
        Similarity::Cosine,
        Prefix::None,
    )?;
    let embedder = RecordingEmbedder::new(meta);
    let config = Config::defaults();

    let _outcome = drain(&input(&store, &embedder, &config))?;
    let seen = embedder.texts();
    assert!(
        seen.first()
            .is_some_and(|text| text == "afirmação curta\ncorpo da nota")
    );
    Ok(())
}
