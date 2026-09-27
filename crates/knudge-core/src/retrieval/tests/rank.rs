//! Ranking por confiança derivada, sem query (K2/D107).

use crate::Result;
use crate::lifecycle::{DriftEntry, DriftIndex};
use crate::retrieval::{Filter, RankQuery, Universe, Why, rank};
use crate::schema::{EdgeKind, NoteType, Scope};
use crate::store::Note;

use super::{
    anchored, base, built, note, with_anchors, with_created, with_edges, with_outcomes, with_scope,
};

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
    let (index, graph) = built(vec![plain, confirmed])?;

    let hits = rank(&index, &graph, &Filter::new(), &query(Universe::All, 0.0));
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
fn drift_reduces_confidence_without_query() -> Result<()> {
    // E19/T01b/D203: âncoras quebradas descontam a confiança mesmo no `rank` (`similarity = 0`).
    let clean = note(NoteType::Fact, "nota limpa", "")?;
    let drifted = note(NoteType::Fact, "nota com drift", "")?;
    let clean_id = clean.id()?.to_string();
    let drifted_id = drifted.id()?.to_string();
    let (index, graph) = built(vec![clean, drifted])?;

    let query = RankQuery {
        universe: Universe::All,
        drift: DriftIndex::new(&[DriftEntry {
            id: drifted_id.clone(),
            drift: 1.0,
        }]),
        ..RankQuery::default()
    };
    let hits = rank(&index, &graph, &Filter::new(), &query);
    let confidence = |id: &str| {
        hits.iter()
            .find(|hit| hit.id == id)
            .map(|hit| hit.confidence)
    };
    let clean_confidence = confidence(&clean_id).unwrap_or(0.0);
    let drifted_confidence = confidence(&drifted_id).unwrap_or(0.0);
    assert!(
        drifted_confidence < clean_confidence,
        "drift {drifted_confidence} deveria ser menor que {clean_confidence}"
    );
    Ok(())
}

#[test]
fn rank_prefers_more_successes_over_fewer() -> Result<()> {
    // D189: a confiança é o limite inferior do posterior Beta — 20 sucessos > 1 sucesso.
    let many = Note::new(
        with_outcomes(
            base(NoteType::Decision, "vinte sucessos")?,
            &["success"; 20],
        )?,
        "",
    );
    let one = Note::new(
        with_outcomes(base(NoteType::Decision, "um sucesso")?, &["success"])?,
        "",
    );
    let many_id = many.id()?.to_string();
    let one_id = one.id()?.to_string();
    let (index, graph) = built(vec![one, many])?;

    let hits = rank(&index, &graph, &Filter::new(), &query(Universe::All, 0.0));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(many_id.as_str())
    );
    assert_eq!(
        hits.last().map(|hit| hit.id.as_str()),
        Some(one_id.as_str())
    );
    Ok(())
}

#[test]
fn rank_penalizes_failures() -> Result<()> {
    let clean = Note::new(
        with_outcomes(
            base(NoteType::Decision, "tres sucessos")?,
            &["success", "success", "success"],
        )?,
        "",
    );
    let mixed = Note::new(
        with_outcomes(
            base(NoteType::Decision, "um sucesso duas falhas")?,
            &["success", "failure", "failure"],
        )?,
        "",
    );
    let clean_id = clean.id()?.to_string();
    let mixed_id = mixed.id()?.to_string();
    let (index, graph) = built(vec![clean, mixed])?;

    let hits = rank(&index, &graph, &Filter::new(), &query(Universe::All, 0.0));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(clean_id.as_str())
    );
    assert_eq!(
        hits.last().map(|hit| hit.id.as_str()),
        Some(mixed_id.as_str())
    );
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
    let (index, graph) = built(vec![task, plain, confirmed])?;

    let hits = rank(
        &index,
        &graph,
        &Filter::new(),
        &query(Universe::Knowledge, 0.1),
    );
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
    let (index, graph) = built(vec![fact, decision])?;
    let filter = Filter {
        types: vec![NoteType::Fact],
        ..Filter::new()
    };

    let hits = rank(&index, &graph, &filter, &query(Universe::All, 0.0));
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(fact_id.as_str())
    );
    Ok(())
}

#[test]
fn rank_prefers_recent_when_evidence_ties() -> Result<()> {
    // D175: sem query (`similarity = 0`), a idade entra aditiva — evidência igual, recente vence.
    let now = 1_700_000_000_000;
    let day = 86_400_000;
    let recent = Note::new(
        with_created(base(NoteType::Fact, "fato recente")?, now - 1_000)?,
        "",
    );
    let old = Note::new(
        with_created(base(NoteType::Fact, "fato antigo")?, now - 400 * day)?,
        "",
    );
    let recent_id = recent.id()?.to_string();
    let old_id = old.id()?.to_string();
    let (index, graph) = built(vec![old, recent])?;
    let query = RankQuery {
        now_ms: Some(now),
        ..query(Universe::All, 0.0)
    };

    let hits = rank(&index, &graph, &Filter::new(), &query);
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(recent_id.as_str())
    );
    assert_eq!(
        hits.last().map(|hit| hit.id.as_str()),
        Some(old_id.as_str())
    );
    Ok(())
}

#[test]
fn rank_demotes_the_losing_side_of_a_contradiction() -> Result<()> {
    // D177: o lado de menor confiança de uma aresta `contradicts` declarada é rebaixado.
    let strong = Note::new(
        with_outcomes(base(NoteType::Decision, "cache com lru")?, &["success"; 5])?,
        "",
    );
    let strong_id = strong.id()?.to_string();
    let weak = Note::new(
        with_edges(
            base(NoteType::Fact, "cache sem lru")?,
            &[(EdgeKind::Contradicts, &strong_id)],
        )?,
        "",
    );
    let weak_id = weak.id()?.to_string();
    let other = note(NoteType::Fact, "fato neutro", "")?;
    let other_id = other.id()?.to_string();
    let (index, graph) = built(vec![strong, weak, other])?;

    let hits = rank(&index, &graph, &Filter::new(), &query(Universe::All, 0.0));
    assert_eq!(
        hits.first().map(|hit| hit.id.as_str()),
        Some(strong_id.as_str())
    );
    let weak_hit = hits.iter().find(|hit| hit.id == weak_id);
    let other_hit = hits.iter().find(|hit| hit.id == other_id);
    let (Some(weak_hit), Some(other_hit)) = (weak_hit, other_hit) else {
        return Ok(());
    };
    assert!(
        weak_hit.confidence < other_hit.confidence,
        "perdedor devia ser rebaixado: {weak_hit:?} vs {other_hit:?}"
    );
    Ok(())
}

#[test]
fn rank_breaks_ties_by_id() -> Result<()> {
    let alpha = note(NoteType::Fact, "alpha", "")?;
    let beta = note(NoteType::Fact, "beta", "")?;
    let (index, graph) = built(vec![beta, alpha])?;

    let hits = rank(&index, &graph, &Filter::new(), &query(Universe::All, 0.0));
    let ids: Vec<&str> = hits.iter().map(|hit| hit.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
    Ok(())
}
