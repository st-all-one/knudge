//! Testes do índice vetorial (E11-T01/T03/T05).

use std::path::Path;

use crate::Result;
use crate::embeddings::{EmbeddingIndex, EmbeddingMeta, EmbeddingState, Similarity};
use crate::ports::fakes::MemFs;

fn meta() -> Result<EmbeddingMeta> {
    EmbeddingMeta::new("http", "modelo", "1", 3, Similarity::Cosine)
}

#[test]
fn round_trip_preserves_entries_and_state() -> Result<()> {
    let meta = meta()?;
    let mut index = EmbeddingIndex::new(meta.clone());
    index.insert("a", "h1", vec![1.0, 0.0, 0.0]);
    let text = index.serialize()?;
    let Some(parsed) = EmbeddingIndex::parse(&text, &meta)? else {
        return Err(crate::Error::internal("índice deveria ser compatível"));
    };
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed.body_hash("a"), Some("h1"));
    assert_eq!(parsed.state_of("a", "h1"), EmbeddingState::Indexed);
    assert_eq!(parsed.state_of("a", "h2"), EmbeddingState::Stale);
    assert_eq!(parsed.state_of("b", "h1"), EmbeddingState::Pending);
    Ok(())
}

#[test]
fn model_change_invalidates_index() -> Result<()> {
    let original = meta()?;
    let mut index = EmbeddingIndex::new(original.clone());
    index.insert("a", "h1", vec![1.0, 0.0, 0.0]);
    let text = index.serialize()?;
    let changed = EmbeddingMeta::new("http", "outro", "1", 3, Similarity::Cosine)?;
    assert!(EmbeddingIndex::parse(&text, &changed)?.is_none());
    assert!(EmbeddingIndex::parse(&text, &original)?.is_some());
    Ok(())
}

#[test]
fn purge_removes_entry() -> Result<()> {
    let meta = meta()?;
    let mut index = EmbeddingIndex::new(meta);
    index.insert("a", "h1", vec![1.0, 0.0, 0.0]);
    assert!(index.purge("a"));
    assert!(!index.purge("a"));
    assert!(index.is_empty());
    Ok(())
}

#[test]
fn load_reports_invalid_header_as_warning() -> Result<()> {
    let fs = MemFs::new();
    let meta = meta()?;
    let mut warnings = Vec::new();
    let index = EmbeddingIndex::load(&fs, Path::new("/p/.knudge"), &meta, &mut warnings)?;
    assert!(index.is_none());
    assert!(warnings.is_empty());

    let path = EmbeddingIndex::path(Path::new("/p/.knudge"));
    fs.insert(&path, "{lixo");
    let mut warnings = Vec::new();
    let index = EmbeddingIndex::load(&fs, Path::new("/p/.knudge"), &meta, &mut warnings)?;
    assert!(index.is_none());
    assert!(!warnings.is_empty());
    Ok(())
}

#[test]
fn save_then_load_round_trip() -> Result<()> {
    let fs = MemFs::new();
    let meta = meta()?;
    let mut index = EmbeddingIndex::new(meta.clone());
    index.insert("a", "h1", vec![1.0, 0.0, 0.0]);
    let mut warnings = Vec::new();
    index.save(&fs, Path::new("/p/.knudge"), &mut warnings)?;
    let mut warnings = Vec::new();
    let loaded = EmbeddingIndex::load(&fs, Path::new("/p/.knudge"), &meta, &mut warnings)?;
    assert_eq!(loaded.map(|index| index.len()), Some(1));
    Ok(())
}
