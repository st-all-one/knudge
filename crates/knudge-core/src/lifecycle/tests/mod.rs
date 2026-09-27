//! Testes do escopo `lifecycle` (E09/E10).

mod beta;
mod clusters;
mod communities;
mod confidence;
mod decay;
mod drift;
mod from_tasks;
mod plan;
mod retention;
mod retire;
mod semantic;
mod shelf_life;
mod supersession;
mod term_drift;

use crate::Result;
use crate::graph::Graph;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{Classification, NoteType, Value};
use crate::store::{EventLog, Note, Store};
use crate::time::Timestamp;
use crate::write::{Draft, WriteContext};

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Raiz do `.knudge/`.
pub(super) const ROOT: &str = "/p/.knudge";

/// Raiz do projeto (âncoras relativas).
pub(super) const PROJECT: &str = "/p";

/// Nota criada em `created_ms`.
pub(super) fn note_created(
    note_type: NoteType,
    statement: &str,
    body: &str,
    created_ms: i64,
) -> Result<Note> {
    Draft::new(note_type, statement)
        .with_body(body)
        .to_note(created_ms)
}

/// Nota com classificação explícita criada em `created_ms`.
pub(super) fn classified(
    statement: &str,
    classification: Classification,
    created_ms: i64,
) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.classification = Some(classification);
    draft.to_note(created_ms)
}

/// Nota com âncoras.
pub(super) fn anchored(statement: &str, anchors: &[&str], created_ms: i64) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.to_note(created_ms)
}

/// Nota com `outcomes` datados (`recorded_at` em ms) — revisão espaçada (D190).
pub(super) fn with_outcomes_at(mut note: Note, statuses: &[(&str, i64)]) -> Result<Note> {
    let items = statuses
        .iter()
        .map(|(status, at)| {
            Value::map([
                ("status".to_string(), Value::Str((*status).to_string())),
                (
                    "recorded_at".to_string(),
                    Value::Str(Timestamp::from_millis(*at).to_rfc3339()),
                ),
            ])
        })
        .collect();
    note.frontmatter.set("outcomes", Value::List(items))?;
    Ok(note)
}

/// Contexto de escrita sobre as notas semeadas em `ROOT`.
pub(super) fn seeded<'a>(fs: &'a MemFs, notes: &[Note], now_ms: i64) -> Result<WriteContext<'a>> {
    let store = Store::new(fs, ROOT);
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    let events = EventLog::new(fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let index = Index::build(notes)?;
    Ok(WriteContext::new(store, events, index, now_ms))
}

/// Índice e grafo das notas.
pub(super) fn built(notes: &[Note]) -> Result<(Index, Graph)> {
    Ok((Index::build(notes)?, Graph::from_notes(notes.to_vec())?))
}
