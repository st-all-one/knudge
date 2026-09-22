//! Testes do handoff/rewind (E08).

mod budget;
mod context;
mod manifest;
mod rewind;
mod scope;

use crate::Result;
use crate::graph::Graph;
use crate::retrieval::Index;
use crate::schema::{Classification, EdgeKind, NoteType, Scope, Value};
use crate::store::Note;
use crate::write::Draft;

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Nota simples.
pub(super) fn note(note_type: NoteType, statement: &str) -> Result<Note> {
    Draft::new(note_type, statement).to_note(NOW)
}

/// Nota com classificação.
pub(super) fn classified(
    note_type: NoteType,
    statement: &str,
    classification: Classification,
) -> Result<Note> {
    let mut draft = Draft::new(note_type, statement);
    draft.classification = Some(classification);
    draft.to_note(NOW)
}

/// Nota com âncoras.
pub(super) fn anchored(statement: &str, anchors: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.to_note(NOW)
}

/// Nota com `outcomes` de sucesso (vira `star`).
pub(super) fn confirmed(statement: &str) -> Result<Note> {
    let mut note = Draft::new(NoteType::Fact, statement).to_note(NOW)?;
    note.frontmatter.set(
        "outcomes",
        Value::List(vec![Value::map([(
            "status".to_string(),
            Value::Str("success".to_string()),
        )])]),
    )?;
    Ok(note)
}

/// Container `plan`.
pub(super) fn container(statement: &str) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Container, statement);
    draft.scope = Some(Scope::Plan);
    draft.to_note(NOW)
}

/// Nota ancorada que depende de `depends`.
pub(super) fn member(statement: &str, anchors: &[&str], depends: &str) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.edges = vec![(EdgeKind::DependsOn, depends.to_string())];
    draft.to_note(NOW)
}

/// Índice e grafo derivados das notas.
pub(super) fn built(notes: &[Note]) -> Result<(Index, Graph)> {
    let index = Index::build(notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    Ok((index, graph))
}
