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
use crate::ports::Fs;
use crate::retrieval::Index;
use crate::store::{Note, Store};
use std::path::Path;

#[cfg(test)]
mod tests;

/// Abaixo deste tamanho o custo de `spawn` domina e a leitura fica sequencial (T12.5/O1).
const PARALLEL_MIN_NOTES: usize = 256;

/// Teto de threads: acima disso o ganho é marginal e o `spawn` pesa (máquinas muitos-núcleos).
const PARALLEL_MAX: usize = 16;

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
    /// Lê todas as notas legíveis numa só passada, **sem** derivar índice/grafo.
    ///
    /// Para corpora grandes, a leitura+parse é paralelizada por `std::thread::scope` (zero-dep,
    /// T12.5): os ids são divididos em faixas contíguas e os resultados remontados **na ordem de
    /// `list_ids`**, então a saída é idêntica à sequencial (determinismo preservado).
    ///
    /// Use quando o comando só precisa dos frontmatters e o grafo é opcional (E15-T20/O8.1).
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura (`Io`).
    pub fn load_notes(store: &Store<'_>) -> Result<Vec<Note>> {
        let ids = store.list_ids()?;
        let width = parallel_width(ids.len());
        if width <= 1 {
            return load_notes_range(store, &ids, 0, ids.len());
        }
        let chunk = ids.len().div_ceil(width);
        let ids_slice: &[String] = &ids;
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(width);
            let mut start = 0;
            while start < ids.len() {
                let end = start.saturating_add(chunk).min(ids.len());
                handles.push(scope.spawn(move || load_notes_range(store, ids_slice, start, end)));
                start = end;
            }
            let mut notes = Vec::with_capacity(ids.len());
            for handle in handles {
                match handle.join() {
                    Ok(result) => notes.extend(result?),
                    Err(_) => {
                        return Err(crate::Error::internal(
                            "thread de leitura do corpus abortou",
                        ));
                    }
                }
            }
            Ok(notes)
        })
    }

    /// Lê todas as notas legíveis e deriva índice e grafo de uma só passada.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura (`Io`) e de extração de metadados (`Schema`).
    pub fn load(store: &Store<'_>) -> Result<Self> {
        Self::from_notes(Self::load_notes(store)?)
    }

    /// Como [`Corpus::load`], mas reusa o índice persistido em `.idx/` quando ele está **fresco**
    /// (`mtime` ≥ o de todas as notas); caso contrário, reconstrói a partir das mesmas notas e
    /// regrava (E15-T11/O1.6). Devolve os avisos do índice (ex.: acima do teto de tamanho).
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse e de escrita do índice.
    pub fn load_fresh(store: &Store<'_>, fs: &dyn Fs, root: &Path) -> Result<(Self, Vec<String>)> {
        let notes = Self::load_notes(store)?;
        let mut warnings = Vec::new();
        let index = if let Some(index) = Index::load_if_fresh(fs, root, store, &mut warnings)? {
            index
        } else {
            let index = Index::build(&notes)?;
            index.save(fs, root, &mut warnings)?;
            warnings
                .push("índice ausente/desatualizado: reconstruído a partir de notas/".to_string());
            index
        };
        let graph = Graph::from_notes_ref(&notes)?;
        Ok((
            Self {
                notes,
                index,
                graph,
            },
            warnings,
        ))
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

/// Largura da paralelização: 1 (sequencial) para corpora pequenos ou máquina de 1 núcleo.
fn parallel_width(len: usize) -> usize {
    if len < PARALLEL_MIN_NOTES {
        return 1;
    }
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(PARALLEL_MAX)
        .min(len)
}

/// Lê `ids[start..end]` (uma faixa contígua), descartando notas inválidas (`read_optional`).
fn load_notes_range(
    store: &Store<'_>,
    ids: &[String],
    start: usize,
    end: usize,
) -> Result<Vec<Note>> {
    let slice = ids.get(start..end).unwrap_or_default();
    let mut notes = Vec::with_capacity(slice.len());
    for id in slice {
        if let Some(note) = store.read_optional(id)? {
            notes.push(note);
        }
    }
    Ok(notes)
}
