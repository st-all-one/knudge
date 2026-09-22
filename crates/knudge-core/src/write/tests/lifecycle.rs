//! `link`, `forget` e `restore` (E07-T04).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::{EdgeKind, NoteType, Status, Value};
use crate::write::{forget, link, restore};

use super::{note, seeded};

#[test]
fn forget_is_soft_and_restore_reverses() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "alpha", "")?;
    let id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;

    let revision = forget(&ctx, &id, Some("obsoleto"))?;
    assert_eq!(revision, 2);
    assert!(ctx.store().exists(&id));
    assert_eq!(
        ctx.store().read(&id)?.frontmatter.status()?,
        Status::Forgotten
    );

    let revision = restore(&ctx, &id)?;
    assert_eq!(revision, 3);
    assert_eq!(ctx.store().read(&id)?.frontmatter.status()?, Status::Active);
    Ok(())
}

#[test]
fn invalid_status_transitions_are_rejected() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "alpha", "")?;
    let id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;

    assert!(restore(&ctx, &id).is_err());
    forget(&ctx, &id, None)?;
    assert!(forget(&ctx, &id, None).is_err());
    Ok(())
}

#[test]
fn link_adds_edge_once() -> Result<()> {
    let fs = MemFs::new();
    let from = note(NoteType::Fact, "origem", "")?;
    let to = note(NoteType::Fact, "destino", "")?;
    let from_id = from.id()?.to_string();
    let to_id = to.id()?.to_string();
    let ctx = seeded(&fs, &[from, to])?;

    assert!(link(&ctx, &from_id, EdgeKind::References, &to_id)?);
    assert!(!link(&ctx, &from_id, EdgeKind::References, &to_id)?);
    let stored = ctx.store().read(&from_id)?;
    assert_eq!(
        stored.frontmatter.string_list("references")?,
        vec![to_id.as_str()]
    );
    assert_eq!(stored.revision(), 2);
    Ok(())
}

#[test]
fn replaces_link_sets_reverse_pointer() -> Result<()> {
    let fs = MemFs::new();
    let new = note(NoteType::Fact, "novo", "")?;
    let old = note(NoteType::Fact, "antigo", "")?;
    let new_id = new.id()?.to_string();
    let old_id = old.id()?.to_string();
    let ctx = seeded(&fs, &[new, old])?;

    assert!(link(&ctx, &new_id, EdgeKind::Replaces, &old_id)?);
    let target = ctx.store().read(&old_id)?;
    assert_eq!(
        target
            .frontmatter
            .get("superseded_by")
            .and_then(Value::as_str),
        Some(new_id.as_str())
    );
    assert_eq!(target.frontmatter.status()?, Status::Superseded);
    Ok(())
}

#[test]
fn link_rejects_invalid_target() -> Result<()> {
    let fs = MemFs::new();
    let from = note(NoteType::Fact, "origem", "")?;
    let from_id = from.id()?.to_string();
    let ctx = seeded(&fs, &[from])?;
    assert!(link(&ctx, &from_id, EdgeKind::References, "nope").is_err());
    Ok(())
}
