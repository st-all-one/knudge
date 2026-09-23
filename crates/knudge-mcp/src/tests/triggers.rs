//! Testes do motor de gatilhos do MCP (E12-T03).

use knudge_core::handoff::{ManifestItem, TrustTier};
use knudge_core::maintenance::{LearnKind, LearnProposal};
use knudge_core::write::Candidate;

use crate::triggers::{HintEngine, HintKind};

fn candidate(id: &str, score: f64) -> Candidate {
    Candidate {
        id: id.to_string(),
        statement: format!("nota {id}"),
        score,
    }
}

fn item(id: &str, score: f64) -> ManifestItem {
    ManifestItem {
        id: id.to_string(),
        statement: format!("nota {id}"),
        tier: TrustTier::Tactical,
        score,
        created_ms: 0,
    }
}

fn proposal(kind: LearnKind, ids: &[&str], score: f64) -> LearnProposal {
    LearnProposal {
        kind,
        ids: ids.iter().map(|id| (*id).to_string()).collect(),
        why: "teste".to_string(),
        score,
    }
}

#[test]
fn pre_write_caps_hints() {
    let mut engine = HintEngine::new(3, 0);
    let candidates = vec![
        candidate("a", 0.9),
        candidate("b", 0.8),
        candidate("c", 0.7),
        candidate("d", 0.6),
    ];
    let hints = engine.pre_write(&candidates);
    assert_eq!(hints.len(), 3);
    assert!(hints.iter().all(|hint| hint.kind == HintKind::Duplicate));
}

#[test]
fn repeated_hints_are_deduped() {
    let mut engine = HintEngine::new(3, 0);
    let candidates = vec![candidate("a", 0.9)];
    assert_eq!(engine.pre_write(&candidates).len(), 1);
    assert!(engine.pre_write(&candidates).is_empty());
}

#[test]
fn session_end_requires_zero_writes() {
    let mut engine = HintEngine::new(3, 0);
    let proposals = vec![proposal(LearnKind::CreateNote, &["a"], 0.5)];
    assert!(engine.session_end(1, &proposals).is_empty());
    assert_eq!(engine.session_end(0, &proposals).len(), 1);
}

#[test]
fn observation_mode_is_marked() {
    let mut engine = HintEngine::new(3, 1);
    let hints = engine.pre_edit(&[item("a", 1.0)]);
    assert_eq!(hints.first().map(|hint| hint.observed), Some(true));
    assert!(engine.is_observing());
    engine.end_session();
    assert!(!engine.is_observing());
    let hints = engine.pre_edit(&[item("a", 1.0)]);
    assert_eq!(hints.first().map(|hint| hint.observed), Some(false));
}

#[test]
fn session_end_maps_learn_kinds() {
    let mut engine = HintEngine::new(3, 0);
    let proposals = vec![
        proposal(LearnKind::CreateNote, &["a"], 0.5),
        proposal(LearnKind::Merge, &["b", "c"], 0.7),
        proposal(LearnKind::Link, &["d"], 0.3),
    ];
    let hints = engine.session_end(0, &proposals);
    assert_eq!(hints.len(), 3);
    assert!(hints.iter().any(|hint| hint.kind == HintKind::WriteGap));
    assert!(hints.iter().any(|hint| hint.kind == HintKind::Merge));
    assert!(hints.iter().any(|hint| hint.kind == HintKind::MissingLink));
}
