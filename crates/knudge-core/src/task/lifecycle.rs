//! Ciclo de vida de tarefa: `adopt`/`release`/`review`, `outcome` e `reorder` (D53).

use crate::schema::{Status, Value};
use crate::store::Note;
use crate::write::{WriteAction, WriteContext, event};
use crate::{Error, Result};

use super::membership;

/// Reexporta a evidência de execução generalizada (D103).
pub use crate::write::outcome::{OutcomeStatus, outcome};

/// Ação de ciclo de vida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskAction {
    /// Marca como `in_progress`.
    Adopt,
    /// Devolve para `active`.
    Release,
    /// Encerra (`closed`).
    Review,
}

impl TaskAction {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adopt => "adopt",
            Self::Release => "release",
            Self::Review => "review",
        }
    }

    const fn target(self) -> Status {
        match self {
            Self::Adopt => Status::InProgress,
            Self::Release => Status::Active,
            Self::Review => Status::Closed,
        }
    }
}

/// Aplica uma transição de ciclo de vida e devolve a nova revisão.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para transição proibida e `Schema` para não-tarefa.
pub fn apply(ctx: &WriteContext<'_>, id: &str, action: TaskAction) -> Result<u32> {
    let mut note = ctx.store().read(id)?;
    ensure_task(&note)?;
    validate_transition(note.frontmatter.status()?, action.target())?;
    note.frontmatter
        .set("status", Value::Str(action.target().as_str().to_string()))?;
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    let record = event("task", id, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("action", Value::Str(action.as_str().to_string()));
    ctx.events().append(&record)?;
    Ok(revision)
}

/// Reordena o filho dentro do pai (`blocks` 1-based) e devolve a nova revisão.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` se a nota não tiver pai ou `blocks` for 0.
pub fn reorder(ctx: &WriteContext<'_>, id: &str, blocks: u32) -> Result<u32> {
    if blocks == 0 {
        return Err(Error::invalid_input("`blocks` é 1-based (D53)"));
    }
    let mut note = ctx.store().read(id)?;
    ensure_task(&note)?;
    let marker = membership::parse(&note.body)
        .ok_or_else(|| Error::invalid_input("tarefa sem pai não pode ser reordenada"))?;
    note.body = membership::set(&note.body, &marker.parent, Some(blocks));
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.refresh_body_hash()?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    let record = event("task", id, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("action", Value::Str("reorder".to_string()))
        .with_data("blocks", Value::Int(i64::from(blocks)));
    ctx.events().append(&record)?;
    Ok(revision)
}

fn ensure_task(note: &Note) -> Result<()> {
    if note.frontmatter.scope()?.is_some() {
        Ok(())
    } else {
        Err(Error::schema("nota sem `scope` não é item de trabalho"))
    }
}

/// Valida uma transição de status de tarefa (`from → to`).
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para transições proibidas (D53).
pub fn validate_transition(from: Status, to: Status) -> Result<()> {
    if from == to {
        return Ok(());
    }
    let allowed = matches!(
        (from, to),
        (
            Status::Active | Status::InProgress,
            Status::InProgress | Status::Closed
        ) | (Status::InProgress, Status::Active)
            | (Status::Blocked, Status::Active | Status::InProgress)
    );
    if allowed {
        Ok(())
    } else {
        Err(Error::invalid_input(format!(
            "transição de tarefa inválida: `{from}` → `{to}`"
        )))
    }
}
