//! Orquestração do `rewind` (E08-T01..T04).

use crate::Result;
use crate::graph::Graph;
use crate::handoff::context::{ContextStore, derive_id, is_valid_context_id};
use crate::handoff::{RewindInput, RewindMode, RewindRequest, rewind};
use crate::lifecycle::{DEFAULT_TASK_CONFIRMATION, Freshness};
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::NoteType;

use super::{NOW, built, container, member, note};

fn input<'a>(index: &'a Index, graph: &'a Graph, changed_paths: &'a [String]) -> RewindInput<'a> {
    RewindInput {
        index,
        graph,
        events: &[],
        changed_paths,
        freshness: Freshness::default(),
        task_confirmation_weight: DEFAULT_TASK_CONFIRMATION,
        now_ms: NOW,
    }
}

#[test]
fn manifest_mode_counts() -> Result<()> {
    let fs = MemFs::new();
    let notes = [note(NoteType::Fact, "a")?, note(NoteType::Decision, "b")?];
    let (index, graph) = built(&notes)?;
    let input = input(&index, &graph, &[]);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let output = rewind(&input, &RewindRequest::new(), &contexts)?;
    assert!(output.text.starts_with("notes=2"));
    assert!(output.items.is_empty());
    assert!(is_valid_context_id(&output.context_id));
    Ok(())
}

#[test]
fn scope_mode_ranks_members() -> Result<()> {
    let fs = MemFs::new();
    let plan = container("plano")?;
    let plan_id = plan.id()?.to_string();
    let inside = member("dentro", &[], &plan_id)?;
    let outside = note(NoteType::Fact, "fora")?;
    let notes = [plan, inside, outside];
    let (index, graph) = built(&notes)?;
    let input = input(&index, &graph, &[]);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let mut request = RewindRequest::new();
    request.mode = RewindMode::Scope(plan_id.clone());
    let output = rewind(&input, &request, &contexts)?;
    assert_eq!(output.items.len(), 2);
    assert!(output.text.contains(&plan_id));
    Ok(())
}

#[test]
fn files_mode_matches_anchor() -> Result<()> {
    let fs = MemFs::new();
    let notes = [
        super::anchored("ancorada", &["src/**"])?,
        note(NoteType::Fact, "solta")?,
    ];
    let (index, graph) = built(&notes)?;
    let changed = vec!["src/main.rs".to_string()];
    let input = input(&index, &graph, &changed);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let mut request = RewindRequest::new();
    request.mode = RewindMode::Files(vec!["src/main.rs".to_string()]);
    let output = rewind(&input, &request, &contexts)?;
    assert_eq!(output.items.len(), 1);
    Ok(())
}

#[test]
fn budget_truncates_last_item() -> Result<()> {
    let fs = MemFs::new();
    let plan = container("plano")?;
    let plan_id = plan.id()?.to_string();
    let long = "a".repeat(118);
    let notes = [
        plan,
        member(&long, &[], &plan_id)?,
        member(&format!("b{long}"), &[], &plan_id)?,
        member(&format!("c{long}"), &[], &plan_id)?,
    ];
    let (index, graph) = built(&notes)?;
    let input = input(&index, &graph, &[]);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let mut request = RewindRequest::new();
    request.mode = RewindMode::Scope(plan_id);
    request.budget = 50;
    let output = rewind(&input, &request, &contexts)?;
    assert!(!output.items.is_empty());
    assert!(output.dropped > 0);
    Ok(())
}

#[test]
fn resume_is_byte_identical() -> Result<()> {
    let fs = MemFs::new();
    let notes = [note(NoteType::Fact, "a")?, note(NoteType::Decision, "b")?];
    let (index, graph) = built(&notes)?;
    let input = input(&index, &graph, &[]);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let first = rewind(&input, &RewindRequest::new(), &contexts)?;

    let mut request = RewindRequest::new();
    request.resume = Some(first.context_id.clone());
    let second = rewind(&input, &request, &contexts)?;
    assert_eq!(first.context_id, second.context_id);
    assert_eq!(first.text, second.text);
    Ok(())
}

#[test]
fn resume_missing_context_errors() -> Result<()> {
    let fs = MemFs::new();
    let notes = [note(NoteType::Fact, "a")?];
    let (index, graph) = built(&notes)?;
    let input = input(&index, &graph, &[]);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let mut request = RewindRequest::new();
    request.resume = Some(derive_id("ausente"));
    assert!(rewind(&input, &request, &contexts).is_err());
    Ok(())
}

#[test]
fn auto_flips_on_large_corpus() -> Result<()> {
    let fs = MemFs::new();
    let mut notes = Vec::new();
    for index in 0..6 {
        notes.push(container(&format!("container {index}"))?);
    }
    let (index, graph) = built(&notes)?;
    let changed = vec!["src/main.rs".to_string()];
    let input = input(&index, &graph, &changed);
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let mut request = RewindRequest::new();
    request.mode = RewindMode::Auto;
    let output = rewind(&input, &request, &contexts)?;
    assert!(output.text.starts_with("notes="));
    assert!(output.items.is_empty());
    Ok(())
}

#[test]
fn manifest_reports_embeddings_pending() -> Result<()> {
    let fs = MemFs::new();
    let notes = vec![note(NoteType::Fact, "a")?, note(NoteType::Fact, "b")?];
    let (index, graph) = built(&notes)?;
    let contexts = ContextStore::new(&fs, "/p/.knudge");
    let request = RewindRequest::new();
    let mut input = input(&index, &graph, &[]);
    input.freshness.pending = 2;
    let output = rewind(&input, &request, &contexts)?;
    assert!(output.text.contains("fresh: stale=0 expiring=0 pending=2"));
    assert_eq!(output.embeddings_pending, 2);
    Ok(())
}
