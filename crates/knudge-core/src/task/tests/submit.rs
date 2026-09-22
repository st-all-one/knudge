//! Criação e filiação de tarefas (E08-T07).

use crate::Result;
use crate::graph::Graph;
use crate::schema::{EdgeKind, NoteType, Scope, Value};
use crate::task::{TaskSpec, is_task, parent_of, submit};
use crate::write::WriteContext;

use super::MemFs;

fn spec(scope: Scope, statement: &str) -> TaskSpec {
    TaskSpec::new(scope, statement)
}

fn new_ctx(fs: &MemFs) -> Result<WriteContext<'_>> {
    super::context(fs)
}

#[test]
fn plan_epic_issue_task_chain() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;

    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    let plan_note = ctx.store().read(&plan.id)?;
    assert_eq!(plan_note.frontmatter.note_type()?, NoteType::Container);
    assert_eq!(plan_note.frontmatter.scope()?, Some(Scope::Plan));
    assert_eq!(plan.parent, None);

    let mut epic_spec = spec(Scope::Epic, "épico");
    epic_spec.parent = Some(plan.id.clone());
    epic_spec.blocks = Some(1);
    let epic = submit(&ctx, &epic_spec)?;
    let epic_note = ctx.store().read(&epic.id)?;
    assert_eq!(epic_note.frontmatter.note_type()?, NoteType::Container);
    assert!(epic_note.body.contains("knudge:parent"));
    assert!(epic_note.body.contains(&plan.id));

    let plan_note = ctx.store().read(&plan.id)?;
    assert_eq!(
        plan_note.frontmatter.string_list("results_in")?,
        vec![epic.id.as_str()]
    );

    let mut issue_spec = spec(Scope::Issue, "issue");
    issue_spec.parent = Some(epic.id);
    let issue = submit(&ctx, &issue_spec)?;
    assert_eq!(
        ctx.store().read(&issue.id)?.frontmatter.note_type()?,
        NoteType::Task
    );

    let mut task_spec = spec(Scope::Task, "tarefa");
    task_spec.parent = Some(issue.id);
    let task = submit(&ctx, &task_spec)?;
    assert_eq!(
        ctx.store().read(&task.id)?.frontmatter.note_type()?,
        NoteType::Task
    );
    Ok(())
}

#[test]
fn wrong_parent_scope_is_rejected() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;

    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(plan.id);
    assert!(submit(&ctx, &issue).is_err());
    Ok(())
}

#[test]
fn plan_cannot_have_parent() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    let mut inner = spec(Scope::Plan, "outro");
    inner.parent = Some(plan.id);
    assert!(submit(&ctx, &inner).is_err());
    Ok(())
}

#[test]
fn blocks_must_be_positive_and_require_parent() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let mut zero = spec(Scope::Plan, "plano");
    zero.blocks = Some(0);
    assert!(submit(&ctx, &zero).is_err());

    let mut orphan = spec(Scope::Epic, "épico");
    orphan.blocks = Some(1);
    assert!(submit(&ctx, &orphan).is_err());
    Ok(())
}

#[test]
fn duplicate_task_conflicts() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let _first = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    assert!(submit(&ctx, &spec(Scope::Plan, "plano")).is_err());
    Ok(())
}

#[test]
fn depends_on_becomes_edge() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    let mut epic = spec(Scope::Epic, "épico");
    epic.parent = Some(plan.id);
    let epic = submit(&ctx, &epic)?;
    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(epic.id.clone());
    let issue = submit(&ctx, &issue)?;

    let mut task = spec(Scope::Task, "tarefa");
    task.depends_on = vec![epic.id.clone()];
    task.parent = Some(issue.id);
    let task = submit(&ctx, &task)?;
    let note = ctx.store().read(&task.id)?;
    assert_eq!(
        note.frontmatter.string_list("depends_on")?,
        vec![epic.id.as_str()]
    );
    let graph = Graph::build(ctx.store())?;
    assert_eq!(graph.targets(&task.id, EdgeKind::DependsOn), &[epic.id]);
    Ok(())
}

#[test]
fn submit_records_membership_event() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    let mut epic = spec(Scope::Epic, "épico");
    epic.parent = Some(plan.id.clone());
    let epic = submit(&ctx, &epic)?;

    let (events, warnings) = ctx.events().read_all()?;
    assert!(warnings.is_empty());
    assert!(events.iter().any(|event| {
        event.op == "task"
            && event.note_id.as_deref() == Some(plan.id.as_str())
            && event.data.get("child").and_then(Value::as_str) == Some(epic.id.as_str())
    }));
    Ok(())
}

#[test]
fn parent_of_reads_marker() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let plan = submit(&ctx, &spec(Scope::Plan, "plano"))?;
    let mut epic = spec(Scope::Epic, "épico");
    epic.parent = Some(plan.id.clone());
    let epic = submit(&ctx, &epic)?;
    let note = ctx.store().read(&epic.id)?;
    assert_eq!(parent_of(&note), Some(plan.id));
    assert!(is_task(&note));
    Ok(())
}
