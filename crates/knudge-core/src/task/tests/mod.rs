//! Testes de tarefa/container (E08-T07).

mod batch;
mod context;
mod hierarchy;
mod impact;
mod kind;
mod lifecycle;
mod mode;
mod plan;
mod program;
mod progress;
mod role;
mod submit;
mod template;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::store::{EventLog, Store};
use crate::write::WriteContext;

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Contexto de escrita vazio.
pub(super) fn context(fs: &MemFs) -> Result<WriteContext<'_>> {
    let store = Store::new(fs, "/p/.knudge");
    store.ensure_dirs()?;
    let events = EventLog::new(fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    Ok(WriteContext::new(store, events, Index::build(&[])?, NOW))
}
