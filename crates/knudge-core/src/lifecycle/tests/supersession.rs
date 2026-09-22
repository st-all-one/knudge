//! Testes da supersessão com proteção de ciclo (E10-T04).

use crate::Result;
use crate::graph::Graph;
use crate::lifecycle::supersession::{cycle_members, demote, filter_protected, protected};
use crate::ports::fakes::MemFs;
use crate::schema::{EdgeKind, NoteType, Status, id};
use crate::store::Note;
use crate::write::Draft;

use super::{NOW, built, note_created, seeded};

/// Par de notas que se substituem mutuamente (ciclo de supersessão).
fn cycle_pair() -> Result<(Note, Note, String, String)> {
    let a_id = id::note_id(NoteType::Decision, "a");
    let b_id = id::note_id(NoteType::Decision, "b");
    let mut a = Draft::new(NoteType::Decision, "a");
    a.edges = vec![(EdgeKind::Replaces, b_id.clone())];
    let mut b = Draft::new(NoteType::Decision, "b");
    b.edges = vec![(EdgeKind::Replaces, a_id.clone())];
    Ok((a.to_note(NOW)?, b.to_note(NOW)?, a_id, b_id))
}

#[test]
fn cycle_members_are_protected() -> Result<()> {
    let (a, b, a_id, b_id) = cycle_pair()?;
    let graph = Graph::from_notes(vec![a, b])?;
    let members = cycle_members(&graph);
    assert!(members.contains(&a_id));
    assert!(members.contains(&b_id));
    assert!(protected(&graph, &a_id));
    Ok(())
}

#[test]
fn cycle_member_is_not_demoted() -> Result<()> {
    let fs = MemFs::new();
    let (a, b, a_id, _) = cycle_pair()?;
    let notes = vec![a, b];
    let ctx = seeded(&fs, &notes, NOW)?;
    let (_, graph) = built(&notes)?;

    let outcome = demote(&ctx, &graph, &a_id, "anchor_decay")?;
    assert!(outcome.is_none());
    assert_eq!(
        ctx.store().read(&a_id)?.frontmatter.status()?,
        Status::Active
    );
    Ok(())
}

#[test]
fn non_cycle_note_is_demoted() -> Result<()> {
    let fs = MemFs::new();
    let note = note_created(NoteType::Fact, "viva", "", NOW)?;
    let id = note.id()?.to_string();
    let notes = vec![note];
    let ctx = seeded(&fs, &notes, NOW)?;
    let (_, graph) = built(&notes)?;

    let outcome = demote(&ctx, &graph, &id, "anchor_decay")?;
    assert!(outcome.is_some());
    assert_eq!(
        ctx.store().read(&id)?.frontmatter.status()?,
        Status::Forgotten
    );
    Ok(())
}

#[test]
fn filter_protected_keeps_only_free_notes() -> Result<()> {
    let (a, b, a_id, _) = cycle_pair()?;
    let free = note_created(NoteType::Fact, "livre", "", NOW)?;
    let free_id = free.id()?.to_string();
    let graph = Graph::from_notes(vec![a, b, free])?;

    let ids = vec![a_id, free_id.clone()];
    let filtered = filter_protected(&graph, &ids);
    assert_eq!(filtered.first().map(String::as_str), Some(free_id.as_str()));
    Ok(())
}
