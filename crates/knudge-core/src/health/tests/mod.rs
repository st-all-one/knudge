//! Testes do escopo `health` (E09).

mod anchors;
mod audit;
mod doctor;
mod evidence;
mod tolerant;
mod validator;

use crate::Result;
use crate::graph::Graph;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{Classification, NoteType, Scope};
use crate::store::{EventLog, Note, Store};
use crate::write::{Draft, WriteContext};

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Raiz do projeto (âncoras são relativas a ela).
pub(super) const PROJECT: &str = "/p";

/// Raiz do `.knudge/`.
pub(super) const ROOT: &str = "/p/.knudge";

/// Nota simples com corpo.
pub(super) fn note(note_type: NoteType, statement: &str, body: &str) -> Result<Note> {
    Draft::new(note_type, statement)
        .with_body(body)
        .to_note(NOW)
}

/// Nota ancorada.
pub(super) fn anchored(statement: &str, anchors: &[&str], body: &str) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement).with_body(body);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.to_note(NOW)
}

/// Tarefa com checks e âncoras.
pub(super) fn task(statement: &str, checks: &[&str], anchors: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Task, statement);
    draft.scope = Some(Scope::Task);
    draft.checks = checks.iter().map(|check| (*check).to_string()).collect();
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.classification = Some(Classification::Tactical);
    draft.to_note(NOW)
}

/// Contexto de escrita sobre as notas semeadas em `ROOT`.
pub(super) fn seeded<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<WriteContext<'a>> {
    let store = Store::new(fs, ROOT);
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    let events = EventLog::new(fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let index = Index::build(notes)?;
    Ok(WriteContext::new(store, events, index, NOW))
}

/// Índice e grafo das notas.
pub(super) fn built(notes: &[Note]) -> Result<(Index, Graph)> {
    Ok((Index::build(notes)?, Graph::from_notes(notes.to_vec())?))
}
