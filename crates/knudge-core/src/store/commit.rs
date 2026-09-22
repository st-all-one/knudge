//! Ordem de commit multi-arquivo (D21): **nota primeiro, evento depois**.
//!
//! Um crash entre os dois deixa no pior caso uma **nota sem evento** — recuperável por
//! `doctor` (E09). O contrário (evento apontando para nota inexistente) é proibido.

use super::Store;
use super::events::{Event, EventLog};
use super::note::Note;
use crate::Result;

/// Grava a nota (canônico) e só então registra o evento (auditoria).
///
/// # Errors
/// Propaga erros de escrita da nota ou do evento. Se o evento falhar, a nota já está no
/// disco — esse é o estado esperado após crash.
pub fn commit(store: &Store<'_>, events: &EventLog<'_>, note: &Note, event: &Event) -> Result<()> {
    store.write(note)?;
    events.append(event)
}
