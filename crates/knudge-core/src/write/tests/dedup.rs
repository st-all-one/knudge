//! Protocolo de dedup em duas fases (E07-T02).

use crate::Result;
use crate::config::{Config, ConfigValue};
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{NoteType, Scope, Value};
use crate::store::Note;
use crate::write::{
    DedupDecision, DedupThresholds, Draft, WriteAction, propose, propose_merges,
    thresholds_from_config, write,
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

#[test]
fn scoped_items_compare_statement_only() -> Result<()> {
    // Corpo idêntico (template de import) não deve tornar duas tarefas distintas duplicatas.
    let body = "zero um dois tres quatro cinco seis sete oito nove dez onze doze treze catorze quinze dezasseis dezassete dezoito dezanove";
    let first = scoped_note("fmt V2 Services Alpha", body)?;
    let second = scoped_note("fmt V2 Services Beta", body)?;
    let index = Index::build(&[first, second])?;
    let proposals = propose_merges(&index, &DedupThresholds::default());
    assert!(
        proposals.is_empty(),
        "corpo template gerou falso positivo: {proposals:?}"
    );
    Ok(())
}

fn scoped_note(statement: &str, body: &str) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Task, statement).with_body(body);
    draft.scope = Some(Scope::Task);
    draft.to_note(super::NOW)
}
