//! `update` versionado e supersede caminhável (E07-T03).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Status, Value};
use crate::write::{Patch, UpdateOutcome, history, update};

use super::{note, seeded};

#[test]
fn body_change_keeps_id_and_bumps_revision() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "alpha", "corpo")?;
    let id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;

    let patch = Patch {
        body: Some("corpo revisado".to_string()),
        ..Patch::default()
    };
    let outcome = update(&ctx, &id, &patch)?;
    assert!(matches!(outcome, UpdateOutcome::Revised { .. }));
    assert_eq!(outcome.id(), id);
    assert_eq!(outcome.revision(), 2);

    let stored = ctx.store().read(&id)?;
    assert_eq!(stored.body, "corpo revisado");
    assert_eq!(stored.revision(), 2);
    Ok(())
}

#[test]
fn type_change_supersedes_and_is_walkable() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "alpha", "corpo")?;
    let old_id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;

    let patch = Patch {
        note_type: Some(NoteType::Decision),
        ..Patch::default()
    };
    let outcome = update(&ctx, &old_id, &patch)?;
    assert!(matches!(outcome, UpdateOutcome::Superseded { .. }));
    let new_id = outcome.id().to_string();
    assert_ne!(new_id, old_id);

    let old = ctx.store().read(&old_id)?;
    assert_eq!(
        old.frontmatter.get("superseded_by").and_then(Value::as_str),
        Some(new_id.as_str())
    );
    assert_eq!(old.frontmatter.status()?, Status::Superseded);

    let new = ctx.store().read(&new_id)?;
    assert_eq!(
        new.frontmatter.string_list("replaces")?,
        vec![old_id.as_str()]
    );

    let chain = history(ctx.store(), &old_id)?;
    assert_eq!(chain.len(), 2);
    assert_eq!(
        chain.first().and_then(|note| note.id().ok()),
        Some(old_id.as_str())
    );
    assert_eq!(
        chain.last().and_then(|note| note.id().ok()),
        Some(new_id.as_str())
    );

    let forward = history(ctx.store(), &new_id)?;
    assert_eq!(forward.len(), 2);
    Ok(())
}

#[test]
fn manual_supersede_status_is_rejected() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "alpha", "")?;
    let id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;
    let patch = Patch {
        status: Some(Status::Superseded),
        ..Patch::default()
    };
    assert!(update(&ctx, &id, &patch).is_err());
    Ok(())
}
