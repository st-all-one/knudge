//! Testes das chaves semânticas: arestas de ontologia, claims e proveniência (E19-T09/D207).

use crate::Result;
use crate::schema::{
    AgentRole, CANONICAL_KEYS, Claim, Classification, EDGE_KEYS, EdgeKind, NoteType, Provenance,
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
            ..Provenance::default()
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

#[test]
fn provenance_round_trips_agent_identity() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    let written = Provenance {
        entity: None,
        activity: Some("derive".to_string()),
        agent: Some("omp".to_string()),
        agent_id: Some("omp:01j".to_string()),
        agent_role: Some(AgentRole::Subagent),
        parent_agent_id: Some("omp:01h".to_string()),
    };
    fm.set("provenance", written.to_value())?;
    fm.validate()?;
    let read = provenance(&fm)?;
    assert_eq!(read, written);
    assert_eq!(read.agent_id_or_legacy(), "omp:01j");
    assert_eq!(read.agent_role_or_main(), AgentRole::Subagent);
    let value = read.to_value();
    assert_eq!(fm.get("provenance"), Some(&value));
    Ok(())
}

#[test]
fn old_note_defaults_identity_without_rewriting_bytes() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    fm.set("schema_version", Value::Int(2))?;
    fm.set(
        "provenance",
        Provenance {
            agent: Some("ktu".to_string()),
            ..Provenance::default()
        }
        .to_value(),
    )?;
    fm.validate()?;
    let read = provenance(&fm)?;
    assert_eq!(read.agent_id_or_legacy(), "ktu");
    assert_eq!(read.agent_role_or_main(), AgentRole::Main);
    assert_eq!(read.parent_agent_id, None);
    let value = read.to_value();
    let map = value
        .as_map()
        .ok_or_else(|| crate::Error::internal("provenance não é mapa"))?;
    assert!(!map.contains_key("agent_id"), "bytes v2 preservados");
    assert!(!map.contains_key("agent_role"), "bytes v2 preservados");
    Ok(())
}

#[test]
fn malformed_agent_identity_is_rejected() -> Result<()> {
    let mut fm = valid_frontmatter()?;
    for (key, value) in [
        ("agent_role", Value::Str("chefe".to_string())),
        ("agent_id", Value::Int(3)),
        ("parent_agent_id", Value::Str("com\ncontrolo".to_string())),
    ] {
        fm.set("provenance", Value::map([(key.to_string(), value)]))?;
        assert!(fm.validate().is_err(), "{key} deveria falhar");
    }
    Ok(())
}
