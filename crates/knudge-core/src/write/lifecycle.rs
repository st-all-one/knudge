//! Ciclo de vida soft: `link`, `forget` e `restore` (D49/D52).
//!
//! `forget`/`restore` só mudam `status` — o arquivo **nunca** é removido. `link` acrescenta uma
//! aresta explícita e, para `replaces`, mantém o ponteiro reverso `superseded_by` (D46).

use crate::graph;
use crate::schema::{EdgeKind, Status, Value};
use crate::store::{Event, Store};
use crate::{Error, Result};

use super::status::validate_transition;
use super::{WriteAction, WriteContext, event};

/// Marca a nota como `forgotten` (soft-delete — D52).
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para transição proibida e propaga erros de I/O.
pub fn forget(ctx: &WriteContext<'_>, id: &str, reason: Option<&str>) -> Result<u32> {
    if ctx.store().read(id)?.frontmatter.status()? == Status::Forgotten {
        return Err(Error::invalid_input("nota já está `forgotten`"));
    }
    let mut record = event("forget", id, ctx.now_ms(), WriteAction::Updated, None);
    if let Some(reason) = reason {
        record = record.with_data("reason", Value::Str(reason.to_string()));
    }
    transition(ctx, id, Status::Forgotten, &record)
}

/// Restaura uma nota `forgotten` para `active`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` se a nota não estiver `forgotten`.
pub fn restore(ctx: &WriteContext<'_>, id: &str) -> Result<u32> {
    if ctx.store().read(id)?.frontmatter.status()? != Status::Forgotten {
        return Err(Error::invalid_input(
            "só nota `forgotten` pode ser restaurada",
        ));
    }
    let record = event("restore", id, ctx.now_ms(), WriteAction::Updated, None);
    transition(ctx, id, Status::Active, &record)
}

/// Acrescenta uma aresta explícita; retorna `true` se mudou.
///
/// # Errors
/// Retorna `ErrorKind::Schema` para destino inválido/auto-aresta e propaga erros de I/O.
pub fn link(ctx: &WriteContext<'_>, from: &str, kind: EdgeKind, to: &str) -> Result<bool> {
    let mut note = ctx.store().read(from)?;
    if !graph::link(&mut note.frontmatter, kind, to)? {
        return Ok(false);
    }
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    let record = event("link", from, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("kind", Value::Str(kind.as_str().to_string()))
        .with_data("to", Value::Str(to.to_string()));
    ctx.events().append(&record)?;
    if kind == EdgeKind::Replaces {
        link_reverse(ctx.store(), from, to)?;
    }
    Ok(true)
}

fn transition(ctx: &WriteContext<'_>, id: &str, target: Status, record: &Event) -> Result<u32> {
    let mut note = ctx.store().read(id)?;
    validate_transition(note.frontmatter.status()?, target)?;
    note.frontmatter
        .set("status", Value::Str(target.as_str().to_string()))?;
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.refresh_body_hash()?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    ctx.events().append(record)?;
    Ok(revision)
}

fn link_reverse(store: &Store<'_>, from: &str, to: &str) -> Result<()> {
    match store.read(to) {
        Ok(mut target) => {
            target
                .frontmatter
                .set("superseded_by", Value::Str(from.to_string()))?;
            target.frontmatter.set(
                "status",
                Value::Str(Status::Superseded.as_str().to_string()),
            )?;
            let revision = target.revision().saturating_add(1);
            target.set_revision(revision)?;
            target.refresh_body_hash()?;
            target.frontmatter.validate()?;
            store.write(&target)
        }
        Err(Error::NotFound(_)) => Ok(()),
        Err(error) => Err(error),
    }
}
