//! Ciclo de vida de tarefa: `review` (fechamento) e `outcome` (D53/D138).

use crate::schema::{Status, Value};
use crate::store::Note;
use crate::write::{WriteAction, WriteContext, event};
use crate::{Error, Result};

/// Reexporta a evidência de execução generalizada (D103).
pub use crate::write::outcome::{OutcomeStatus, outcome};

/// Ação de ciclo de vida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskAction {
    /// Encerra (`closed`).
    Review,
}

impl TaskAction {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
        }
    }

    const fn target(self) -> Status {
        match self {
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

/// Garante que a nota é item de trabalho (tem `scope`).
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
