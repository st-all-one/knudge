//! Filtros determinísticos (D41/D53).

use std::collections::BTreeSet;

use crate::Result;
use crate::retrieval::{Filter, Index, Meta};
use crate::schema::{Classification, NoteType, Status};
use crate::store::Note;

use super::{base, note, tagged, with_status};

#[test]
fn tag_filter_reduces_candidates_before_bm25() -> Result<()> {
    let lang = tagged(NoteType::Fact, "rust json", &["lang"])?;
    let config = tagged(NoteType::Fact, "rust yaml", &["config"])?;
    let expected = lang.id()?.to_string();
    let index = Index::build(&[lang, config])?;

    let mut filter = Filter::new();
    filter.tags = vec!["lang".to_string()];
    let allowed: BTreeSet<String> = index
        .docs
        .iter()
        .filter(|doc| filter.matches(&doc.meta))
        .map(|doc| doc.meta.id.clone())
        .collect();
    assert_eq!(allowed.len(), 1);

    let hits = index.score("rust", &allowed);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(expected.as_str())
    );
    Ok(())
}

#[test]
fn type_and_status_filters() -> Result<()> {
    let task = note(NoteType::Task, "tarefa", "")?;
    let fact = note(NoteType::Fact, "fato", "")?;
    let index = Index::build(&[task, fact])?;

    let mut filter = Filter::new();
    filter.types = vec![NoteType::Task];
    assert_eq!(
        index
            .docs
            .iter()
            .filter(|doc| filter.matches(&doc.meta))
            .count(),
        1
    );

    let closed = Note::new(
        with_status(base(NoteType::Fact, "encerrado")?, Status::Closed)?,
        "",
    );
    let index = Index::build(&[closed])?;
    let mut filter = Filter::new();
    filter.statuses = vec![Status::Closed];
    let matched = index
        .docs
        .iter()
        .filter(|doc| filter.matches(&doc.meta))
        .count();
    assert_eq!(matched, 1);
    Ok(())
}

#[test]
fn classification_and_anchors_filters() {
    let mut filter = Filter::new();
    filter.classifications = vec![Classification::Foundational];
    filter.anchors = vec!["V2/**".to_string()];
    assert!(!filter.is_empty());

    let mut meta = empty_meta();
    meta.classification = Classification::Foundational;
    meta.anchors = vec!["V2/Modules/x.rs".to_string()];
    assert!(filter.matches(&meta));
    meta.anchors = vec!["V1/x.rs".to_string()];
    assert!(!filter.matches(&meta));
}

#[test]
fn empty_filter_accepts_everything() {
    let filter = Filter::new();
    assert!(filter.is_empty());
    assert!(filter.matches(&empty_meta()));
}

fn empty_meta() -> Meta {
    Meta {
        id: "fact_00000000".to_string(),
        note_type: NoteType::Fact,
        classification: Classification::Tactical,
        status: Status::Active,
        tags: Vec::new(),
        anchors: Vec::new(),
        created_ms: 0,
        confirmation: 0.0,
    }
}
