//! Corpus de leitura única (E15-T02/O1).
//!
//! Cada comando costumava reler e reparsear `notas/` de 2 a 5 vezes (índice, grafo, corpos,
//! fila de embeddings). Este módulo lê as notas **uma vez** e deriva o índice de retrieval e o
//! grafo do **mesmo** vetor, sem clonar o corpus.
//!
//! É pura orquestração sobre portas (`Store` sobre uma [`crate::ports::Fs`]) — sem terminal,
//! relógio ou FS real. A ordem é determinística: `list_ids` ordena os ids e `Index::build`
//! reordena por `id`, então o resultado é byte-idêntico ao de duas leituras separadas.

use crate::Result;
use crate::graph::Graph;
use crate::retrieval::Index;
use crate::store::{Note, Store};

#[cfg(test)]
mod tests;

/// Notas + índice + grafo derivados de **uma** leitura do store.
#[derive(Debug, Clone)]
pub struct Corpus {
    /// Todas as notas legíveis, na ordem de [`Store::list_ids`].
    pub notes: Vec<Note>,
    /// Índice de retrieval (documentos ordenados por `id`).
    pub index: Index,
    /// Grafo de arestas, derivado das mesmas notas.
    pub graph: Graph,
}

impl Corpus {
    /// Lê todas as notas legíveis e deriva índice e grafo de uma só passada.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura (`Io`) e de extração de metadados (`Schema`).
    pub fn load(store: &Store<'_>) -> Result<Self> {
        let ids = store.list_ids()?;
        let mut notes = Vec::new();
        let _reserved = notes.try_reserve(ids.len());
        for id in &ids {
            if let Some(note) = store.read_optional(id)? {
                notes.push(note);
            }
        }
        Self::from_notes(notes)
    }

    /// Deriva índice e grafo de um vetor de notas já carregado.
    ///
    /// # Errors
    /// Propaga erros de extração de metadados (`Schema`).
    pub fn from_notes(notes: Vec<Note>) -> Result<Self> {
        let index = Index::build(&notes)?;
        let graph = Graph::from_notes_ref(&notes)?;
        Ok(Self {
            notes,
            index,
            graph,
        })
    }
}
