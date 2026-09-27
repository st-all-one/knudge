//! Construção de nós e arestas explícitas (E05/D49).
//!
//! Extraído de `graph/mod.rs` para manter o arquivo abaixo do teto. `insert_note` projeta uma
//! nota num [`Node`](super::Node); [`link`] adiciona uma aresta ao frontmatter sem duplicar.

use std::collections::BTreeMap;

use super::Node;
use crate::schema::{EdgeKind, Frontmatter, Value, id};
use crate::store::Note;
use crate::{Error, Result};

/// Insere um nó no mapa do grafo a partir de uma nota (compartilhado por
/// [`Graph::from_notes`](super::Graph::from_notes)/
/// [`Graph::from_notes_ref`](super::Graph::from_notes_ref)).
pub(super) fn insert_note(by_id: &mut BTreeMap<String, Node>, note: &Note) -> Result<()> {
    let frontmatter = &note.frontmatter;
    let id = note.id()?.to_string();
    let note_type = frontmatter.note_type()?;
    let scope = frontmatter.scope()?;
    let status = frontmatter.status()?;
    let superseded_by = match frontmatter.get("superseded_by") {
        Some(Value::Str(target)) => Some(target.clone()),
        _ => None,
    };
    let mut edges = BTreeMap::new();
    for kind in EdgeKind::ALL {
        let targets: Vec<String> = frontmatter
            .string_list(kind.key())?
            .into_iter()
            .map(str::to_string)
            .collect();
        if !targets.is_empty() {
            edges.insert(kind, targets);
        }
    }
    by_id.insert(
        id.clone(),
        Node {
            id,
            note_type,
            scope,
            status,
            superseded_by,
            edges,
        },
    );
    Ok(())
}

/// Adiciona uma aresta explícita ao frontmatter, sem duplicar (D49).
///
/// Retorna `true` se o frontmatter mudou. A fonte da verdade continua sendo a nota; o
/// [`Graph`](super::Graph) é reconstruído a partir dela.
///
/// # Errors
/// Retorna `ErrorKind::Schema` para destino inválido ou auto-aresta.
pub fn link(frontmatter: &mut Frontmatter, kind: EdgeKind, to: &str) -> Result<bool> {
    if !id::is_valid_note_id(to) {
        return Err(Error::schema(format!("destino de aresta inválido: {to:?}")));
    }
    if frontmatter.id().is_ok_and(|from| from == to) {
        return Err(Error::schema("auto-aresta não é permitida"));
    }
    let mut targets: Vec<String> = frontmatter
        .string_list(kind.key())?
        .into_iter()
        .map(str::to_string)
        .collect();
    if targets.iter().any(|target| target == to) {
        return Ok(false);
    }
    targets.push(to.to_string());
    let value = Value::List(targets.into_iter().map(Value::Str).collect());
    frontmatter.set(kind.key(), value)?;
    Ok(true)
}
