//! Testes da escrita (E07).

mod dedup;
mod idempotent;
mod lifecycle;
mod reconcile;
mod strict;
mod update;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::NoteType;
use crate::store::{EventLog, Note, Store};
use crate::write::{Draft, WriteContext};

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Nota pronta para semear o store.
pub(super) fn note(note_type: NoteType, statement: &str, body: &str) -> Result<Note> {
    Draft::new(note_type, statement)
        .with_body(body)
        .to_note(NOW)
}

/// Contexto de escrita sobre as notas semeadas.
pub(super) fn seeded<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<WriteContext<'a>> {
    let store = Store::new(fs, "/p/.knudge");
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    let events = EventLog::new(fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    let index = Index::build(notes)?;
    Ok(WriteContext::new(store, events, index, NOW))
}
