//! Dedup semântico eventual: só propostas, nunca merge silencioso (E07-T06).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::write::{DedupThresholds, Draft, propose_merges};

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
fn distinct_notes_yield_no_proposal() -> Result<()> {
    let fs = MemFs::new();
    let first = note(NoteType::Fact, "alpha beta", "")?;
    let second = note(NoteType::Fact, "zeta eta", "")?;
    let ctx = seeded(&fs, &[first, second])?;
    let proposals = propose_merges(ctx.index(), &DedupThresholds::default());
    assert!(proposals.is_empty());
    Ok(())
}
