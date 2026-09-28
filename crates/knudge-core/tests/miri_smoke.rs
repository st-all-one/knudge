//! Smoke de UB sob Miri (E13-T08): parser/emissor TOON, codec JSONL e hashes — dados pequenos.
//!
//! É a **única** suíte que o `make miri` roda: proptest de 256 casos e corpora grandes ficam de
//! fora (lentos sob interpretação); a cobertura funcional fica no `make check`. O core
//! é `#![forbid(unsafe_code)]`, então aqui o Miri vale como sanidade de alocação/bounds/UTF-8.

use knudge_core::Result;
use knudge_core::jsonl::{decode, encode, lines};
use knudge_core::schema::body::{body_hash, normalize};
use knudge_core::schema::hash::{base36_8, hex8};
use knudge_core::schema::id::note_id;
use knudge_core::schema::{NoteType, Value};
use knudge_core::toon::{emit, parse, split_frontmatter};

/// Frontmatter canônico (byte-exato) — cobre escalares, lista inline, mapa e lista de mapas.
const CANONICAL: &str = "\
id: fact_7a3c1b2d
type: fact
statement: Rust > JSON por decodificador formal
created_at: 2026-01-02T03:04:05.678Z
confidence: 0.7
body_hash: 1a2b3c4d
schema_version: 1
tags: [busca, sem-vector-db]
evidence:
  build: true
outcomes:
  - status: success
    duration: 120
";

#[test]
fn toon_round_trip_is_byte_exact() {
    let parsed = parse(CANONICAL);
    assert!(parsed.is_ok(), "parse falhou: {parsed:?}");
    assert_eq!(emit(&parsed.unwrap_or(Value::Bool(false))), CANONICAL);
}

#[test]
fn toon_frontmatter_tolerates_crlf_and_unicode() {
    let split = split_frontmatter("---\r\na: 1\r\n---\r\ncorpo café\r\n");
    assert!(split.is_ok(), "split falhou: {split:?}");
    let (frontmatter, body) = split.unwrap_or_default();
    assert!(frontmatter.contains("a: 1"));
    assert!(body.contains("café"));
}

#[test]
fn jsonl_round_trips_and_skips_blank_lines() -> Result<()> {
    let value = Value::map([
        ("a".to_string(), Value::Int(1)),
        ("b".to_string(), Value::List(vec![Value::Bool(true)])),
    ]);
    let text = encode(&value)?;
    assert_eq!(decode(&text)?, value);
    assert_eq!(lines("1\n\n2\r\n").count(), 2);
    Ok(())
}

#[test]
fn ids_and_hashes_are_stable() {
    assert_eq!(base36_8(0).len(), 8);
    assert_eq!(hex8(b"x").len(), 8);
    assert_eq!(body_hash("s", "b").len(), 8);
    let id = note_id(NoteType::Fact, "café com leite");
    assert!(id.starts_with("fact_"), "id inesperado: {id}");
}

#[test]
fn normalize_is_idempotent() {
    let once = normalize("  a\u{a0}  b  ");
    assert_eq!(normalize(&once), once);
}
