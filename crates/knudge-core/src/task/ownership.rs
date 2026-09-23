//! Dono derivado de eventos `claim`/`release` (D114).
//!
//! A posse **não** é armazenada no frontmatter: dois agentes em branches paralelas que
//! reivindicam a mesma tarefa resolvem por `merge=union` + dedup de eventos (D96), sem conflito
//! de campo. O dono é sempre uma **projeção** do log.

use crate::store::events::Event;
use crate::write::WriteContext;
use crate::{Error, Result};

/// `op` de reivindicação.
pub const CLAIM: &str = "claim";
/// `op` de liberação.
pub const RELEASE: &str = "release";

/// Dono derivado: `actor` do último `claim` não seguido de `release`/`close`.
#[must_use]
pub fn ownership(events: &[Event], id: &str) -> Option<String> {
    let mut owner: Option<String> = None;
    for record in events {
        if record.note_id.as_deref() != Some(id) {
            continue;
        }
        match record.op.as_str() {
            CLAIM => owner.clone_from(&record.actor),
            RELEASE | "close" => owner = None,
            _ => {}
        }
    }
    owner
}

/// Registra `claim` (com `actor`) ou `release` (sem ator).
///
/// # Errors
/// Retorna `ErrorKind::Schema` se a nota não for item de trabalho (sem `scope`); propaga I/O.
pub fn claim(ctx: &WriteContext<'_>, id: &str, actor: Option<&str>) -> Result<()> {
    let note = ctx.store().read(id)?;
    if note.frontmatter.scope()?.is_none() {
        return Err(Error::schema(format!("{id} não é item de trabalho")));
    }
    let op = if actor.is_some() { CLAIM } else { RELEASE };
    let mut record = Event::new(op, ctx.now_ms()).with_note_id(id);
    if let Some(actor) = actor {
        record = record.with_actor(actor);
    }
    ctx.events().append(&record)
}
