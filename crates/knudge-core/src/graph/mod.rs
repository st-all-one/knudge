//! Escopo `graph`: arestas explícitas, integridade e ciclos (E05).
//!
//! O grafo é uma **projeção** das notas: a fonte da verdade continua sendo
//! `notas/<id>.md` (D49). Arestas explícitas vêm do frontmatter; a extração textual é
//! sugestão revisável em `.idx/suggestions.jsonl` e **nunca** entra no grafo (D49/D50).

pub mod cycles;
pub mod extract;
pub mod integrity;
pub mod suggestions;

#[cfg(test)]
mod tests;

pub use cycles::{cyclic_components, strongly_connected};
pub use extract::{Suggestion, extract};
pub use integrity::{Issue, IssueKind};
pub use suggestions::{SuggestionRecord, SuggestionStore};

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::{EdgeKind, Frontmatter, NoteType, Status, Value, id};
use crate::store::{Note, Store};
use crate::{Error, Result};

/// Nó do grafo — projeção do frontmatter.
#[derive(Debug, Clone)]
struct Node {
    id: String,
    note_type: NoteType,
    status: Status,
    superseded_by: Option<String>,
    /// Arestas de saída, por tipo (só tipos com alvos presentes).
    edges: BTreeMap<EdgeKind, Vec<String>>,
}

/// Grafo de arestas explícitas.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: BTreeMap<String, Node>,
}

/// Hit de expansão BFS determinística.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandHit {
    /// Id alcançado.
    pub id: String,
    /// Tipo da aresta percorrida.
    pub kind: EdgeKind,
    /// Distância a partir da raiz (≥ 1).
    pub depth: u32,
}

impl Graph {
    /// Constrói o grafo a partir de notas já parseadas.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se um frontmatter for inválido.
    pub fn from_notes(notes: impl IntoIterator<Item = Note>) -> Result<Self> {
        let mut by_id = BTreeMap::new();
        for note in notes {
            let frontmatter = &note.frontmatter;
            let id = note.id()?.to_string();
            let note_type = frontmatter.note_type()?;
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
                    status,
                    superseded_by,
                    edges,
                },
            );
        }
        Ok(Self { nodes: by_id })
    }

    /// Constrói o grafo lendo todas as notas do store.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse.
    pub fn build(store: &Store<'_>) -> Result<Self> {
        let mut notes = Vec::new();
        for id in store.list_ids()? {
            notes.push(store.read(&id)?);
        }
        Self::from_notes(notes)
    }

    /// `true` se o nó existe.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }

    /// Ids dos nós, ordenados.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.nodes.keys().map(String::as_str).collect()
    }

    /// Tipo da nota.
    #[must_use]
    pub fn note_type(&self, id: &str) -> Option<NoteType> {
        self.nodes.get(id).map(|node| node.note_type)
    }

    /// Status da nota.
    #[must_use]
    pub fn status(&self, id: &str) -> Option<Status> {
        self.nodes.get(id).map(|node| node.status)
    }

    /// Sucessor (`superseded_by`) da nota.
    #[must_use]
    pub fn superseded_by(&self, id: &str) -> Option<&str> {
        self.nodes
            .get(id)
            .and_then(|node| node.superseded_by.as_deref())
    }

    /// Alvos de uma aresta de saída.
    #[must_use]
    pub fn targets(&self, id: &str, kind: EdgeKind) -> &[String] {
        self.nodes
            .get(id)
            .and_then(|node| node.edges.get(&kind))
            .map_or(&[][..], Vec::as_slice)
    }

    /// Expansão BFS determinística até `depth` (arestas explícitas apenas).
    #[must_use]
    pub fn expand(&self, id: &str, kind: Option<EdgeKind>, depth: u32) -> Vec<ExpandHit> {
        let kinds: Vec<EdgeKind> = match kind {
            Some(kind) => vec![kind],
            None => EdgeKind::ALL.to_vec(),
        };
        let mut visits: BTreeSet<String> = BTreeSet::new();
        visits.insert(id.to_string());
        let mut frontier = vec![id.to_string()];
        let mut hits = Vec::new();
        for level in 1..=depth {
            let mut level_seen: BTreeSet<String> = BTreeSet::new();
            let mut next: BTreeSet<String> = BTreeSet::new();
            for current in &frontier {
                for kind in &kinds {
                    for target in self.targets(current, *kind) {
                        if visits.contains(target) || !level_seen.insert(target.clone()) {
                            continue;
                        }
                        next.insert(target.clone());
                        hits.push(ExpandHit {
                            id: target.clone(),
                            kind: *kind,
                            depth: level,
                        });
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            visits.extend(next.iter().cloned());
            frontier = next.into_iter().collect();
        }
        hits
    }
}

/// Adiciona uma aresta explícita ao frontmatter, sem duplicar (D49).
///
/// Retorna `true` se o frontmatter mudou. A fonte da verdade continua sendo a nota; o
/// [`Graph`] é reconstruído a partir dela.
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
