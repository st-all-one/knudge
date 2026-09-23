//! BM25, pesos e boost (D35–D38).

use std::collections::BTreeSet;

use crate::Result;
use crate::retrieval::{Index, type_weight};
use crate::schema::NoteType;
use crate::store::Note;

use super::{base, note, with_outcomes};

fn allowed(index: &Index) -> BTreeSet<String> {
    index.docs.iter().map(|doc| doc.meta.id.clone()).collect()
}

fn id_of(note: &Note) -> Result<String> {
    note.id().map(str::to_string)
}

#[test]
fn statement_dominates_body() -> Result<()> {
    let in_statement = note(NoteType::Fact, "json decoder", "")?;
    let in_body = note(NoteType::Fact, "notas soltas", "json decoder")?;
    let expected = id_of(&in_statement)?;

    let index = Index::build(&[in_statement, in_body])?;
    let hits = index.score("json decoder", &allowed(&index));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(expected.as_str())
    );
    Ok(())
}

#[test]
fn accents_collapse_to_ascii_prefix() -> Result<()> {
    let target = note(NoteType::Fact, "caf com leite", "")?;
    let expected = id_of(&target)?;
    let index = Index::build(&[target])?;
    let hits = index.score("café", &allowed(&index));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(expected.as_str())
    );
    Ok(())
}

#[test]
fn confirmation_boost_changes_order() -> Result<()> {
    let plain = note(NoteType::Fact, "cache alfa", "")?;
    let plain_id = id_of(&plain)?;
    let boosted = Note::new(
        with_outcomes(base(NoteType::Fact, "cache beta")?, &["success"])?,
        "",
    );
    let boosted_id = id_of(&boosted)?;

    let index = Index::build(&[plain, boosted])?;
    let hits = index.score("cache", &allowed(&index));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(boosted_id.as_str())
    );
    assert_eq!(
        hits.get(1).map(|hit| hit.id.as_str()),
        Some(plain_id.as_str())
    );
    Ok(())
}

#[test]
fn type_weight_orders_decision_above_container() {
    assert!(type_weight(NoteType::Decision) > type_weight(NoteType::Container));
    assert!(type_weight(NoteType::Fact) > type_weight(NoteType::Link));
}

#[test]
fn empty_query_yields_no_hits() -> Result<()> {
    let index = Index::build(&[note(NoteType::Fact, "alpha", "")?])?;
    assert!(index.score("   ", &allowed(&index)).is_empty());
    Ok(())
}

#[test]
fn task_boost_raises_score() -> Result<()> {
    let plain = note(NoteType::Fact, "cache alfa", "")?;
    let boosted = note(NoteType::Fact, "cache beta", "")?;
    let boosted_id = id_of(&boosted)?;
    let index = Index::build(&[plain, boosted])?;
    let allowed = allowed(&index);

    let base = index.score("cache", &allowed);
    let with_boost = index.score_with("cache", &allowed, |meta| {
        if meta.id == boosted_id { 0.5 } else { 0.0 }
    });
    let base_score = base
        .iter()
        .find(|hit| hit.id == boosted_id)
        .map(|hit| hit.score);
    let boosted_score = with_boost
        .iter()
        .find(|hit| hit.id == boosted_id)
        .map(|hit| hit.score);
    assert!(boosted_score > base_score, "boost não alterou o score");
    Ok(())
}
