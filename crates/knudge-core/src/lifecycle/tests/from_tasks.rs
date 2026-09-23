//! Confirmação derivada de tarefas que compartilham âncoras (X1/D108).

use crate::Result;
use crate::lifecycle::confidence::{from_tasks, from_tasks_with};
use crate::retrieval::{Index, Meta};
use crate::schema::{NoteType, Scope, Value};
use crate::store::Note;
use crate::write::Draft;

use super::NOW;

fn note_with(
    note_type: NoteType,
    statement: &str,
    anchors: &[&str],
    scope: Option<Scope>,
    outcomes: &[&str],
) -> Result<Note> {
    let mut draft = Draft::new(note_type, statement);
    draft.scope = scope;
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    let mut note = draft.to_note(NOW)?;
    if !outcomes.is_empty() {
        let items = outcomes
            .iter()
            .map(|status| Value::map([("status".to_string(), Value::Str((*status).to_string()))]))
            .collect();
        note.frontmatter.set("outcomes", Value::List(items))?;
    }
    Ok(note)
}

fn confirmation(index: &Index, id: &str, weight: f64) -> Option<f64> {
    index
        .docs
        .iter()
        .find(|doc| doc.meta.id == id)
        .map(|doc| from_tasks(&doc.meta, index, weight))
}

#[test]
fn success_task_confirms_shared_anchor() -> Result<()> {
    let task = note_with(
        NoteType::Task,
        "implementar retry",
        &["src/retry.ts"],
        Some(Scope::Task),
        &["success"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[task, decision])?;

    assert_eq!(confirmation(&index, &decision_id, 0.1), Some(0.1));
    Ok(())
}

#[test]
fn failure_task_does_not_confirm() -> Result<()> {
    let task = note_with(
        NoteType::Task,
        "tentar retry",
        &["src/retry.ts"],
        Some(Scope::Task),
        &["failure"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[task, decision])?;

    assert_eq!(confirmation(&index, &decision_id, 0.1), Some(0.0));
    Ok(())
}

#[test]
fn non_task_outcome_does_not_confirm() -> Result<()> {
    let fact = note_with(
        NoteType::Fact,
        "cache usa body_hash",
        &["src/retry.ts"],
        None,
        &["success"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[fact, decision])?;

    assert_eq!(confirmation(&index, &decision_id, 0.1), Some(0.0));
    Ok(())
}

#[test]
fn unrelated_anchor_does_not_confirm() -> Result<()> {
    let task = note_with(
        NoteType::Task,
        "implementar retry",
        &["src/retry.ts"],
        Some(Scope::Task),
        &["success"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/queue.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[task, decision])?;

    assert_eq!(confirmation(&index, &decision_id, 0.1), Some(0.0));
    Ok(())
}

#[test]
fn confirmation_saturates_at_one() -> Result<()> {
    let mut notes = Vec::new();
    for n in 0..15 {
        notes.push(note_with(
            NoteType::Task,
            &format!("tarefa {n}"),
            &["src/retry.ts"],
            Some(Scope::Task),
            &["success"],
        )?);
    }
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    notes.push(decision);
    let index = Index::build(&notes)?;

    assert_eq!(confirmation(&index, &decision_id, 0.1), Some(1.0));
    Ok(())
}

#[test]
fn zero_weight_disables_confirmation() -> Result<()> {
    let task = note_with(
        NoteType::Task,
        "implementar retry",
        &["src/retry.ts"],
        Some(Scope::Task),
        &["success"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[task, decision])?;

    assert_eq!(confirmation(&index, &decision_id, 0.0), Some(0.0));
    Ok(())
}

#[test]
fn from_tasks_with_matches_index_scan() -> Result<()> {
    let task = note_with(
        NoteType::Task,
        "implementar retry",
        &["src/retry.ts"],
        Some(Scope::Task),
        &["success"],
    )?;
    let decision = note_with(
        NoteType::Decision,
        "usar full jitter",
        &["src/retry.ts"],
        None,
        &[],
    )?;
    let decision_id = decision.id()?.to_string();
    let index = Index::build(&[task, decision])?;
    let confirmers: Vec<&Meta> = index
        .docs
        .iter()
        .map(|doc| &doc.meta)
        .filter(|meta| meta.scope.is_some() && meta.confirmation > 0.0)
        .collect();
    let Some(doc) = index.docs.iter().find(|doc| doc.meta.id == decision_id) else {
        return Ok(());
    };

    assert!(
        (from_tasks_with(&doc.meta, &confirmers, 0.1) - from_tasks(&doc.meta, &index, 0.1)).abs()
            < 1e-9
    );
    Ok(())
}
