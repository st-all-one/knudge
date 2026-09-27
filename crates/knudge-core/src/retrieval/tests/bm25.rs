//! BM25, pesos e boost (D35–D38).

use std::borrow::Cow;
use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::Result;
use crate::retrieval::stem::content_terms_stemmed;
use crate::retrieval::{Index, Postings, type_weight};
use crate::schema::NoteType;
use crate::store::Note;

use super::{base, note, tagged, with_outcomes};

fn allowed(index: &Index) -> BTreeSet<String> {
    index.docs.iter().map(|doc| doc.meta.id.clone()).collect()
}

/// Corpus pequeno com overlap em `statement`, `body` e `tags`, e um doc sem overlap.
fn corpus() -> Option<Index> {
    let in_statement = note(NoteType::Fact, "json decoder cache", "").ok()?;
    let in_body = note(NoteType::Fact, "grafo indice", "json decoder").ok()?;
    let in_tags = tagged(NoteType::Fact, "nota solta", &["json"]).ok()?;
    let disjoint = note(NoteType::Fact, "assunto alheio", "outro texto").ok()?;
    Index::build(&[in_statement, in_body, in_tags, disjoint]).ok()
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
fn accents_fold_to_ascii() -> Result<()> {
    let target = note(NoteType::Fact, "cafe com leite", "")?;
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
    assert!(type_weight(NoteType::Decision) > type_weight(NoteType::Epic));
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
    let with_boost = index.score_with("cache", &allowed, 0.0, |meta| {
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

#[test]
fn score_matches_manual_scan() -> Result<()> {
    let index = Index::build(&[
        note(NoteType::Fact, "json decoder", "")?,
        note(NoteType::Fact, "notas soltas", "json decoder")?,
        tagged(NoteType::Fact, "sem corpo", &["json"])?,
        note(NoteType::Fact, "assunto alheio", "outro texto")?,
    ])?;
    let allowed = allowed(&index);
    let terms = content_terms_stemmed("json decoder");

    let mut expected: Vec<(String, f64)> = index
        .docs
        .iter()
        .filter(|doc| allowed.contains(&doc.meta.id))
        .map(|doc| (doc.meta.id.clone(), index.score_doc(doc, &terms)))
        .filter(|(_, score)| *score > 0.0)
        .collect();
    expected.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let hits: Vec<(String, f64)> = index
        .score("json decoder", &allowed)
        .into_iter()
        .map(|hit| (hit.id, hit.score))
        .collect();
    assert_eq!(hits, expected);
    Ok(())
}

#[test]
fn term_ratio_counts_documents_with_the_term() -> Result<()> {
    let notes = vec![
        note(NoteType::Fact, "comum alfa", "")?,
        note(NoteType::Fact, "comum beta", "")?,
        note(NoteType::Fact, "raro gama", "")?,
    ];
    let index = Index::build(&notes)?;
    assert!((index.term_ratio("comum") - 2.0 / 3.0).abs() < 1e-9);
    assert!((index.term_ratio("raro") - 1.0 / 3.0).abs() < 1e-9);
    assert!(index.term_ratio("ausente").abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn high_frequency_terms_are_dropped_only_for_large_corpora() -> Result<()> {
    // 64 notas: `ubiquo` em 40 (62,5%) e `raro<i>` em uma cada.
    let mut notes = Vec::new();
    for serial in 0..64 {
        let statement = if serial < 40 {
            format!("ubiquo raro{serial}")
        } else {
            format!("outro raro{serial}")
        };
        notes.push(note(NoteType::Fact, &statement, "")?);
    }
    let index = Index::build(&notes)?;
    let allowed = allowed(&index);
    // Sem corte: `ubiquo` casa 40 notas.
    assert_eq!(index.score("ubiquo", &allowed).len(), 40);
    // Com corte (0.5): `ubiquo` some (df/N = 62,5% ≥ 50%).
    assert!(
        index
            .score_with("ubiquo", &allowed, 0.5, |_| 0.0)
            .is_empty()
    );
    // `raro0` não é ubíquo: segue casando, na ordem canônica.
    assert_eq!(index.score_with("raro0", &allowed, 0.5, |_| 0.0).len(), 1);
    Ok(())
}

#[test]
fn high_frequency_cutoff_is_noop_below_min_corpus() -> Result<()> {
    // Corpus pequeno (< MIN_CUTOFF_CORPUS): mesmo com ratio 0.5, nada é cortado.
    let notes = vec![
        note(NoteType::Fact, "ubiquo alfa", "")?,
        note(NoteType::Fact, "ubiquo beta", "")?,
    ];
    let index = Index::build(&notes)?;
    let allowed = allowed(&index);
    assert_eq!(index.score_with("ubiquo", &allowed, 0.5, |_| 0.0).len(), 2);
    Ok(())
}

proptest! {
    #[test]
    fn sieve_positions_match_scan(terms in prop::collection::vec("[a-z]{2,6}", 1..5)) {
        if let Some(index) = corpus() {
            let postings = Postings::build(&index);
            let borrowed: Vec<Cow<'_, str>> =
                terms.iter().map(|term| Cow::Borrowed(term.as_str())).collect();
            let positions = postings.sieve(&borrowed);
            prop_assert!(
                positions
                    .windows(2)
                    .all(|window| matches!(window, [a, b] if a < b))
            );
            let expected: Vec<u32> = index
                .docs
                .iter()
                .enumerate()
                .filter(|(_, doc)| index.score_doc(doc, &borrowed) > 0.0)
                .filter_map(|(position, _)| u32::try_from(position).ok())
                .collect();
            prop_assert_eq!(positions, expected);
        }
    }
}
