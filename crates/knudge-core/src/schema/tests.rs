//! Testes do schema canônico (E02-T01/T03/T05/T06).

use super::body::{body_hash, normalize};
use super::frontmatter::Frontmatter;
use super::hash::{base36_8, hex8};
use super::id::{is_valid_note_id, note_id};
use super::text::{count_scalars, validate_statement};
use super::types::{Classification, NoteType, Scope, Status};
use super::{SCHEMA_VERSION, Value};
use crate::{Result, toon};
use proptest::prelude::*;
use std::str::FromStr;

#[test]
fn note_type_round_trips_and_prefixes() {
    for note_type in NoteType::ALL {
        assert_eq!(NoteType::from_str(note_type.as_str()).ok(), Some(note_type));
        assert_eq!(note_type.to_string(), note_type.as_str());
        assert!(!note_type.prefix().is_empty());
    }
}

#[test]
fn unknown_type_is_rejected() {
    assert!(NoteType::from_str("inventado").is_err());
    assert!(Scope::from_str("nivel").is_err());
    assert!(Classification::from_str("vago").is_err());
    assert!(Status::from_str("morto").is_err());
}

#[test]
fn scope_hierarchy_is_closed() {
    assert_eq!(Scope::Plan.depth(), 1);
    assert_eq!(Scope::Task.depth(), 4);
    assert_eq!(Scope::Task.parent(), Some(Scope::Issue));
    assert_eq!(Scope::Plan.parent(), None);
}

#[test]
fn normalize_collapses_and_nfc() {
    assert_eq!(normalize("  a\t b\n c  "), "a b c");
    // "e" + combining acute (U+0301) vira "é" pré-composto em NFC.
    assert_eq!(normalize("e\u{0301}"), "\u{e9}");
    assert_eq!(normalize(""), "");
}

#[test]
fn body_hash_depends_on_statement_and_body() {
    let base = body_hash("afirmação", "corpo");
    assert_eq!(base, body_hash("afirmação", "corpo"));
    assert_ne!(base, body_hash("outra afirmação", "corpo"));
    assert_ne!(base, body_hash("afirmação", "outro corpo"));
    // Variação tipográfica não muda o hash (normaliza antes).
    assert_eq!(base, body_hash("  afirmação ", "corpo\n"));
    assert_eq!(base.len(), 8);
}

#[test]
fn ids_are_idempotent_and_prefixed() {
    let first = note_id(NoteType::Fact, "Rust > JSON");
    let second = note_id(NoteType::Fact, "Rust > JSON");
    assert_eq!(first, second, "mesmo conteúdo ⇒ mesmo id");
    assert!(first.starts_with("fact_"));
    assert!(is_valid_note_id(&first));
    assert_ne!(
        note_id(NoteType::Fact, "x"),
        note_id(NoteType::Decision, "x")
    );
}

#[test]
fn id_format_is_validated() {
    assert!(!is_valid_note_id("fact_"));
    assert!(!is_valid_note_id("fact_123"));
    assert!(!is_valid_note_id("fact_1234567G"));
    assert!(!is_valid_note_id("nope_12345678"));
    assert!(is_valid_note_id("decision_00000000"));
}

#[test]
fn base36_is_zero_padded() {
    assert_eq!(base36_8(0), "00000000");
    assert_eq!(base36_8(35), "0000000z");
    assert_eq!(base36_8(36), "00000010");
}

