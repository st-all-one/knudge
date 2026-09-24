//! Testes do store (E03) com fakes determinísticos.

mod commit;
mod events;
mod lock;
mod note;
mod purge;
mod rebuild;
mod sweep;

use crate::Result;
use crate::schema::{Frontmatter, NoteType, Value, id};
use crate::store::Note;

/// Nota válida para testes (id endereçado por conteúdo, campos obrigatórios).
fn sample_note(statement: &str) -> Result<Note> {
    let note_type = NoteType::Fact;
    let mut frontmatter = Frontmatter::new();
    frontmatter.set("id", Value::Str(id::note_id(note_type, statement)))?;
    frontmatter.set("type", Value::Str(note_type.as_str().to_string()))?;
    frontmatter.set("statement", Value::Str(statement.to_string()))?;
    frontmatter.set("created_at", Value::Int(1_700_000_000_000))?;
    frontmatter.set("schema_version", Value::Int(1))?;
    let mut note = Note::new(frontmatter, "corpo");
    note.refresh_body_hash()?;
    Ok(note)
}
