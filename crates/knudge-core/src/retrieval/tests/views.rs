//! Views `ready`/`blocked` (D53).

use crate::Result;
use crate::graph::{Graph, link};
use crate::retrieval::{BlockReason, block_reason, compute_views};
use crate::schema::{EdgeKind, NoteType, Scope, Status, id};
use crate::store::Note;

use super::{base, with_scope, with_status};

#[test]
fn ready_and_blocked_follow_dependencies() -> Result<()> {
    let closed_id = id::note_id(NoteType::Task, "base feita");
    let closed = Note::new(
        with_status(base(NoteType::Task, "base feita")?, Status::Closed)?,
        "",
    );

    let mid_id = id::note_id(NoteType::Task, "meio");
    let mut mid_frontmatter = base(NoteType::Task, "meio")?;
    link(&mut mid_frontmatter, EdgeKind::DependsOn, &closed_id)?;
    let mid = Note::new(mid_frontmatter, "");

    let top_id = id::note_id(NoteType::Task, "topo");
    let mut top_frontmatter = base(NoteType::Task, "topo")?;
    link(&mut top_frontmatter, EdgeKind::DependsOn, &mid_id)?;
    let top = Note::new(top_frontmatter, "");

    let open_id = id::note_id(NoteType::Task, "aberta");
    let open = Note::new(base(NoteType::Task, "aberta")?, "");

    let blocked_id = id::note_id(NoteType::Task, "bloqueada");
    let mut blocked_frontmatter = base(NoteType::Task, "bloqueada")?;
    link(&mut blocked_frontmatter, EdgeKind::DependsOn, &open_id)?;
    let blocked = Note::new(blocked_frontmatter, "");

    let graph = Graph::from_notes(vec![closed, mid, top, open, blocked])?;
    let views = compute_views(&graph);

    assert!(views.ready.contains(&closed_id));
    assert!(views.ready.contains(&mid_id));
    assert!(views.blocked.contains(&top_id));
    assert!(views.blocked.contains(&blocked_id));
    Ok(())
}

#[test]
fn dependency_cycle_is_blocked() -> Result<()> {
    let a_id = id::note_id(NoteType::Task, "a");
    let b_id = id::note_id(NoteType::Task, "b");

    let mut a_frontmatter = base(NoteType::Task, "a")?;
    link(&mut a_frontmatter, EdgeKind::DependsOn, &b_id)?;
    let mut b_frontmatter = base(NoteType::Task, "b")?;
    link(&mut b_frontmatter, EdgeKind::DependsOn, &a_id)?;

    let graph = Graph::from_notes(vec![
        Note::new(a_frontmatter, ""),
        Note::new(b_frontmatter, ""),
    ])?;
    let views = compute_views(&graph);
    assert!(views.blocked.contains(&a_id));
    assert!(views.blocked.contains(&b_id));
    assert!(views.ready.is_empty());
    Ok(())
}

#[test]
fn block_reason_reports_smallest_pending_dependency() -> Result<()> {
    let dep_a = id::note_id(NoteType::Task, "dep a");
    let dep_b = id::note_id(NoteType::Task, "dep b");
    let mut frontmatter = base(NoteType::Task, "dependente")?;
    link(&mut frontmatter, EdgeKind::DependsOn, &dep_a)?;
    link(&mut frontmatter, EdgeKind::DependsOn, &dep_b)?;
    let target = id::note_id(NoteType::Task, "dependente");
    let graph = Graph::from_notes(vec![
        Note::new(frontmatter, ""),
        Note::new(base(NoteType::Task, "dep a")?, ""),
        Note::new(base(NoteType::Task, "dep b")?, ""),
    ])?;
    let expected = if dep_a < dep_b { dep_a } else { dep_b };
    assert_eq!(
        block_reason(&graph, &target),
        Some(BlockReason::Dependency(expected))
    );
    Ok(())
}

#[test]
fn error_kind_work_item_is_ready_but_knowledge_error_is_not() -> Result<()> {
    // Espécie `error` **com** `scope` é item de trabalho (D113/D120): entra em `ready`.
    let work_id = id::note_id(NoteType::Error, "corrigir bug");
    let work = Note::new(
        with_scope(base(NoteType::Error, "corrigir bug")?, Scope::Task)?,
        "",
    );
    // Espécie `error` **sem** `scope` é conhecimento: não entra nas views.
    let knowledge_id = id::note_id(NoteType::Error, "bug conhecido");
    let knowledge = Note::new(base(NoteType::Error, "bug conhecido")?, "");

    let graph = Graph::from_notes(vec![work, knowledge])?;
    let views = compute_views(&graph);
    assert!(views.ready.contains(&work_id));
    assert!(!views.ready.contains(&knowledge_id));
    assert!(!views.blocked.contains(&knowledge_id));
    Ok(())
}

#[test]
fn block_reason_reports_cycle_and_ignores_containers() -> Result<()> {
    let a_id = id::note_id(NoteType::Task, "a");
    let b_id = id::note_id(NoteType::Task, "b");
    let mut a = base(NoteType::Task, "a")?;
    link(&mut a, EdgeKind::DependsOn, &b_id)?;
    let mut b = base(NoteType::Task, "b")?;
    link(&mut b, EdgeKind::DependsOn, &a_id)?;
    let epic_id = id::note_id(NoteType::Epic, "épico");
    let graph = Graph::from_notes(vec![
        Note::new(a, ""),
        Note::new(b, ""),
        Note::new(base(NoteType::Epic, "épico")?, ""),
    ])?;
    assert_eq!(block_reason(&graph, &a_id), Some(BlockReason::Cycle));
    assert_eq!(block_reason(&graph, &epic_id), None);
    Ok(())
}
