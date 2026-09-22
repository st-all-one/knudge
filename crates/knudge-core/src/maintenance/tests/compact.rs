//! `compact` como proposta (E08-T08).

use crate::maintenance::compact::{apply_compact, propose_compact};
use crate::schema::{NoteType, Status};
use crate::write::dedup::DedupThresholds;
use crate::{Error, Result};

use super::{MemFs, context, note};

#[test]
fn proposal_does_not_write() -> Result<()> {
    let fs = MemFs::new();
    let notes = [
        note(NoteType::Fact, "alpha beta gamma")?,
        note(NoteType::Fact, "gamma beta alpha")?,
    ];
    let ctx = context(&fs, &notes)?;
    let before = ctx.store().list_ids()?.len();
    let proposals = propose_compact(ctx.store(), ctx.index(), &DedupThresholds::default());
    assert_eq!(proposals.len(), 1);
    assert_eq!(ctx.store().list_ids()?.len(), before);
    Ok(())
}

#[test]
fn apply_merges_and_forgets() -> Result<()> {
    let fs = MemFs::new();
    let notes = [
        note(NoteType::Fact, "alpha beta gamma")?,
        note(NoteType::Fact, "gamma beta alpha")?,
    ];
    let ctx = context(&fs, &notes)?;
    let proposals = propose_compact(ctx.store(), ctx.index(), &DedupThresholds::default());
    let proposal = proposals
        .first()
        .ok_or_else(|| Error::internal("sem proposta"))?;
    let outcome = apply_compact(&ctx, proposal)?;
    assert_eq!(outcome.forgotten.len(), 1);
    let drop_id = outcome.forgotten.first().cloned().unwrap_or_default();
    assert_eq!(
        ctx.store().read(&drop_id)?.frontmatter.status()?,
        Status::Forgotten
    );
    assert!(ctx.store().exists(&outcome.kept));
    Ok(())
}

#[test]
fn distinct_notes_yield_no_proposal() -> Result<()> {
    let fs = MemFs::new();
    let notes = [
        note(NoteType::Fact, "alpha beta")?,
        note(NoteType::Fact, "zeta eta")?,
    ];
    let ctx = context(&fs, &notes)?;
    assert!(propose_compact(ctx.store(), ctx.index(), &DedupThresholds::default()).is_empty());
    Ok(())
}
