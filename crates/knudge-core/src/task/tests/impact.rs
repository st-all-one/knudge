//! Impacto derivado: tarefas abertas que dependem de um nó (D106/D109).

use crate::Result;
use crate::graph::Graph;
use crate::schema::{NoteType, Scope, Status};
use crate::store::Note;
use crate::task::{TaskSpec, impact, is_actionable, submit};
use proptest::prelude::*;

use super::{MemFs, NOW, context};

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
fn error_kind_work_item_counts_as_dependent() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let a = submit(&ctx, &TaskSpec::new(Scope::Task, "a"))?.id;
    let mut b = TaskSpec::new(Scope::Task, "corrigir bug");
    b.kind = Some(NoteType::Error);
    b.depends_on = vec![a.clone()];
    let _b = submit(&ctx, &b)?;

    let graph = graph_of(&fs)?;
    assert_eq!(
        impact(&graph, &a),
        1,
        "espécie `error` deve contar no impacto"
    );
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

#[test]
fn actionable_excludes_terminal_statuses() {
    assert!(is_actionable(Some(Status::Active)));
    assert!(is_actionable(Some(Status::InProgress)));
    assert!(is_actionable(Some(Status::Blocked)));
    assert!(!is_actionable(Some(Status::Closed)));
    assert!(!is_actionable(Some(Status::Superseded)));
    assert!(!is_actionable(Some(Status::Forgotten)));
    // status desconhecido é tolerado como acionável (nunca acontece vindo do store)
    assert!(is_actionable(None));
}

/// Id determinístico da tarefa `t<index>` (a afirmação define o id).
fn task_id(index: usize) -> Result<String> {
    let note = TaskSpec::new(Scope::Task, format!("t{index}")).to_note(NOW)?;
    Ok(note.id()?.to_string())
}

/// Grafo de 4 tarefas `t0..t3` com as arestas `(from depende de to)`. `to_note` não toca o
/// store, então o id pode ser recalculado com a mesma afirmação.
fn graph_with(edges: &[(usize, usize)]) -> Result<Graph> {
    let ids: Vec<String> = (0..4).map(task_id).collect::<Result<Vec<_>>>()?;
    let mut specs: Vec<TaskSpec> = (0..4)
        .map(|index| TaskSpec::new(Scope::Task, format!("t{index}")))
        .collect();
    for (from, to) in edges {
        if let (Some(spec), Some(id)) = (specs.get_mut(*from), ids.get(*to)) {
            spec.depends_on.push(id.clone());
        }
    }
    let notes = specs
        .iter()
        .map(|spec| spec.to_note(NOW))
        .collect::<Result<Vec<_>>>()?;
    Graph::from_notes(notes)
}

proptest! {
    #[test]
    fn adding_dependency_never_decreases_impact(
        edges in prop::collection::vec((0usize..4, 0usize..4), 0..6),
        extra in (0usize..4, 0usize..4),
    ) {
        let before = graph_with(&edges).ok();
        let mut after_edges = edges;
        after_edges.push(extra);
        let after = graph_with(&after_edges).ok();
        if let (Some(before), Some(after)) = (before, after) {
            for index in 0..4 {
                if let Ok(id) = task_id(index) {
                    prop_assert!(impact(&after, &id) >= impact(&before, &id));
                }
            }
        }
    }
}
