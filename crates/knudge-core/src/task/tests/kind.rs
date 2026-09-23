//! Espécie (`--kind`) relaxa D93 (D113).

use crate::Result;
use crate::schema::{NoteType, Scope};
use crate::task::{TaskSpec, WORK_KINDS, validate_kind};

#[test]
fn work_kinds_are_accepted_for_issues() -> Result<()> {
    for kind in WORK_KINDS {
        validate_kind(Scope::Issue, Some(kind))?;
        validate_kind(Scope::Task, Some(kind))?;
    }
    Ok(())
}

#[test]
fn work_kinds_match_is_work_kind() {
    // O array público e o predicado canônico não podem divergir (D113/D120).
    for kind in NoteType::ALL {
        assert_eq!(WORK_KINDS.contains(&kind), kind.is_work_kind());
    }
}

#[test]
fn containers_only_accept_container_kind() -> Result<()> {
    validate_kind(Scope::Epic, Some(NoteType::Container))?;
    assert!(validate_kind(Scope::Epic, Some(NoteType::Error)).is_err());
    assert!(validate_kind(Scope::Issue, Some(NoteType::Container)).is_err());
    Ok(())
}

#[test]
fn kind_sets_note_type() -> Result<()> {
    let mut spec = TaskSpec::new(Scope::Issue, "corrigir off-by-one");
    spec.kind = Some(NoteType::Error);
    let note = spec.to_note(super::NOW)?;
    assert_eq!(note.frontmatter.note_type()?, NoteType::Error);
    Ok(())
}
