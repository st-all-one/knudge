//! `learn`: propostas determinísticas (E08-T06).

use crate::Result;
use crate::graph::Graph;
use crate::maintenance::learn::{LearnInput, LearnKind, LearnProposal, learn};
use crate::retrieval::Index;
use crate::schema::NoteType;
use crate::write::dedup::DedupThresholds;

use super::{anchored, note, success_task};

fn proposals<'a>(
    index: &'a Index,
    graph: &'a Graph,
    changed_paths: &'a [String],
    thresholds: &'a DedupThresholds,
) -> Vec<LearnProposal> {
    let input = LearnInput {
        index,
        graph,
        events: &[],
        changed_paths,
        scope: None,
        thresholds,
    };
    learn(&input)
}

#[test]
fn write_gap_proposes_create_note() -> Result<()> {
    let notes = [note(NoteType::Fact, "sem ancora")?];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let changed = vec!["src/x.rs".to_string()];
    let found = proposals(&index, &graph, &changed, &DedupThresholds::default());
    assert!(found.iter().any(|p| p.kind == LearnKind::CreateNote));
    Ok(())
}

#[test]
fn covered_path_yields_no_gap() -> Result<()> {
    let notes = [anchored("com ancora", &["src/**"])?];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let changed = vec!["src/x.rs".to_string()];
    let found = proposals(&index, &graph, &changed, &DedupThresholds::default());
    assert!(!found.iter().any(|p| p.kind == LearnKind::CreateNote));
    Ok(())
}

#[test]
fn success_task_without_note_proposes_create_note() -> Result<()> {
    let notes = [success_task("implementar retry", &["src/retry.ts"])?];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let found = proposals(&index, &graph, &[], &DedupThresholds::default());
    assert!(
        found
            .iter()
            .any(|p| p.kind == LearnKind::CreateNote && p.why == "tarefa fechada sem nota"),
        "propostas: {found:?}"
    );
    Ok(())
}

#[test]
fn success_task_covered_by_note_yields_no_task_gap() -> Result<()> {
    let notes = [
        success_task("implementar retry", &["src/retry.ts"])?,
        anchored("decisão de backoff", &["src/retry.ts"])?,
    ];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let found = proposals(&index, &graph, &[], &DedupThresholds::default());
    assert!(
        !found
            .iter()
            .any(|p| p.kind == LearnKind::CreateNote && p.why == "tarefa fechada sem nota"),
        "propostas: {found:?}"
    );
    Ok(())
}

#[test]
fn near_duplicates_propose_supersede() -> Result<()> {
    let notes = [
        note(NoteType::Fact, "alpha beta gamma")?,
        note(NoteType::Fact, "gamma beta alpha")?,
    ];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let found = proposals(&index, &graph, &[], &DedupThresholds::default());
    assert!(found.iter().any(|p| p.kind == LearnKind::Supersede));
    Ok(())
}

#[test]
fn shared_anchor_proposes_link() -> Result<()> {
    let notes = [anchored("um", &["src/**"])?, anchored("dois", &["src/**"])?];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let found = proposals(&index, &graph, &[], &DedupThresholds::default());
    assert!(found.iter().any(|p| p.kind == LearnKind::Link));
    Ok(())
}

#[test]
fn learn_is_deterministic() -> Result<()> {
    let notes = [
        anchored("um", &["src/**"])?,
        anchored("dois", &["src/**"])?,
        note(NoteType::Fact, "alpha beta")?,
    ];
    let index = Index::build(&notes)?;
    let graph = Graph::from_notes(notes.to_vec())?;
    let changed = vec!["docs/y.md".to_string()];
    let thresholds = DedupThresholds::default();
    let first = proposals(&index, &graph, &changed, &thresholds);
    let second = proposals(&index, &graph, &changed, &thresholds);
    assert_eq!(first, second);
    Ok(())
}
