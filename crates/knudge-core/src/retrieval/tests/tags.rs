//! Vocabulário de tags (`kd ask --tags`, D107).

use crate::Result;
use crate::retrieval::{Index, tag_counts};
use crate::schema::{NoteType, Status};
use crate::store::Note;

use super::{base, tagged, with_status, with_tags};

#[test]
fn tag_counts_orders_by_count_then_name() -> Result<()> {
    let notes = [
        tagged(NoteType::Fact, "a", &["retry", "queue"])?,
        tagged(NoteType::Fact, "b", &["retry"])?,
        tagged(NoteType::Fact, "c", &["alpha"])?,
    ];
    let index = Index::build(&notes)?;

    let counts = tag_counts(&index);
    assert_eq!(
        counts.first().map(|(tag, count)| (tag.as_str(), *count)),
        Some(("retry", 2))
    );
    assert_eq!(counts.get(1).map(|(tag, _)| tag.as_str()), Some("alpha"));
    assert_eq!(counts.get(2).map(|(tag, _)| tag.as_str()), Some("queue"));
    Ok(())
}

#[test]
fn tag_counts_ignores_forgotten_and_superseded() -> Result<()> {
    let forgotten = with_status(
        with_tags(base(NoteType::Fact, "a")?, &["morto"])?,
        Status::Forgotten,
    )?;
    let superseded = with_status(
        with_tags(base(NoteType::Fact, "b")?, &["morto"])?,
        Status::Superseded,
    )?;
    let index = Index::build(&[Note::new(forgotten, ""), Note::new(superseded, "")])?;
    assert!(tag_counts(&index).is_empty());
    Ok(())
}
