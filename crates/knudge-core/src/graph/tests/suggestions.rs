//! Armazenamento derivado de sugestões (E05-T02 / D50/D84).

use std::path::Path;

use crate::Result;
use crate::graph::{Suggestion, SuggestionStore};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::schema::{EdgeKind, NoteType, id};
use crate::store::purge_derived;

fn suggestion(kind: EdgeKind, target: &str, reason: &str) -> Suggestion {
    Suggestion {
        kind,
        target: target.to_string(),
        reason: reason.to_string(),
    }
}

#[test]
fn writes_lists_and_replaces() -> Result<()> {
    let fs = MemFs::new();
    let store = SuggestionStore::new(&fs, "/p/.knudge");
    let source = id::note_id(NoteType::Decision, "S");
    let target = id::note_id(NoteType::Fact, "T");

    store.write(
        &source,
        &[suggestion(EdgeKind::References, &target, "menção")],
    )?;
    let records = store.list()?;
    assert_eq!(records.len(), 1);
    assert_eq!(
        records.first().map(|record| record.targets.clone()),
        Some(vec![target.clone()])
    );

    store.write(&source, &[suggestion(EdgeKind::Supports, &target, "verbo")])?;
    let records = store.list()?;
    assert_eq!(records.len(), 1);
    assert_eq!(
        records.first().map(|record| record.kind),
        Some(EdgeKind::Supports)
    );

    store.write(&source, &[])?;
    assert!(store.list()?.is_empty());
    assert!(!fs.exists(&store.path()));
    Ok(())
}

#[test]
fn purge_derived_prunes_sources_and_targets() -> Result<()> {
    let fs = MemFs::new();
    let root = "/p/.knudge";
    let store = SuggestionStore::new(&fs, root);
    let source = id::note_id(NoteType::Decision, "S");
    let target = id::note_id(NoteType::Fact, "T");

    store.write(
        &source,
        &[suggestion(EdgeKind::References, &target, "menção")],
    )?;

    purge_derived(&fs, Path::new(root), &target)?;
    let records = store.list()?;
    assert_eq!(records.len(), 1);
    assert!(
        records
            .first()
            .is_some_and(|record| record.targets.is_empty())
    );

    purge_derived(&fs, Path::new(root), &source)?;
    assert!(store.list()?.is_empty());
    Ok(())
}
