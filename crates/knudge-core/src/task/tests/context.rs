//! Contexto estrutural de um item: pai, bloqueadores, bloqueados e filhos (D125).

use crate::Result;
use crate::graph::Graph;
use crate::schema::{EdgeKind, Scope, Status};
use crate::task::{Progress, TaskSpec, context_of, submit};
use crate::write;

use super::{MemFs, context};

#[test]
fn context_resolves_parent_blockers_and_children() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let epic = submit(&ctx, &TaskSpec::new(Scope::Epic, "épico"))?.id;
    let mut issue = TaskSpec::new(Scope::Issue, "issue");
    issue.parent = Some(epic.clone());
    let issue = submit(&ctx, &issue)?.id;
    let mut a = TaskSpec::new(Scope::Task, "a");
    a.parent = Some(issue.clone());
    let a = submit(&ctx, &a)?.id;
    let mut b = TaskSpec::new(Scope::Task, "b");
    b.parent = Some(issue.clone());
    let b = submit(&ctx, &b)?.id;
    let _dep = write::link(&ctx, &b, EdgeKind::DependsOn, &a)?;

    let graph = Graph::build(ctx.store())?;
    let view = context_of(ctx.store(), &graph, &b)?;
    assert_eq!(
        view.parent.as_ref().map(|r| r.id.as_str()),
        Some(issue.as_str())
    );
    assert_eq!(
        view.parent.as_ref().map(|r| r.statement.as_str()),
        Some("issue")
    );
    assert_eq!(
        view.parent.as_ref().and_then(|r| r.scope),
        Some(Scope::Issue)
    );
    assert_eq!(view.blocked_by.len(), 1);
    assert_eq!(
        view.blocked_by.first().map(|r| r.id.as_str()),
        Some(a.as_str())
    );
    assert!(view.blocks.is_empty());
    assert!(view.children.is_empty());
    assert_eq!(
        view.epic.as_ref().map(|entry| entry.epic.id.as_str()),
        Some(epic.as_str())
    );
    assert_eq!(
        view.epic.as_ref().map(|entry| entry.progress),
        Some(Progress::new(0, 2))
    );

    let view_a = context_of(ctx.store(), &graph, &a)?;
    assert_eq!(view_a.blocks.len(), 1);
    assert_eq!(
        view_a.blocks.first().map(|r| r.id.as_str()),
        Some(b.as_str())
    );
    assert_eq!(
        view_a.parent.as_ref().map(|r| r.id.as_str()),
        Some(issue.as_str())
    );
    Ok(())
}

#[test]
fn context_reports_children_and_status() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let issue = submit(&ctx, &TaskSpec::new(Scope::Issue, "issue"))?.id;
    let mut done = TaskSpec::new(Scope::Task, "concluída");
    done.parent = Some(issue.clone());
    done.status = Some(Status::Closed);
    let done = submit(&ctx, &done)?.id;
    let mut open = TaskSpec::new(Scope::Task, "aberta");
    open.parent = Some(issue.clone());
    let _open = submit(&ctx, &open)?;

    let graph = Graph::build(ctx.store())?;
    let view = context_of(ctx.store(), &graph, &issue)?;
    assert_eq!(view.children.len(), 2);
    assert_eq!(
        view.children
            .iter()
            .find(|r| r.id == done)
            .map(|r| r.status),
        Some(Status::Closed)
    );
    Ok(())
}
