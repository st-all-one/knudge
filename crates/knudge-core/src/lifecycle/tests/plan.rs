//! Testes do planejamento de demolição (E10-T01/T02/T04).

use std::collections::BTreeMap;
use std::path::Path;

use crate::Result;
use crate::graph::Graph;
use crate::lifecycle::decay::{AnchorValidity, DecayPolicy, compute_anchor_validity};
use crate::lifecycle::plan::{DemotionInput, DemotionReason, demotion_candidates};
use crate::lifecycle::shelf_life::{DAY_MS, ShelfLife};
use crate::lifecycle::supersession::demote;
use crate::ports::fakes::MemFs;
use crate::schema::{Classification, EdgeKind, NoteType, Status, id};
use crate::store::Note;
use crate::write::Draft;

use super::{NOW, PROJECT, anchored, built, classified, note_created, seeded, with_outcomes_at};

fn days(n: i64) -> i64 {
    DAY_MS.saturating_mul(n)
}

#[test]
fn plans_expired_and_decayed_but_not_healthy() -> Result<()> {
    let expired = classified(
        "expira",
        Classification::Observational,
        NOW.saturating_sub(days(31)),
    )?;
    let decayed = anchored("decai", &["a", "b", "c", "d"], NOW.saturating_sub(days(40)))?;
    let healthy = note_created(NoteType::Fact, "saudável", "", NOW)?;
    let healthy_id = healthy.id()?.to_string();
    let decayed_id = decayed.id()?.to_string();
    let notes = vec![expired, decayed, healthy];

    let mut validity = BTreeMap::new();
    validity.insert(
        decayed_id,
        AnchorValidity {
            total: 4,
            valid: 1,
            broken: 3,
        },
    );

    let graph = Graph::from_notes(notes.clone())?;
    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    assert_eq!(candidates.len(), 2);
    let reasons: Vec<DemotionReason> = candidates.iter().map(|c| c.reason).collect();
    assert!(reasons.contains(&DemotionReason::Expired));
    assert!(reasons.contains(&DemotionReason::AnchorDecay));
    assert!(candidates.iter().all(|c| c.id != healthy_id));
    Ok(())
}

#[test]
fn contradiction_proposes_the_losing_side() -> Result<()> {
    // D177: a aresta `contradicts` elege o lado de menor confiança como candidato.
    let strong_id = id::note_id(NoteType::Decision, "forte");
    let strong = with_outcomes_at(
        note_created(NoteType::Decision, "forte", "", NOW)?,
        &[("success", NOW), ("success", NOW), ("success", NOW)],
    )?;
    let mut weak_draft = Draft::new(NoteType::Fact, "fraca");
    weak_draft.edges = vec![(EdgeKind::Contradicts, strong_id)];
    let weak = weak_draft.to_note(NOW)?;
    let weak_id = weak.id()?.to_string();
    let notes = vec![strong, weak];

    let graph = Graph::from_notes(notes.clone())?;
    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &BTreeMap::new(),
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    let contradicted: Vec<&str> = candidates
        .iter()
        .filter(|candidate| candidate.reason == DemotionReason::Contradicted)
        .map(|candidate| candidate.id.as_str())
        .collect();
    assert_eq!(contradicted, vec![weak_id.as_str()]);
    Ok(())
}

#[test]
fn cycle_members_are_excluded_from_plan() -> Result<()> {
    let a_id = id::note_id(NoteType::Decision, "a");
    let b_id = id::note_id(NoteType::Decision, "b");
    let mut a = Draft::new(NoteType::Decision, "a");
    a.classification = Some(Classification::Observational);
    a.edges = vec![(EdgeKind::Replaces, b_id)];
    let mut b = Draft::new(NoteType::Decision, "b");
    b.classification = Some(Classification::Observational);
    b.edges = vec![(EdgeKind::Replaces, a_id)];
    let notes = vec![
        a.to_note(NOW.saturating_sub(days(90)))?,
        b.to_note(NOW.saturating_sub(days(90)))?,
    ];

    let graph = Graph::from_notes(notes.clone())?;
    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let validity = BTreeMap::new();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    assert!(candidates.is_empty());
    Ok(())
}

#[test]
fn end_to_end_decay_demotes_note() -> Result<()> {
    let fs = MemFs::new();
    let note = anchored(
        "decai",
        &["gone/a.rs", "gone/b.rs"],
        NOW.saturating_sub(days(40)),
    )?;
    let id = note.id()?.to_string();
    let notes = vec![note];
    let ctx = seeded(&fs, &notes, NOW)?;
    let (_, graph) = built(&notes)?;

    let anchors = vec!["gone/a.rs".to_string(), "gone/b.rs".to_string()];
    let validity = compute_anchor_validity(&fs, Path::new(PROJECT), &anchors);
    let mut map = BTreeMap::new();
    map.insert(id.clone(), validity);

    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &map,
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    assert_eq!(
        candidates.first().map(|c| c.reason),
        Some(DemotionReason::AnchorDecay)
    );

    let outcome = demote(&ctx, &graph, &id, "anchor_decay")?;
    assert!(outcome.is_some());
    assert_eq!(
        ctx.store().read(&id)?.frontmatter.status()?,
        Status::Forgotten
    );
    Ok(())
}

/// Nota fundacional ancorada, criada em `created_ms` (sem shelf-life).
fn drift_note(statement: &str, created_ms: i64) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.classification = Some(Classification::Foundational);
    draft.anchors = vec!["src/x.rs".to_string()];
    draft.to_note(created_ms)
}

#[test]
fn a_dependent_of_a_retracted_premise_is_proposed() -> Result<()> {
    // TMS (D208): dependente de premissa esquecida cai no plano.
    let premise_id = id::note_id(NoteType::Fact, "premissa retratada");
    let mut dependent = Draft::new(NoteType::Fact, "dependente da premissa");
    dependent.edges = vec![(EdgeKind::DependsOn, premise_id)];
    let dependent = dependent.to_note(NOW)?;
    let dependent_id = dependent.id()?.to_string();
    let mut premise = Draft::new(NoteType::Fact, "premissa retratada");
    premise.status = Some(Status::Forgotten);
    let premise = premise.to_note(NOW)?;

    let notes = vec![dependent, premise];
    let graph = Graph::from_notes(notes.clone())?;
    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let validity = BTreeMap::new();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates.first().map(|c| c.id.as_str()),
        Some(dependent_id.as_str())
    );
    assert_eq!(
        candidates.first().map(|c| c.reason),
        Some(DemotionReason::Defeated)
    );
    Ok(())
}

#[test]
fn a_topic_whose_vocabulary_changed_is_proposed_as_drifted() -> Result<()> {
    // Drift KL/JS (D208): mesma âncora, vocabulário antigo × novo divergente.
    let notes = vec![
        drift_note("cache lru", NOW.saturating_sub(days(4)))?,
        drift_note("cache lru expira", NOW.saturating_sub(days(3)))?,
        drift_note("vetor hnsw", NOW.saturating_sub(days(2)))?,
        drift_note("vetor hnsw recall", NOW.saturating_sub(days(1)))?,
    ];
    let graph = Graph::from_notes(notes.clone())?;
    let shelf_life = ShelfLife::default();
    let decay = DecayPolicy::default();
    let validity = BTreeMap::new();
    let input = DemotionInput {
        now_ms: NOW,
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: None,
    };
    let candidates = demotion_candidates(&notes, &input, &graph)?;
    assert_eq!(candidates.len(), 4);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.reason == DemotionReason::Drifted),
        "todos deviam ser drifted: {candidates:?}"
    );
    Ok(())
}
