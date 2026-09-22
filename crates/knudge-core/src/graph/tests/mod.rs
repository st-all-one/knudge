//! Testes do grafo e das arestas (E05).

use crate::Result;
use crate::graph::link;
use crate::schema::{EdgeKind, Frontmatter, NoteType, Value, body, id};
use crate::store::Note;

mod cycles;
mod extract;
mod graph;
mod integrity;
mod suggestions;

/// Frontmatter mínimo válido (sem corpo).
pub(super) fn frontmatter(note_type: NoteType, statement: &str) -> Result<Frontmatter> {
    let mut fm = Frontmatter::new();
    fm.set("id", Value::Str(id::note_id(note_type, statement)))?;
    fm.set("type", Value::Str(note_type.as_str().to_string()))?;
    fm.set("statement", Value::Str(statement.to_string()))?;
    fm.set(
        "created_at",
        Value::Str("2026-01-02T03:04:05.678Z".to_string()),
    )?;
    fm.set("confidence", Value::Float(0.7))?;
    fm.set("body_hash", Value::Str(body::body_hash(statement, "")))?;
    fm.set("schema_version", Value::Int(1))?;
    Ok(fm)
}

/// Nota com arestas explícitas.
pub(super) fn note(
    note_type: NoteType,
    statement: &str,
    edges: &[(EdgeKind, &str)],
) -> Result<Note> {
    let mut fm = frontmatter(note_type, statement)?;
    for (kind, target) in edges {
        link(&mut fm, *kind, target)?;
    }
    Ok(Note::new(fm, ""))
}

/// Aplica `superseded_by` ao frontmatter.
pub(super) fn with_superseded(mut fm: Frontmatter, successor: &str) -> Result<Frontmatter> {
    fm.set("superseded_by", Value::Str(successor.to_string()))?;
    Ok(fm)
}

/// Adiciona uma aresta direta ao frontmatter, sem validar contra auto-aresta (testes adversariais).
pub(super) fn force_edge(fm: &mut Frontmatter, kind: EdgeKind, target: &str) -> Result<()> {
    fm.set(
        kind.key(),
        Value::List(vec![Value::Str(target.to_string())]),
    )
}
