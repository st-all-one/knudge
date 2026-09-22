//! Testes da auditoria (E09-T03).

use std::path::Path;

use crate::Result;
use crate::graph::suggestions::SuggestionStore;
use crate::graph::{Graph, Suggestion};
use crate::health::{AuditInput, audit};
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{EdgeKind, NoteType};
use crate::write::WriteContext;
use crate::write::dedup::DedupThresholds;

use super::{NOW, PROJECT, ROOT, anchored, built, note, seeded};

fn input<'a>(
    fs: &'a MemFs,
    ctx: &'a WriteContext<'a>,
    index: &'a Index,
    graph: &'a Graph,
    thresholds: &'a DedupThresholds,
) -> AuditInput<'a> {
    AuditInput {
        fs,
        root: Path::new(ROOT),
        project_root: Path::new(PROJECT),
        store: ctx.store(),
        graph,
        index,
        now_ms: NOW,
        lock_stale_ms: 30_000,
        thresholds,
    }
}

#[test]
fn clean_corpus_has_no_issues() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "tudo certo", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let (index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let report = audit(&input(&fs, &ctx, &index, &graph, &thresholds))?;
    assert!(report.is_clean(), "{report:?}");
    Ok(())
}

#[test]
fn broken_anchor_is_reported() -> Result<()> {
    let fs = MemFs::new();
    let fact = anchored("âncora quebrada", &["src/sumiu.rs"], "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let (index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let report = audit(&input(&fs, &ctx, &index, &graph, &thresholds))?;
    assert_eq!(report.broken_anchors.len(), 1);
    assert_eq!(
        report.broken_anchors.first().map(|a| a.anchor.as_str()),
        Some("src/sumiu.rs")
    );
    Ok(())
}

#[test]
fn stale_lock_is_reported() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "lock", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let (index, graph) = built(std::slice::from_ref(&fact))?;
    fs.insert_at("/p/.knudge/.locks/a.lock", b"{}".to_vec(), NOW - 60_000);
    let thresholds = DedupThresholds::default();
    let report = audit(&input(&fs, &ctx, &index, &graph, &thresholds))?;
    assert_eq!(report.stale_locks.len(), 1);
    Ok(())
}

#[test]
fn suggested_edge_not_materialized_is_reported() -> Result<()> {
    let fs = MemFs::new();
    let left = note(NoteType::Fact, "origem", "")?;
    let right = note(NoteType::Fact, "destino", "")?;
    let left_id = left.id()?.to_string();
    let right_id = right.id()?.to_string();
    let ctx = seeded(&fs, &[left.clone(), right.clone()])?;
    let (index, graph) = built(&[left, right])?;

    let suggestions = SuggestionStore::new(&fs, ROOT);
    suggestions.write(
        &left_id,
        &[Suggestion {
            kind: EdgeKind::References,
            target: right_id.clone(),
            reason: "menção".to_string(),
        }],
    )?;

    let thresholds = DedupThresholds::default();
    let report = audit(&input(&fs, &ctx, &index, &graph, &thresholds))?;
    assert_eq!(report.missing_edges.len(), 1);
    assert_eq!(
        report.missing_edges.first().map(|e| e.targets.clone()),
        Some(vec![right_id])
    );
    Ok(())
}
