//! Testes do drift de termos KL/JS (E19-T10/D208).

use std::collections::BTreeMap;

use super::note_created;
use crate::Result;
use crate::lifecycle::{js_divergence, kl_divergence, term_distribution, topic_drift};
use crate::schema::NoteType;

fn distribution(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs
        .iter()
        .map(|(term, probability)| ((*term).to_string(), *probability))
        .collect()
}

#[test]
fn identical_distributions_have_zero_js() {
    let p = distribution(&[("cache", 0.5), ("vetor", 0.5)]);
    assert!(js_divergence(&p, &p).abs() < 1e-9);
    assert!(kl_divergence(&p, &p).abs() < 1e-9);
}

#[test]
fn disjoint_distributions_have_positive_js() {
    let p = distribution(&[("cache", 1.0)]);
    let q = distribution(&[("vetor", 1.0)]);
    assert!(js_divergence(&p, &q) > 0.5);
    assert!(js_divergence(&p, &q) <= 1.0 + 1e-9);
}

#[test]
fn js_is_symmetric() {
    let p = distribution(&[("a", 0.7), ("b", 0.3)]);
    let q = distribution(&[("a", 0.2), ("c", 0.8)]);
    assert!((js_divergence(&p, &q) - js_divergence(&q, &p)).abs() < 1e-9);
}

#[test]
fn term_distribution_is_normalized() {
    let texts = ["cache usa lru cache", "vetor de cache"];
    let distribution = term_distribution(texts.iter().copied());
    let total: f64 = distribution.values().sum();
    assert!((total - 1.0).abs() < 1e-9, "soma = {total}");
}

#[test]
fn topic_drift_needs_two_notes() -> Result<()> {
    let only = note_created(NoteType::Fact, "cache", "", 1)?;
    assert_eq!(topic_drift(&[only])?, None);
    Ok(())
}

#[test]
fn topic_drift_grows_when_the_vocabulary_changes() -> Result<()> {
    let old = vec![
        note_created(NoteType::Fact, "cache lru", "cache lru", 1_000)?,
        note_created(
            NoteType::Fact,
            "cache lru expira",
            "cache lru expira",
            2_000,
        )?,
    ];
    let new = vec![
        note_created(NoteType::Fact, "vetor hnsw", "vetor hnsw", 3_000)?,
        note_created(
            NoteType::Fact,
            "vetor hnsw recall",
            "vetor hnsw recall",
            4_000,
        )?,
    ];
    let mut notes = old;
    notes.extend(new);
    let drift = topic_drift(&notes)?.unwrap_or(0.0);
    assert!(drift > 0.5, "drift devia ser alto: {drift}");
    Ok(())
}
