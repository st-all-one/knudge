//! Testes do escopo `embeddings` (E11).

mod cache;
mod flush;
mod index;
mod lightweight;
mod meta;
mod pipeline;
mod semantic;
mod state;
mod vector;
mod versioned;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::store::{Note, Store};
use crate::write::Draft;

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Raiz do `.knudge/`.
pub(super) const ROOT: &str = "/p/.knudge";

/// Cria uma nota válida com corpo.
pub(super) fn note(statement: &str, body: &str) -> Result<Note> {
    Draft::new(NoteType::Fact, statement)
        .with_body(body)
        .to_note(NOW)
}

/// Cria um store com as notas gravadas.
///
/// # Errors
/// Propaga erros de escrita.
pub(super) fn seeded<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<Store<'a>> {
    let store = Store::new(fs, ROOT);
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    Ok(store)
}
