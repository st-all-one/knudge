//! Testes do TMS/derrota de crenças (E19-T10/D208).

use std::collections::BTreeSet;

use super::{frontmatter, note};
use crate::Result;
use crate::graph::{Graph, defeated_by_replacement, defeated_dependents, retracted};
use crate::schema::{EdgeKind, Frontmatter, NoteType, Status, Value};
use crate::store::Note;

fn with_status(mut fm: Frontmatter, status: Status) -> Result<Frontmatter> {
    fm.set("status", Value::Str(status.as_str().to_string()))?;
    Ok(fm)
}

#[test]
fn retracted_collects_forgotten_and_superseded() -> Result<()> {
    let forgotten = with_status(
        frontmatter(NoteType::Fact, "premissa esquecida")?,
        Status::Forgotten,
    )?;
    let superseded = with_status(
        frontmatter(NoteType::Fact, "premissa substituida")?,
        Status::Superseded,
    )?;
    let active = frontmatter(NoteType::Fact, "premissa ativa")?;
    let active_id = active.id()?.to_string();
    let graph = Graph::from_notes(vec![
        Note::new(forgotten, ""),
        Note::new(superseded, ""),
        Note::new(active, ""),
    ])?;
    let retracted = retracted(&graph);
    assert_eq!(retracted.len(), 2);
    assert!(!retracted.contains(&active_id));
    Ok(())
}

#[test]
fn defeated_dependents_is_transitive_and_excludes_premises() -> Result<()> {
    // a depende de b; b depende de c; c é premissa retratada.
    let a = frontmatter(NoteType::Fact, "dependente a")?
        .id()?
        .to_string();
    let b = frontmatter(NoteType::Fact, "dependente b")?
        .id()?
        .to_string();
    let c = frontmatter(NoteType::Fact, "premissa c")?.id()?.to_string();
    let notes = vec![
        note(NoteType::Fact, "dependente a", &[(EdgeKind::DependsOn, &b)])?,
        note(NoteType::Fact, "dependente b", &[(EdgeKind::DependsOn, &c)])?,
        note(NoteType::Fact, "premissa c", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let mut retracted = BTreeSet::new();
    retracted.insert(c);
    let defeated = defeated_dependents(&graph, &retracted);
    assert_eq!(defeated.len(), 2);
    assert!(defeated.contains(&a));
    assert!(defeated.contains(&b));
    Ok(())
}

#[test]
fn defeated_by_replacement_marks_the_replaced_note() -> Result<()> {
    let old = frontmatter(NoteType::Fact, "versao antiga")?
        .id()?
        .to_string();
    let notes = vec![
        note(NoteType::Fact, "versao nova", &[(EdgeKind::Replaces, &old)])?,
        note(NoteType::Fact, "versao antiga", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let losers = defeated_by_replacement(&graph);
    assert_eq!(losers.len(), 1);
    assert!(losers.contains(&old));
    Ok(())
}

#[test]
fn a_retracted_note_without_dependents_defeats_nothing() -> Result<()> {
    let fm = with_status(frontmatter(NoteType::Fact, "sozinha")?, Status::Forgotten)?;
    let id = fm.id()?.to_string();
    let graph = Graph::from_notes(vec![Note::new(fm, "")])?;
    let mut retracted = BTreeSet::new();
    retracted.insert(id);
    assert!(defeated_dependents(&graph, &retracted).is_empty());
    Ok(())
}
