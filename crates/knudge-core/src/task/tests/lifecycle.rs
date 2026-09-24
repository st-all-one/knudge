//! Ciclo de vida de tarefa (E08-T07, D138).

use crate::Result;
use crate::schema::{NoteType, Scope, Status};
use crate::store::{Event, commit};
use crate::task::lifecycle::{OutcomeStatus, TaskAction, apply, outcome, validate_transition};
use crate::task::{TaskSpec, submit};
use crate::write::{Draft, WriteContext};

use super::MemFs;

fn seeded(fs: &MemFs) -> Result<(WriteContext<'_>, String, String)> {
    let ctx = super::context(fs)?;
    let epic = submit(&ctx, &TaskSpec::new(Scope::Epic, "épico"))?;
    let mut issue_spec = TaskSpec::new(Scope::Issue, "issue");
    issue_spec.parent = Some(epic.id.clone());
    let issue = submit(&ctx, &issue_spec)?;
    let mut task_spec = TaskSpec::new(Scope::Task, "tarefa");
    task_spec.parent = Some(issue.id);
    let task = submit(&ctx, &task_spec)?;
    Ok((ctx, epic.id, task.id))
}

#[test]
fn review_closes_task() -> Result<()> {
    let fs = MemFs::new();
    let (ctx, _epic, task) = seeded(&fs)?;

    assert_eq!(apply(&ctx, &task, TaskAction::Review)?, 2);
    assert_eq!(
        ctx.store().read(&task)?.frontmatter.status()?,
        Status::Closed
    );
    Ok(())
}

#[test]
fn closed_task_cannot_reopen() {
    assert!(validate_transition(Status::Closed, Status::InProgress).is_err());
    assert!(validate_transition(Status::Closed, Status::Active).is_err());
}

#[test]
fn outcome_is_appended() -> Result<()> {
    let fs = MemFs::new();
    let (ctx, _epic, task) = seeded(&fs)?;
    assert_eq!(outcome(&ctx, &task, OutcomeStatus::Success, Some("ok"))?, 2);
    assert_eq!(outcome(&ctx, &task, OutcomeStatus::Partial, None)?, 3);

    let note = ctx.store().read(&task)?;
    let outcomes = note
        .frontmatter
        .get("outcomes")
        .and_then(|value| value.as_list())
        .map(<[_]>::len);
    assert_eq!(outcomes, Some(2));
    Ok(())
}

#[test]
fn non_task_cannot_use_lifecycle() -> Result<()> {
    let fs = MemFs::new();
    let ctx = super::context(&fs)?;
    let fact = Draft::new(NoteType::Fact, "fato");
    let note = fact.to_note(super::NOW)?;
    let id = note.id()?.to_string();
    commit(
        ctx.store(),
        ctx.events(),
        &note,
        &Event::new("write", super::NOW),
    )?;
    assert!(apply(&ctx, &id, TaskAction::Review).is_err());
    Ok(())
}
