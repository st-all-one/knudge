//! Testes do retrieval (E06).

use crate::Result;
use crate::schema::{Frontmatter, NoteType, Scope, Status, Value, body, id};
use crate::store::Note;

mod anchor;
mod bm25;
mod filter;
mod index;
mod rank;
mod recall;
mod rrf;
mod tags;
mod token;
mod views;

/// Frontmatter mínimo válido.
///
/// `Task` exige `scope` (D93/D113), então já o define; assim as views o enxergam como item de
/// trabalho. `Error`/`Question`/`Risk`/`Decision` **não** ganham `scope` (continuam conhecimento).
pub(super) fn base(note_type: NoteType, statement: &str) -> Result<Frontmatter> {
    let mut frontmatter = Frontmatter::new();
    frontmatter.set("id", Value::Str(id::note_id(note_type, statement)))?;
    if !note_type.is_group() {
        frontmatter.set("type", Value::Str(note_type.as_str().to_string()))?;
    }
    frontmatter.set("statement", Value::Str(statement.to_string()))?;
    match note_type {
        NoteType::Task => {
            frontmatter.set("scope", Value::Str(Scope::Task.as_str().to_string()))?;
        }
        NoteType::Epic => {
            frontmatter.set("scope", Value::Str(Scope::Epic.as_str().to_string()))?;
        }
        _ => {}
    }
    frontmatter.set("created_at", Value::Int(1_700_000_000_000))?;
    frontmatter.set("body_hash", Value::Str(body::body_hash(statement, "")))?;
    frontmatter.set("schema_version", Value::Int(1))?;
    Ok(frontmatter)
}

/// Nota com corpo (hash recalculado).
pub(super) fn note(note_type: NoteType, statement: &str, text: &str) -> Result<Note> {
    let mut note = Note::new(base(note_type, statement)?, text);
    note.refresh_body_hash()?;
    Ok(note)
}

/// Nota com tags.
pub(super) fn tagged(note_type: NoteType, statement: &str, tags: &[&str]) -> Result<Note> {
    let mut note = Note::new(with_tags(base(note_type, statement)?, tags)?, "");
    note.refresh_body_hash()?;
    Ok(note)
}

/// Nota com âncoras.
pub(super) fn anchored(note_type: NoteType, statement: &str, anchors: &[&str]) -> Result<Note> {
    let mut note = Note::new(with_anchors(base(note_type, statement)?, anchors)?, "");
    note.refresh_body_hash()?;
    Ok(note)
}

/// Aplica `tags`.
pub(super) fn with_tags(mut frontmatter: Frontmatter, tags: &[&str]) -> Result<Frontmatter> {
    frontmatter.set("tags", string_list(tags))?;
    Ok(frontmatter)
}

/// Aplica `anchors`.
pub(super) fn with_anchors(mut frontmatter: Frontmatter, anchors: &[&str]) -> Result<Frontmatter> {
    frontmatter.set("anchors", string_list(anchors))?;
    Ok(frontmatter)
}

/// Aplica `status`.
pub(super) fn with_status(mut frontmatter: Frontmatter, status: Status) -> Result<Frontmatter> {
    frontmatter.set("status", Value::Str(status.as_str().to_string()))?;
    Ok(frontmatter)
}

/// Aplica `scope` (nível de tarefa — D113).
pub(super) fn with_scope(mut frontmatter: Frontmatter, scope: Scope) -> Result<Frontmatter> {
    frontmatter.set("scope", Value::Str(scope.as_str().to_string()))?;
    Ok(frontmatter)
}

/// Aplica `outcomes` com os status dados.
pub(super) fn with_outcomes(
    mut frontmatter: Frontmatter,
    statuses: &[&str],
) -> Result<Frontmatter> {
    let items = statuses
        .iter()
        .map(|status| Value::map([("status".to_string(), Value::Str((*status).to_string()))]))
        .collect();
    frontmatter.set("outcomes", Value::List(items))?;
    Ok(frontmatter)
}

fn string_list(items: &[&str]) -> Value {
    Value::List(
        items
            .iter()
            .map(|item| Value::Str((*item).to_string()))
            .collect(),
    )
}
