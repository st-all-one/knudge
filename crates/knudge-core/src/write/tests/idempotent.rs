//! `write` idempotente por conteúdo (E07-T01).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::NoteType;
use crate::store::{EventLog, Store};
use crate::write::{DedupThresholds, Draft, WriteAction, WriteContext, write};

use super::NOW;

fn context(fs: &MemFs) -> Result<WriteContext<'_>> {
    let store = Store::new(fs, "/p/.knudge");
    store.ensure_dirs()?;
    let events = EventLog::new(fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    Ok(WriteContext::new(store, events, Index::build(&[])?, NOW))
}

#[test]
fn repeated_write_creates_one_note() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let thresholds = DedupThresholds::default();
    let draft = Draft::new(NoteType::Fact, "alpha");

    let first = write(&ctx, &draft, &thresholds)?;
    assert_eq!(first.action, WriteAction::Created);
    assert_eq!(first.revision, Some(1));

    let second = write(&ctx, &draft, &thresholds)?;
    assert_eq!(second.action, WriteAction::Unchanged);
    assert_eq!(second.id, first.id);
    assert_eq!(ctx.store().list_ids()?.len(), 1);

    let (records, warnings) = ctx.events().read_all()?;
    assert!(warnings.is_empty());
    assert!(
        records
            .iter()
            .any(|record| record.note_id.as_deref() == Some(first.id.as_str()))
    );
    Ok(())
}

#[test]
fn same_key_different_body_conflicts() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let thresholds = DedupThresholds::default();

    let first = write(
        &ctx,
        &Draft::new(NoteType::Fact, "alpha").with_body("um"),
        &thresholds,
    )?;
    let error = write(
        &ctx,
        &Draft::new(NoteType::Fact, "alpha").with_body("dois"),
        &thresholds,
    );
    assert!(error.is_err());
    assert_eq!(ctx.store().list_ids()?.len(), 1);
    assert_eq!(first.action, WriteAction::Created);
    Ok(())
}

#[test]
fn write_rejects_task_and_container() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let thresholds = DedupThresholds::default();
    let draft = Draft::new(NoteType::Task, "tarefa");
    assert!(write(&ctx, &draft, &thresholds).is_err());
    Ok(())
}
