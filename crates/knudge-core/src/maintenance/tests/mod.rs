//! Testes de manutenção: `diff`, `learn`, `compact` (E08).

mod compact;
mod diff;
mod learn;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{NoteType, Scope, Value};
use crate::store::{EventLog, Note, Store};
use crate::time::Timestamp;
use crate::write::{Draft, WriteContext};

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Nota simples.
pub(super) fn note(note_type: NoteType, statement: &str) -> Result<Note> {
    Draft::new(note_type, statement).to_note(NOW)
}

/// Nota com âncoras.
pub(super) fn anchored(statement: &str, anchors: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.to_note(NOW)
}

/// Tarefa com `outcomes` de sucesso e âncoras (X1/D108).
pub(super) fn success_task(statement: &str, anchors: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Task, statement);
    draft.scope = Some(Scope::Task);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    let mut note = draft.to_note(NOW)?;
    let entry = Value::map([
        ("status".to_string(), Value::Str("success".to_string())),
        (
            "recorded_at".to_string(),
            Value::Str(Timestamp::from_millis(NOW).to_rfc3339()),
        ),
    ]);
    note.frontmatter.set("outcomes", Value::List(vec![entry]))?;
    note.refresh_body_hash()?;
    Ok(note)
}

/// Contexto de escrita sobre as notas semeadas.
pub(super) fn context<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<WriteContext<'a>> {
    let store = Store::new(fs, "/p/.knudge");
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    let events = EventLog::new(fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    let index = Index::build(notes)?;
    Ok(WriteContext::new(store, events, index, NOW))
}
