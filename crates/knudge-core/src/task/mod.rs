//! Escopo `task`: hierarquia `plan ⊃ epic ⊃ issue ⊃ task` como view derivada (E08-T07).
//!
//! `plan`/`epic` são `type=container` (sem verdade própria, D52); `issue`/`task` são
//! `type=task`. O pai vive no **marcador do corpo** (D93) e é projetado como aresta
//! `results_in` no grafo. `kd write` rejeita `task`/`container` — tudo de tarefa passa aqui.

pub mod hierarchy;
pub mod lifecycle;
pub mod membership;
pub mod program;
pub mod spec;

#[cfg(test)]
mod tests;

pub use hierarchy::{children, expected_parent, validate_blocks, validate_parent};
pub use lifecycle::{OutcomeStatus, TaskAction, apply, outcome, reorder, validate_transition};
pub use membership::Marker;
pub use program::{ProgramNode, program_of, root_for_path, subtree};
pub use spec::TaskSpec;

use crate::graph;
use crate::schema::{EdgeKind, NoteType, Value};
use crate::store::{Note, commit};
use crate::write::{WriteAction, WriteContext, event};
use crate::{Error, Result};

/// Resultado de [`submit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitOutcome {
    /// Id da tarefa criada.
    pub id: String,
    /// Pai, quando houver.
    pub parent: Option<String>,
}

/// Cria uma tarefa/container e a anexa ao pai (D52/D93).
///
/// # Errors
/// - `ErrorKind::Conflict` se o id já existe;
/// - `ErrorKind::Schema` para hierarquia/escopo/marcador inválidos;
/// - propaga erros de I/O.
pub fn submit(ctx: &WriteContext<'_>, spec: &TaskSpec) -> Result<SubmitOutcome> {
    if let Some(parent) = &spec.parent {
        let parent_note = ctx.store().read(parent)?;
        let parent_scope = parent_note
            .frontmatter
            .scope()?
            .ok_or_else(|| Error::schema(format!("pai {parent} não tem `scope`")))?;
        validate_parent(parent_scope, spec.scope)?;
    }
    let note = spec.to_note(ctx.now_ms())?;
    let id = note.id()?.to_string();
    if ctx.store().exists(&id) {
        return Err(Error::conflict(format!("tarefa {id} já existe")));
    }
    let record = event("task", &id, ctx.now_ms(), WriteAction::Created, None)
        .with_data("action", Value::Str("submit".to_string()))
        .with_data("scope", Value::Str(spec.scope.as_str().to_string()));
    commit(ctx.store(), ctx.events(), &note, &record)?;
    if let Some(parent) = &spec.parent {
        attach(ctx, parent, &id)?;
    }
    Ok(SubmitOutcome {
        id,
        parent: spec.parent.clone(),
    })
}

/// Id do pai declarado no marcador do corpo.
#[must_use]
pub fn parent_of(note: &Note) -> Option<String> {
    membership::parse(&note.body).map(|marker| marker.parent)
}

/// `true` se a nota é tarefa ou container.
#[must_use]
pub fn is_task(note: &Note) -> bool {
    matches!(
        note.frontmatter.note_type(),
        Ok(NoteType::Task | NoteType::Container)
    )
}

fn attach(ctx: &WriteContext<'_>, parent: &str, child: &str) -> Result<()> {
    let mut parent_note = ctx.store().read(parent)?;
    if !graph::link(&mut parent_note.frontmatter, EdgeKind::ResultsIn, child)? {
        return Ok(());
    }
    let revision = parent_note.revision().saturating_add(1);
    parent_note.set_revision(revision)?;
    parent_note.frontmatter.validate()?;
    ctx.store().write(&parent_note)?;
    let record = event("task", parent, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("action", Value::Str("attach".to_string()))
        .with_data("child", Value::Str(child.to_string()));
    ctx.events().append(&record)
}