#[test]
fn hex8_is_eight_lowercase_hex() {
    let value = hex8(b"knudge");
    assert_eq!(value.len(), 8);
    assert!(value.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn statement_limit_counts_scalars() {
    // 120 emoji (4 bytes cada) ainda cabe; 121 não.
    let emoji: String = "\u{1f600}".repeat(120);
    assert_eq!(count_scalars(&emoji), 120);
    assert!(validate_statement(&emoji).is_ok());
    assert!(validate_statement(&"\u{1f600}".repeat(121)).is_err());
    // CJK e combining contam como 1 escalar cada.
    assert_eq!(count_scalars("日本語"), 3);
    assert_eq!(count_scalars("e\u{0301}"), 2);
}

#[test]
fn schema_version_is_frozen() {
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn value_accessors() {
    assert_eq!(Value::Int(3).as_f64(), Some(3.0));
    assert_eq!(Value::Float(0.5).as_f64(), Some(0.5));
    assert_eq!(Value::Str("x".to_string()).as_str(), Some("x"));
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert!(Value::Int(1).as_str().is_none());
}

#[test]
fn value_map_preserves_order() {
    let value = Value::map([
        ("a".to_string(), Value::Int(1)),
        ("b".to_string(), Value::Int(2)),
    ]);
    let keys: Vec<&str> = value
        .as_map()
        .map(|map| map.keys().map(String::as_str).collect())
        .unwrap_or_default();
    assert_eq!(keys, ["a", "b"]);
}

fn valid_frontmatter() -> Result<Frontmatter> {
    let mut fm = Frontmatter::new();
    fm.set("id", Value::Str(note_id(NoteType::Fact, "afirmação")))?;
    fm.set("type", Value::Str("fact".to_string()))?;
    fm.set("statement", Value::Str("afirmação".to_string()))?;
    fm.set(
        "created_at",
        Value::Str("2026-01-02T03:04:05.678Z".to_string()),
    )?;
    fm.set("confidence", Value::Float(0.7))?;
    fm.set("body_hash", Value::Str(body_hash("afirmação", "corpo")))?;
    fm.set("schema_version", Value::Int(1))?;
    Ok(fm)
}

#[test]
fn frontmatter_orders_canonically() -> Result<()> {
    let mut fm = Frontmatter::new();
    fm.set("schema_version", Value::Int(1))?;
    fm.set("id", Value::Str("fact_00000000".to_string()))?;
    let keys: Vec<String> = fm
        .to_value()
        .as_map()
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    assert_eq!(keys, ["id", "schema_version"]);
    Ok(())
}

#[test]
fn frontmatter_rejects_unknown_key_on_write() {
    let mut fm = Frontmatter::new();
    assert!(fm.set("author", Value::Str("eu".to_string())).is_err());
}

#[test]
fn frontmatter_reads_with_warning_on_unknown_key() -> Result<()> {
    let value = toon::parse("id: x\nlegacy: y\n")?;
    let (fm, warnings) = Frontmatter::from_value(&value)?;
    assert!(fm.get("legacy").is_none());
    assert_eq!(warnings.len(), 1);
    Ok(())
}

#[test]
fn frontmatter_round_trips_via_toon() -> Result<()> {
    let fm = valid_frontmatter()?;
    let text = fm.to_string();
    let (back, warnings) = Frontmatter::parse(&text)?;
    assert!(warnings.is_empty(), "sem warnings: {warnings:?}");
    assert_eq!(back.to_value(), fm.to_value());
    Ok(())
}

#[test]
fn frontmatter_validate_checks_required_and_ranges() -> Result<()> {
    valid_frontmatter()?.validate()?;

    let mut missing = valid_frontmatter()?;
    missing.remove("type");
    assert!(missing.validate().is_err());

    let mut bad_confidence = valid_frontmatter()?;
    bad_confidence.set("confidence", Value::Float(1.5))?;
    assert!(bad_confidence.validate().is_err());

    let mut long = valid_frontmatter()?;
    long.set("statement", Value::Str("x".repeat(121)))?;
    assert!(long.validate().is_err());
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn normalize_is_idempotent(input in "\\PC*") {
        let once = normalize(&input);
        prop_assert_eq!(normalize(&once), once);
    }

    #[test]
    fn note_id_is_deterministic(statement in "\\PC*") {
        prop_assert_eq!(
            note_id(NoteType::Fact, &statement),
            note_id(NoteType::Fact, &statement)
        );
    }
}
