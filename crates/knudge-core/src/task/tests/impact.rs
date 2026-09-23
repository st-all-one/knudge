//! Impacto derivado: tarefas abertas que dependem de um nó (D106/D109).

use crate::Result;
use crate::graph::Graph;
use crate::schema::{Scope, Status};
use crate::store::Note;
use crate::task::{TaskSpec, impact, submit};

use super::{MemFs, context};

fn graph_of(fs: &MemFs) -> Result<Graph> {
    let ctx = context(fs)?;
    let mut notes = Vec::new();
    for id in ctx.store().list_ids()? {
        notes.push(ctx.store().read(&id)?);
    }
    Graph::from_notes(notes)
}

#[test]
fn impact_counts_open_dependents_transitively() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let a = submit(&ctx, &TaskSpec::new(Scope::Task, "a"))?.id;
    let mut b = TaskSpec::new(Scope::Task, "b");
    b.depends_on = vec![a.clone()];
    let b = submit(&ctx, &b)?.id;
    let mut c = TaskSpec::new(Scope::Task, "c");
    c.depends_on = vec![b.clone()];
    let c = submit(&ctx, &c)?.id;

    let graph = graph_of(&fs)?;
    assert_eq!(impact(&graph, &a), 2);
    assert_eq!(impact(&graph, &b), 1);
    assert_eq!(impact(&graph, &c), 0);
    Ok(())
}

#[test]
fn closed_dependents_do_not_count() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let a = submit(&ctx, &TaskSpec::new(Scope::Task, "a"))?.id;
    let mut b = TaskSpec::new(Scope::Task, "b");
    b.depends_on = vec![a.clone()];
    b.status = Some(Status::Closed);
    let _b = submit(&ctx, &b)?;

    let graph = graph_of(&fs)?;
    assert_eq!(impact(&graph, &a), 0);
    Ok(())
}

#[test]
fn self_dependency_is_not_counted() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let a = submit(&ctx, &TaskSpec::new(Scope::Task, "a"))?.id;
    let note: Note = ctx.store().read(&a)?;
    let graph = Graph::from_notes([note])?;
    assert_eq!(impact(&graph, &a), 0);
    Ok(())
}
