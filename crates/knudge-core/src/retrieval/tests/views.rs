//! Views `ready`/`blocked` (D53).

use crate::Result;
use crate::graph::{Graph, link};
use crate::retrieval::compute_views;
use crate::schema::{EdgeKind, NoteType, Status, id};
use crate::store::Note;

use super::{base, with_status};

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
