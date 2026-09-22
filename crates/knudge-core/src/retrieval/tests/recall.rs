//! Contratos `recall`/`get` e degradação graciosa (E06-T06/T07).

use crate::Result;
use crate::graph::Graph;
use crate::ports::fakes::MemFs;
use crate::retrieval::{Index, RecallQuery, Why, format_hit, get, recall};
use crate::schema::{NoteType, id};
use crate::store::Store;

use super::{anchored, note};

#[test]
fn pipe_format_golden() -> Result<()> {
    let index = Index::build(&[note(NoteType::Fact, "alpha", "")?])?;
    let graph = Graph::from_notes(Vec::new())?;
    let output = recall(&index, &graph, &RecallQuery::new("alpha"))?;

    let hit = output.hits.first().cloned();
    let expected_id = id::note_id(NoteType::Fact, "alpha");
    assert_eq!(
        hit.as_ref().map(format_hit),
        Some(format!("{expected_id}|alpha|0.02|universal"))
    );
    Ok(())
}

#[test]
fn why_belongs_to_closed_set() -> Result<()> {
    let index = Index::build(&[note(NoteType::Fact, "alpha", "")?])?;
    let graph = Graph::from_notes(Vec::new())?;
    let output = recall(&index, &graph, &RecallQuery::new("alpha"))?;
    assert!(output.hits.iter().all(|hit| Why::ALL.contains(&hit.why)));
    Ok(())
}

#[test]
fn anchored_note_recalls_without_lexical_match() -> Result<()> {
    let anchored = anchored(NoteType::Fact, "sem termos da consulta", &["V2/**"])?;
    let anchored_id = anchored.id()?.to_string();
    let index = Index::build(&[anchored])?;
    let graph = Graph::from_notes(Vec::new())?;

    let mut query = RecallQuery::new("zzz inexistente");
    query.working_paths = vec!["V2/foo.rs".to_string()];
    let output = recall(&index, &graph, &query)?;

    let hit = output.hits.first();
    assert_eq!(hit.map(|hit| hit.id.as_str()), Some(anchored_id.as_str()));
    assert_eq!(hit.map(|hit| hit.why), Some(Why::FileMatch));
    Ok(())
}

#[test]
fn failed_channel_degrades_and_strict_errors() -> Result<()> {
    let index = Index::build(&[note(NoteType::Fact, "alpha", "")?])?;
    let graph = Graph::from_notes(Vec::new())?;

    let mut query = RecallQuery::new("alpha");
    query.channel_warnings = vec!["vetor indisponível".to_string()];
    let output = recall(&index, &graph, &query)?;
    assert!(!output.hits.is_empty());
    assert_eq!(output.warnings.len(), 1);

    query.strict = true;
    assert!(recall(&index, &graph, &query).is_err());
    Ok(())
}

#[test]
fn get_reads_only_requested_ids() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    store.ensure_dirs()?;
    let alpha = note(NoteType::Fact, "alpha", "corpo a")?;
    let alpha_id = alpha.id()?.to_string();
    store.write(&alpha)?;

    let output = get(&store, &[alpha_id.clone(), "fact_00000000".to_string()])?;
    assert_eq!(output.notes.len(), 1);
    assert_eq!(
        output.notes.first().map(|note| note.id().ok()),
        Some(Some(alpha_id.as_str()))
    );
    assert_eq!(output.warnings.len(), 1);
    Ok(())
}
