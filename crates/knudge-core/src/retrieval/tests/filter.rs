//! Filtros determinísticos (D41/D53).

use std::collections::BTreeSet;

use crate::Result;
use crate::retrieval::{Filter, Index, Meta};
use crate::schema::{AgentRole, Classification, NoteType, Status};
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
fn anchor_filter_accepts_note_glob_matching_requested_path() {
    // O pedido é um caminho concreto; a nota está ancorada por glob (direção do canal).
    let mut filter = Filter::new();
    filter.anchors = vec!["src/retry.ts".to_string()];
    let mut meta = empty_meta();
    meta.anchors = vec!["src/**".to_string()];
    assert!(filter.matches(&meta));

    meta.anchors = vec!["other/**".to_string()];
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
        scope: None,
        classification: Classification::Tactical,
        status: Status::Active,
        tags: Vec::new(),
        anchors: Vec::new(),
        agent: None,
        agent_id: None,
        agent_role: None,
        parent_agent_id: None,
        activity: None,
        created_ms: 0,
        confirmation: 0.0,
        failures: 0.0,
    }
}

#[test]
fn provenance_filters_match_agent_and_activity() {
    let mut meta = empty_meta();
    meta.agent = Some("ktu".to_string());
    meta.activity = Some("derive".to_string());

    let mut filter = Filter::new();
    assert!(filter.matches(&meta));
    filter.agents = vec!["ktu".to_string()];
    filter.activities = vec!["derive".to_string()];
    assert!(!filter.is_empty());
    assert!(filter.matches(&meta));

    filter.agents = vec!["goose".to_string()];
    assert!(!filter.matches(&meta));
    filter.agents = vec!["ktu".to_string()];
    filter.activities = vec!["write".to_string()];
    assert!(!filter.matches(&meta));
}

#[test]
fn provenance_filters_exclude_notes_without_provenance() {
    let meta = empty_meta();
    let mut filter = Filter::new();
    filter.agents = vec!["ktu".to_string()];
    assert!(!filter.matches(&meta));
    filter.agents.clear();
    filter.activities = vec!["derive".to_string()];
    assert!(!filter.matches(&meta));
}

#[test]
fn provenance_filters_match_agent_identity_and_role() {
    let mut meta = empty_meta();
    meta.agent = Some("omp".to_string());
    meta.agent_id = Some("omp:main".to_string());
    meta.agent_role = Some(AgentRole::Main);
    meta.parent_agent_id = None;

    let mut filter = Filter::new();
    filter.agent_ids = vec!["omp:main".to_string()];
    filter.agent_roles = vec![AgentRole::Main];
    assert!(filter.matches(&meta));

    filter.agent_ids = vec!["omp:sub".to_string()];
    assert!(!filter.matches(&meta));
    filter.agent_ids = vec!["omp:main".to_string()];
    filter.agent_roles = vec![AgentRole::Subagent];
    assert!(!filter.matches(&meta));

    let mut bare = empty_meta();
    bare.agent_id = None;
    bare.agent_role = None;
    let mut filter = Filter::new();
    filter.agent_ids = vec!["omp:main".to_string()];
    assert!(!filter.matches(&bare));
    filter.agent_ids.clear();
    filter.agent_roles = vec![AgentRole::Main];
    assert!(
        filter.matches(&bare),
        "agent_role ausente aplica o default `main` (D221)"
    );
}

#[test]
fn provenance_identity_filters_apply_read_defaults() {
    let mut meta = empty_meta();
    meta.agent = Some("ktu".to_string());
    let mut filter = Filter::new();
    filter.agent_ids = vec!["ktu".to_string()];
    assert!(
        filter.matches(&meta),
        "agent_id ausente cai para agent (D221)"
    );
    filter.agent_ids = vec!["legacy".to_string()];
    assert!(!filter.matches(&meta));
    filter.agent_ids.clear();
    filter.agent_roles = vec![AgentRole::Main];
    assert!(
        filter.matches(&meta),
        "agent_role ausente cai para main (D221)"
    );
    filter.agent_roles = vec![AgentRole::Subagent];
    assert!(!filter.matches(&meta));

    let bare = empty_meta();
    let mut filter = Filter::new();
    filter.agent_ids = vec!["legacy".to_string()];
    assert!(filter.matches(&bare), "sem proveniência cai para legacy");
}
