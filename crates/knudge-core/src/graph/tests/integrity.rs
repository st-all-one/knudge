//! Integridade referencial e bidirecionalidade (E05-T03 / D46).

use crate::Result;
use crate::graph::{Graph, IssueKind};
use crate::schema::{EdgeKind, NoteType, id};
use crate::store::Note;

use super::{force_edge, frontmatter, note, with_superseded};

#[test]
fn dangling_edge_is_reported() -> Result<()> {
    let missing = id::note_id(NoteType::Fact, "Nunca");
    let b = note(NoteType::Decision, "B", &[(EdgeKind::DependsOn, &missing)])?;
    let b_id = b.id()?.to_string();

    let graph = Graph::from_notes(vec![b])?;
    let issue = graph.integrity().into_iter().next();
    assert_eq!(
        issue.as_ref().map(|issue| issue.kind),
        Some(IssueKind::Dangling)
    );
    assert_eq!(
        issue.as_ref().map(|issue| issue.from.as_str()),
        Some(b_id.as_str())
    );
    assert_eq!(issue.map(|issue| issue.to), Some(missing));
    Ok(())
}

#[test]
fn superseded_by_without_replaces_is_missing_forward() -> Result<()> {
    let b = note(NoteType::Fact, "B", &[])?;
    let b_id = b.id()?.to_string();
    let a = Note::new(
        with_superseded(frontmatter(NoteType::Fact, "A")?, &b_id)?,
        "",
    );
    let a_id = a.id()?.to_string();

    let graph = Graph::from_notes(vec![a, b])?;
    let issues = graph.integrity();
    assert_eq!(issues.len(), 1);
    let issue = issues.first();
    assert_eq!(
        issue.map(|issue| issue.kind),
        Some(IssueKind::MissingForward)
    );
    assert_eq!(issue.map(|issue| issue.from.as_str()), Some(a_id.as_str()));
    assert_eq!(issue.map(|issue| issue.to.as_str()), Some(b_id.as_str()));
    Ok(())
}

#[test]
fn replaces_without_superseded_by_is_missing_backref() -> Result<()> {
    let a = note(NoteType::Fact, "A", &[])?;
    let a_id = a.id()?.to_string();
    let b = note(NoteType::Fact, "B", &[(EdgeKind::Replaces, &a_id)])?;
    let b_id = b.id()?.to_string();

    let graph = Graph::from_notes(vec![a, b])?;
    let issues = graph.integrity();
    assert_eq!(issues.len(), 1);
    let issue = issues.first();
    assert_eq!(
        issue.map(|issue| issue.kind),
        Some(IssueKind::MissingBackref)
    );
    assert_eq!(issue.map(|issue| issue.from.as_str()), Some(b_id.as_str()));
    assert_eq!(issue.map(|issue| issue.to.as_str()), Some(a_id.as_str()));
    Ok(())
}

#[test]
fn self_edge_is_reported() -> Result<()> {
    let mut fm = frontmatter(NoteType::Fact, "A")?;
    let own = fm.id()?.to_string();
    force_edge(&mut fm, EdgeKind::DependsOn, &own)?;

    let graph = Graph::from_notes(vec![Note::new(fm, "")])?;
    let issue = graph.integrity().into_iter().next();
    assert_eq!(
        issue.as_ref().map(|issue| issue.kind),
        Some(IssueKind::SelfEdge)
    );
    assert_eq!(
        issue.and_then(|issue| issue.edge),
        Some(EdgeKind::DependsOn)
    );
    Ok(())
}

#[test]
fn well_formed_supersession_has_no_issues() -> Result<()> {
    let a_id = id::note_id(NoteType::Fact, "A");
    let b_id = id::note_id(NoteType::Fact, "B");

    let a = Note::new(
        with_superseded(frontmatter(NoteType::Fact, "A")?, &b_id)?,
        "",
    );
    let b = note(NoteType::Fact, "B", &[(EdgeKind::Replaces, &a_id)])?;

    let graph = Graph::from_notes(vec![a, b])?;
    assert!(graph.integrity().is_empty());
    Ok(())
}
