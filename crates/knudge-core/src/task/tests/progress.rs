//! Rollup de progresso por épico: `epic_of`/`progress_of` (D127).

use crate::Result;
use crate::graph::Graph;
use crate::schema::Scope;
use crate::task::{Progress, TaskAction, TaskSpec, apply, epic_of, progress_of, submit};
use crate::write::WriteContext;

use super::{MemFs, context};

/// Constrói `épico > issue > {a, b}` e devolve `(epic, issue, a)`.
fn fixture(ctx: &WriteContext<'_>) -> Result<(String, String, String)> {
    let epic = submit(ctx, &TaskSpec::new(Scope::Epic, "épico"))?.id;
    let mut issue = TaskSpec::new(Scope::Issue, "issue");
    issue.parent = Some(epic.clone());
    let issue = submit(ctx, &issue)?.id;
    let mut a = TaskSpec::new(Scope::Task, "a");
    a.parent = Some(issue.clone());
    let a = submit(ctx, &a)?.id;
    let mut b = TaskSpec::new(Scope::Task, "b");
    b.parent = Some(issue.clone());
    let _b = submit(ctx, &b)?;
    Ok((epic, issue, a))
}

#[test]
fn progress_counts_leaf_work_items() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let (epic, issue, a) = fixture(&ctx)?;

    let graph = Graph::build(ctx.store())?;
    assert_eq!(progress_of(&graph, &epic), Progress::new(0, 2));
    assert_eq!(epic_of(&graph, &a).as_deref(), Some(epic.as_str()));
    assert_eq!(epic_of(&graph, &epic).as_deref(), Some(epic.as_str()));

    // Fechar a `issue` (que ainda tem task aberta) não conta: o progresso é de folhas.
    let _ignored = apply(&ctx, &issue, TaskAction::Review)?;
    let graph = Graph::build(ctx.store())?;
    assert_eq!(progress_of(&graph, &epic), Progress::new(0, 2));

    let _ignored = apply(&ctx, &a, TaskAction::Review)?;
    let graph = Graph::build(ctx.store())?;
    assert_eq!(progress_of(&graph, &epic), Progress::new(1, 2));
    Ok(())
}

#[test]
fn epic_of_is_none_without_epic() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let issue = submit(&ctx, &TaskSpec::new(Scope::Issue, "issue"))?.id;
    let graph = Graph::build(ctx.store())?;
    assert_eq!(epic_of(&graph, &issue), None);
    Ok(())
}

#[test]
fn progress_label_and_ratio() {
    let progress = Progress::new(3, 16);
    assert_eq!(progress.label(), "3/16");
    assert!((progress.ratio() - 0.1875_f64).abs() < f64::EPSILON);
    assert!(Progress::default().is_empty());
    assert_eq!(Progress::default().label(), "0/0");
}
