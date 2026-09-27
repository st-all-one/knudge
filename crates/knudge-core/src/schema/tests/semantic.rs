//! Testes das chaves semânticas: arestas de ontologia, claims e proveniência (E19-T09/D207).

use crate::Result;
use crate::schema::{
    CANONICAL_KEYS, Claim, Classification, EDGE_KEYS, EdgeKind, NoteType, Provenance,
    SCHEMA_VERSION, Status, Value, claims, provenance,
};

use super::valid_frontmatter;

#[test]
fn ontology_edge_kinds_match_canonical_keys() {
    assert_eq!(EdgeKind::ALL.len(), EDGE_KEYS.len());
    for kind in EdgeKind::ALL {
        assert_eq!(kind.key(), kind.as_str());
        assert!(EDGE_KEYS.contains(&kind.as_str()));
    }
    // A janela das arestas em `CANONICAL_KEYS` é contígua e na ordem de `ALL`.
    let start = CANONICAL_KEYS
        .iter()
        .position(|key| *key == EdgeKind::References.key());
    let window =
        start.and_then(|index| CANONICAL_KEYS.get(index..index.saturating_add(EDGE_KEYS.len())));
    assert_eq!(window, Some(EDGE_KEYS.as_slice()));
    assert!(EdgeKind::SameAs.is_symmetric());
    assert_eq!(EdgeKind::Broader.inverse(), EdgeKind::Narrower);
    assert_eq!(EdgeKind::Narrower.inverse(), EdgeKind::Broader);
    assert!(EdgeKind::Related.is_ontology());
    assert!(!EdgeKind::References.is_ontology());
}

#[test]
fn claims_round_trip_through_frontmatter() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    let claim = Claim::new("kd", "usa", "TOON");
    fm.set("claims", Value::List(vec![claim.to_value()]))?;
    fm.validate()?;
    assert_eq!(claims(&fm)?, vec![claim]);
    Ok(())
}

#[test]
fn claim_rejects_empty_or_overlong_fields() {
    assert!(Claim::new("kd", "", "TOON").validate().is_err());
    assert!(Claim::new("kd", "usa", "x".repeat(121)).validate().is_err());
    assert!(Claim::new(" kd ", " usa ", " TOON ").validate().is_ok());
}

#[test]
fn malformed_claim_is_a_schema_error() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    fm.set("claims", Value::List(vec![Value::Str("solto".to_string())]))?;
    assert!(claims(&fm).is_err());
    Ok(())
}

#[test]
fn provenance_round_trips_and_is_empty_by_default() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    assert!(provenance(&fm)?.is_empty());
    fm.set(
        "provenance",
        Provenance {
            entity: None,
            activity: Some("write".to_string()),
            agent: Some("agente".to_string()),
        }
        .to_value(),
    )?;
    fm.validate()?;
    let read = provenance(&fm)?;
    assert_eq!(read.activity.as_deref(), Some("write"));
    assert_eq!(read.agent.as_deref(), Some("agente"));
    assert_eq!(read.entity, None);
    Ok(())
}

#[test]
fn frontmatter_optional_defaults() -> Result<()> {
    let fm = valid_frontmatter()?;
    assert_eq!(fm.classification()?, Classification::Tactical);
    assert_eq!(fm.status()?, Status::Active);
    assert_eq!(fm.scope()?, None);
    assert_eq!(fm.note_type()?, NoteType::Fact);
    assert_eq!(fm.schema_version()?, SCHEMA_VERSION);
    Ok(())
}
