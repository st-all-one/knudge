//! Ranking por confiança derivada, sem query (K2/D107).

use crate::Result;
use crate::retrieval::{Filter, Index, RankQuery, Universe, Why, rank};
use crate::schema::{NoteType, Scope};
use crate::store::Note;

use super::{anchored, base, note, with_anchors, with_outcomes, with_scope};

fn query(universe: Universe, task_weight: f64) -> RankQuery {
    RankQuery {
        universe,
        task_weight,
        ..RankQuery::default()
    }
}

#[test]
fn rank_orders_confirmed_before_plain() -> Result<()> {
    let confirmed = Note::new(
        with_outcomes(
            base(NoteType::Decision, "backoff com jitter")?,
            &["success", "success"],
        )?,
        "",
    );
    let plain = note(NoteType::Fact, "sem evidência", "")?;
    let confirmed_id = confirmed.id()?.to_string();
    let plain_id = plain.id()?.to_string();
    let index = Index::build(&[plain, confirmed])?;

    let hits = rank(&index, &Filter::new(), &query(Universe::All, 0.0));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(confirmed_id.as_str())
    );
    assert_eq!(
        hits.last().map(|hit| hit.id.as_str()),
        Some(plain_id.as_str())
    );
    assert!(hits.iter().all(|hit| (0.0..=1.0).contains(&hit.confidence)));
    assert_eq!(hits.first().map(|hit| hit.why), Some(Why::Stars));
    Ok(())
}

#[test]
fn rank_promotes_task_confirmed_note() -> Result<()> {
    let task = Note::new(
        with_anchors(
            with_scope(
                with_outcomes(base(NoteType::Task, "implementar retry")?, &["success"])?,
                Scope::Task,
            )?,
            &["src/retry.ts"],
        )?,
        "",
    );
    let confirmed = anchored(NoteType::Decision, "jitter alfa", &["src/retry.ts"])?;
    let plain = note(NoteType::Fact, "jitter beta", "")?;
    let confirmed_id = confirmed.id()?.to_string();
    let plain_id = plain.id()?.to_string();
    let index = Index::build(&[task, plain, confirmed])?;

    let hits = rank(&index, &Filter::new(), &query(Universe::Knowledge, 0.1));
    // A tarefa fica de fora (universo de conhecimento) e a nota confirmada sobe.
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(confirmed_id.as_str())
    );
    assert_eq!(hits.first().map(|hit| hit.why), Some(Why::Stars));
    assert_eq!(
        hits.last().map(|hit| hit.id.as_str()),
        Some(plain_id.as_str())
    );
    Ok(())
}

#[test]
fn rank_respects_filter() -> Result<()> {
    let fact = note(NoteType::Fact, "alpha", "")?;
    let decision = note(NoteType::Decision, "beta", "")?;
    let fact_id = fact.id()?.to_string();
    let index = Index::build(&[fact, decision])?;
    let filter = Filter {
        types: vec![NoteType::Fact],
        ..Filter::new()
    };

    let hits = rank(&index, &filter, &query(Universe::All, 0.0));
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(fact_id.as_str())
    );
    Ok(())
}

#[test]
fn rank_breaks_ties_by_id() -> Result<()> {
    let alpha = note(NoteType::Fact, "alpha", "")?;
    let beta = note(NoteType::Fact, "beta", "")?;
    let index = Index::build(&[beta, alpha])?;

    let hits = rank(&index, &Filter::new(), &query(Universe::All, 0.0));
    let ids: Vec<&str> = hits.iter().map(|hit| hit.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
    Ok(())
}
