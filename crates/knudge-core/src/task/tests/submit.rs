//! Criação e filiação de tarefas (E08-T07/D134).

use crate::Result;
use crate::graph::Graph;
use crate::schema::{EdgeKind, NoteType, Scope, Value};
use crate::task::{TaskSpec, is_task, parent_of, submit};
use crate::write::{WriteContext, link};

use super::MemFs;

fn spec(scope: Scope, statement: &str) -> TaskSpec {
    TaskSpec::new(scope, statement)
}

fn new_ctx(fs: &MemFs) -> Result<WriteContext<'_>> {
    super::context(fs)
}

#[test]
fn epic_issue_task_chain() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;

    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let epic_note = ctx.store().read(&epic.id)?;
    assert_eq!(epic_note.frontmatter.note_type()?, NoteType::Epic);
    assert_eq!(epic_note.frontmatter.scope()?, Some(Scope::Epic));
    assert_eq!(epic.parent, None);

    let mut issue_spec = spec(Scope::Issue, "issue");
    issue_spec.parent = Some(epic.id.clone());
    issue_spec.blocks = Some(1);
    let issue = submit(&ctx, &issue_spec)?;
    let issue_note = ctx.store().read(&issue.id)?;
    assert_eq!(issue_note.frontmatter.note_type()?, NoteType::Task);
    assert!(issue_note.body.contains("knudge:parent"));
    assert!(issue_note.body.contains(&epic.id));
    assert_eq!(
        ctx.store()
            .read(&epic.id)?
            .frontmatter
            .string_list("results_in")?,
        vec![issue.id.as_str()]
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
fn epic_can_parent_a_task_directly() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut task = spec(Scope::Task, "tarefa");
    task.parent = Some(epic.id);
    assert!(submit(&ctx, &task).is_ok());
    Ok(())
}

#[test]
fn wrong_parent_scope_is_rejected() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(epic.id);
    let issue = submit(&ctx, &issue)?;

    // `issue` não pode ser filho de outro `issue` (rank não é estritamente menor).
    let mut bad = spec(Scope::Issue, "outra issue");
    bad.parent = Some(issue.id);
    assert!(submit(&ctx, &bad).is_err());
    Ok(())
}

#[test]
fn epic_cannot_have_parent() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut inner = spec(Scope::Epic, "outro");
    inner.parent = Some(epic.id);
    assert!(submit(&ctx, &inner).is_err());
    Ok(())
}

#[test]
fn blocks_must_be_positive_and_require_parent() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let mut zero = spec(Scope::Epic, "épico");
    zero.blocks = Some(0);
    assert!(submit(&ctx, &zero).is_err());

    let mut orphan = spec(Scope::Epic, "épico órfão");
    orphan.blocks = Some(1);
    assert!(submit(&ctx, &orphan).is_err());
    Ok(())
}

#[test]
fn duplicate_task_conflicts() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let _first = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    assert!(submit(&ctx, &spec(Scope::Epic, "épico")).is_err());
    Ok(())
}

#[test]
fn link_creates_depends_on_edge() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(epic.id.clone());
    let issue = submit(&ctx, &issue)?;

    let mut task = spec(Scope::Task, "tarefa");
    task.parent = Some(issue.id);
    let task = submit(&ctx, &task)?;
    let _linked = link(&ctx, &task.id, EdgeKind::DependsOn, &epic.id)?;
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
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(epic.id.clone());
    let issue = submit(&ctx, &issue)?;

    let (events, warnings) = ctx.events().read_all()?;
    assert!(warnings.is_empty());
    assert!(events.iter().any(|event| {
        event.op == "task"
            && event.note_id.as_deref() == Some(epic.id.as_str())
            && event.data.get("child").and_then(Value::as_str) == Some(issue.id.as_str())
    }));
    Ok(())
}

#[test]
fn parent_of_reads_marker() -> Result<()> {
    let fs = MemFs::new();
    let ctx = new_ctx(&fs)?;
    let epic = submit(&ctx, &spec(Scope::Epic, "épico"))?;
    let mut issue = spec(Scope::Issue, "issue");
    issue.parent = Some(epic.id.clone());
    let issue = submit(&ctx, &issue)?;
    let note = ctx.store().read(&issue.id)?;
    assert_eq!(parent_of(&note), Some(epic.id));
    assert!(is_task(&note));
    Ok(())
}
