//! Índice derivado: rebuild determinístico e carga tolerante (E06-T01).

use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::retrieval::{INDEX_FILE, INDEX_WARN_BYTES, Index, size_warning};
use crate::schema::{NoteType, Scope};
use crate::store::{Note, Store};

use super::{base, note, with_scope};

#[test]
fn rebuild_is_byte_for_byte() -> Result<()> {
    let notes = vec![
        note(NoteType::Fact, "alpha beta", "corpo alpha")?,
        note(NoteType::Decision, "gama delta", "")?,
    ];
    let first = Index::build(&notes)?;
    let bytes = first.serialize()?;

    let loaded = Index::parse(&bytes)?;
    assert_eq!(loaded, first);
    assert_eq!(loaded.serialize()?, bytes);
    Ok(())
}

#[test]
fn scope_survives_round_trip() -> Result<()> {
    let task = Note::new(
        with_scope(base(NoteType::Task, "tarefa")?, Scope::Task)?,
        "",
    );
    let index = Index::build(&[task])?;
    let loaded = Index::parse(&index.serialize()?)?;
    assert_eq!(loaded, index);
    assert_eq!(
        loaded.docs.first().and_then(|doc| doc.meta.scope),
        Some(Scope::Task)
    );
    Ok(())
}

#[test]
fn open_rebuilds_when_absent() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    store.ensure_dirs()?;
    store.write(&note(NoteType::Fact, "alpha", "corpo")?)?;

    let (index, warnings) = Index::open(&fs, store.root(), &store)?;
    assert_eq!(index.docs.len(), 1);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("reconstruído"))
    );
    assert!(fs.exists(&Index::path(store.root())));

    let (again, warnings) = Index::open(&fs, store.root(), &store)?;
    assert_eq!(again, index);
    assert!(warnings.is_empty());
    Ok(())
}

#[test]
fn index_file_lives_in_idx_dir() {
    let path = Index::path(std::path::Path::new("/p/.knudge"));
    assert!(path.ends_with(INDEX_FILE));
    assert!(path.to_string_lossy().contains(".idx"));
}

#[test]
fn size_warning_only_above_threshold() {
    let limit = usize::try_from(INDEX_WARN_BYTES).unwrap_or(usize::MAX);
    assert!(size_warning(0).is_none());
    assert!(size_warning(limit).is_none());
    assert!(size_warning(limit.saturating_add(1)).is_some());
}
