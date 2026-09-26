//! Dedup semântico eventual: só propostas, nunca merge silencioso (E07-T06).

use std::collections::BTreeSet;

use proptest::prelude::*;
use proptest::sample::select;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::{Index, NoteDoc};
use crate::schema::{NoteType, Status, Value};
use crate::write::dedup::{MAX_CANDIDATES, dice, dice_statement};
use crate::write::{DedupThresholds, Draft, MergeProposal, propose_merges};

use super::{NOW, note, seeded};

#[test]
fn near_duplicates_become_proposals() -> Result<()> {
    let fs = MemFs::new();
    let first = Draft::new(NoteType::Fact, "alpha beta gamma delta").to_note(NOW - 1_000)?;
    let second = Draft::new(NoteType::Fact, "gamma delta beta alpha").to_note(NOW)?;
    let first_id = first.id()?.to_string();
    let second_id = second.id()?.to_string();
    let ctx = seeded(&fs, &[first, second])?;

    let before = ctx.store().list_ids()?.len();
    let proposals = propose_merges(ctx.index(), &DedupThresholds::default());
    assert_eq!(proposals.len(), 1);
    let proposal = proposals.first();
    assert!(proposal.is_some());
    assert_eq!(proposal.map(|p| p.keep.as_str()), Some(first_id.as_str()));
    assert_eq!(proposal.map(|p| p.drop.as_str()), Some(second_id.as_str()));
    assert_eq!(ctx.store().list_ids()?.len(), before);
    Ok(())
}

#[test]
fn forgotten_and_superseded_are_not_proposed() -> Result<()> {
    let fs = MemFs::new();
    let mut first = note(NoteType::Fact, "alpha beta gamma delta", "")?;
    first
        .frontmatter
        .set("status", Value::Str("forgotten".to_string()))?;
    let mut second = note(NoteType::Fact, "gamma delta beta alpha", "")?;
    second
        .frontmatter
        .set("status", Value::Str("superseded".to_string()))?;
    let ctx = seeded(&fs, &[first, second])?;

    let proposals = propose_merges(ctx.index(), &DedupThresholds::default());
    assert!(proposals.is_empty());
    Ok(())
}

#[test]
fn distinct_notes_yield_no_proposal() -> Result<()> {
    let fs = MemFs::new();
    let first = note(NoteType::Fact, "alpha beta", "")?;
    let second = note(NoteType::Fact, "zeta eta", "")?;
    let ctx = seeded(&fs, &[first, second])?;
    let proposals = propose_merges(ctx.index(), &DedupThresholds::default());
    assert!(proposals.is_empty());
    Ok(())
}

/// Referência O(N²) (pré-O3): `Index::score` completo + `find` linear. Trava a peneira.
fn reference(index: &Index, thresholds: &DedupThresholds) -> Vec<MergeProposal> {
    let allowed: BTreeSet<String> = index
        .docs
        .iter()
        .filter(|doc| !matches!(doc.meta.status, Status::Forgotten | Status::Superseded))
        .map(|doc| doc.meta.id.clone())
        .collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proposals = Vec::new();
    for doc in &index.docs {
        if !allowed.contains(&doc.meta.id) {
            continue;
        }
        for hit in index
            .score(&doc.statement, &allowed)
            .into_iter()
            .take(MAX_CANDIDATES)
        {
            if hit.id == doc.meta.id {
                continue;
            }
            let Some(other) = index.docs.iter().find(|other| other.meta.id == hit.id) else {
                continue;
            };
            let scoped = doc.meta.scope.is_some() || other.meta.scope.is_some();
            let (score, basis) = if scoped {
                (dice_statement(doc, other), "statement")
            } else {
                (dice(doc, other), "similaridade")
            };
            if score < thresholds.merge_below {
                continue;
            }
            let (keep, drop) = reference_order(doc, other);
            if !seen.insert((keep.clone(), drop.clone())) {
                continue;
            }
            proposals.push(MergeProposal {
                keep,
                drop,
                score,
                reason: format!("{basis} {score:.2} ≥ {:.2}", thresholds.merge_below),
            });
        }
    }
    proposals.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.keep.cmp(&b.keep))
            .then_with(|| a.drop.cmp(&b.drop))
    });
    proposals
}

fn reference_order(a: &NoteDoc, b: &NoteDoc) -> (String, String) {
    let a_key = (a.meta.created_ms, a.meta.id.as_str());
    let b_key = (b.meta.created_ms, b.meta.id.as_str());
    if a_key <= b_key {
        (a.meta.id.clone(), b.meta.id.clone())
    } else {
        (b.meta.id.clone(), a.meta.id.clone())
    }
}

proptest! {
    /// A peneira de postings (O3) propõe exatamente os mesmos pares, na mesma ordem, que a
    /// varredura completa anterior.
    #[test]
    fn sieve_matches_reference(
        rows in prop::collection::vec(
            (
                prop::collection::vec(
                    select(vec![
                        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
                    ]),
                    2..5,
                ),
                0_i64..4,
            ),
            2..10,
        ),
    ) {
        let fs = MemFs::new();
        let mut notes = Vec::new();
        for (position, (words, day)) in rows.iter().enumerate() {
            let offset = i64::try_from(position).unwrap_or(0);
            let created = NOW
                .saturating_add(day.saturating_mul(1_000))
                .saturating_add(offset);
            let statement = format!("{} item{position}", words.join(" "));
            let Ok(note) = Draft::new(NoteType::Fact, statement).to_note(created) else {
                continue;
            };
            notes.push(note);
        }
        if let Ok(ctx) = seeded(&fs, &notes) {
            let fast = propose_merges(ctx.index(), &DedupThresholds::default());
            let slow = reference(ctx.index(), &DedupThresholds::default());
            prop_assert_eq!(fast, slow);
        }
    }
}
