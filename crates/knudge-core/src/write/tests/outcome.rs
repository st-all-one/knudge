//! `outcomes[]` generalizado para qualquer nota (D103).

use std::str::FromStr;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::write::outcome::{OutcomeStatus, outcome};

use super::{note, seeded};

#[test]
fn outcome_is_appended_to_a_knowledge_note() -> Result<()> {
    let fs = MemFs::new();
    let original = note(NoteType::Fact, "o cache usa body_hash", "")?;
    let id = original.id()?.to_string();
    let ctx = seeded(&fs, &[original])?;

    assert_eq!(outcome(&ctx, &id, OutcomeStatus::Success, Some("ok"))?, 2);

    let stored = ctx.store().read(&id)?;
    let entries = stored
        .frontmatter
        .get("outcomes")
        .and_then(|value| value.as_list())
        .map(<[_]>::len);
    assert_eq!(entries, Some(1));
    Ok(())
}

#[test]
fn outcome_status_round_trips() -> Result<()> {
    for status in [
        OutcomeStatus::Success,
        OutcomeStatus::Partial,
        OutcomeStatus::Failure,
        OutcomeStatus::Abandoned,
    ] {
        assert_eq!(OutcomeStatus::from_str(status.as_str())?, status);
    }
    assert!(OutcomeStatus::from_str("nope").is_err());
    Ok(())
}
