//! Protocolo de dedup em duas fases (E07-T02).

use crate::Result;
use crate::config::{Config, ConfigValue};
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Value};
use crate::write::{
    DedupDecision, DedupThresholds, Draft, WriteAction, propose, thresholds_from_config, write,
};

use super::{note, seeded};

#[test]
fn bands_create_merge_and_reject() -> Result<()> {
    let fs = MemFs::new();
    let candidate = note(NoteType::Fact, "alpha beta gamma delta", "")?;
    let candidate_id = candidate.id()?.to_string();
    let ctx = seeded(&fs, &[candidate])?;
    let thresholds = DedupThresholds::default();

    let create = write(
        &ctx,
        &Draft::new(NoteType::Fact, "alpha zeta eta theta iota"),
        &thresholds,
    )?;
    assert_eq!(create.action, WriteAction::Created);

    let merge = write(
        &ctx,
        &Draft::new(NoteType::Fact, "alpha beta gamma delta epsilon"),
        &thresholds,
    )?;
    assert_eq!(merge.action, WriteAction::Merged);
    assert_eq!(merge.id, candidate_id);
    assert_eq!(merge.revision, Some(2));

    let reject = write(
        &ctx,
        &Draft::new(NoteType::Fact, "delta gamma beta alpha"),
        &thresholds,
    )?;
    assert_eq!(reject.action, WriteAction::Rejected);
    assert_eq!(reject.id, candidate_id);
    Ok(())
}

#[test]
fn proposal_does_not_write() -> Result<()> {
    let fs = MemFs::new();
    let candidate = note(NoteType::Fact, "alpha beta gamma delta", "")?;
    let ctx = seeded(&fs, &[candidate])?;
    let before = ctx.store().list_ids()?.len();
    let draft = Draft::new(NoteType::Fact, "alpha beta gamma delta epsilon");
    let proposal = propose(ctx.index(), &draft, &DedupThresholds::default())?;
    assert_eq!(proposal.candidates.len(), 1);
    assert!(matches!(proposal.decision, DedupDecision::Merge { .. }));
    assert_eq!(ctx.store().list_ids()?.len(), before);
    Ok(())
}

#[test]
fn propose_ignores_forgotten_candidate() -> Result<()> {
    let fs = MemFs::new();
    let mut candidate = note(NoteType::Fact, "alpha beta gamma delta", "")?;
    candidate
        .frontmatter
        .set("status", Value::Str("forgotten".to_string()))?;
    let ctx = seeded(&fs, &[candidate])?;
    let draft = Draft::new(NoteType::Fact, "alpha beta gamma delta epsilon");

    let proposal = propose(ctx.index(), &draft, &DedupThresholds::default())?;
    assert_eq!(proposal.decision, DedupDecision::Create);
    assert!(proposal.candidates.is_empty());
    Ok(())
}

#[test]
fn thresholds_are_configurable() -> Result<()> {
    let fs = MemFs::new();
    let candidate = note(NoteType::Fact, "alpha beta gamma delta", "")?;
    let ctx = seeded(&fs, &[candidate])?;
    let draft = Draft::new(NoteType::Fact, "alpha beta gamma delta epsilon");

    let strict = DedupThresholds::new(0.95, 0.99)?;
    let proposal = propose(ctx.index(), &draft, &strict)?;
    assert_eq!(proposal.decision, DedupDecision::Create);

    let mut config = Config::defaults();
    config.set_value("dedup.create_below", ConfigValue::Float(0.5))?;
    let from_config = thresholds_from_config(&config)?;
    assert!((from_config.create_below - 0.5).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn invalid_thresholds_are_rejected() {
    assert!(DedupThresholds::new(0.9, 0.5).is_err());
    assert!(DedupThresholds::new(-0.1, 0.9).is_err());
    assert!(DedupThresholds::new(0.5, 1.5).is_err());
}
