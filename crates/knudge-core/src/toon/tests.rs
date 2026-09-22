//! Testes do parser/emissor TOON (E02-T02) e do contrato de frontmatter (D74/D75).

use super::{detect_version, emit, parse, split_frontmatter};
use crate::schema::Value;

fn value_of(src: &str, key: &str) -> Value {
    parse(src)
        .ok()
        .and_then(|value| value.as_map().and_then(|map| map.get(key).cloned()))
        .unwrap_or(Value::Bool(false))
}

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
fn round_trip_is_byte_exact() {
    let parsed = parse(CANONICAL);
    assert!(parsed.is_ok(), "parse falhou: {parsed:?}");
    assert_eq!(emit(&parsed.unwrap_or(Value::Bool(false))), CANONICAL);
}

#[test]
fn empty_input_is_empty_map() {
    let parsed = parse("");
    assert_eq!(parsed.ok(), Some(Value::map(Vec::new())));
    assert_eq!(emit(&Value::map(Vec::new())), "");
}

#[test]
fn scalars_are_typed() {
    assert_eq!(value_of("n: 42\n", "n"), Value::Int(42));
    assert_eq!(value_of("n: -7\n", "n"), Value::Int(-7));
    assert_eq!(value_of("f: 0.5\n", "f"), Value::Float(0.5));
    assert_eq!(value_of("b: true\n", "b"), Value::Bool(true));
    assert_eq!(value_of("s: texto\n", "s"), Value::Str("texto".to_string()));
}

#[test]
fn inline_collections() {
    assert_eq!(
        value_of("tags: [a, b, c]\n", "tags"),
        Value::List(vec![
            Value::Str("a".to_string()),
            Value::Str("b".to_string()),
            Value::Str("c".to_string()),
        ])
    );
    assert_eq!(value_of("empty: []\n", "empty"), Value::List(Vec::new()));
    let map = value_of("m: {a: 1, b: 2}\n", "m");
    assert_eq!(map.as_map().map(indexmap::IndexMap::len), Some(2));
}

#[test]
fn integers_never_emit_dot_zero() {
    // `1.0` normaliza para `1` na emissão (D09).
    assert_eq!(
        emit(&parse("x: 1.0\n").unwrap_or(Value::Bool(false))),
        "x: 1\n"
    );
    // Um `Value::Float(1.0)` também sai como `1`.
    assert_eq!(
        emit(&Value::map([("x".to_string(), Value::Float(1.0))])),
        "x: 1\n"
    );
}

#[test]
fn strings_are_quoted_only_when_needed() {
    // `:` em valor é seguro (o parser corta só no primeiro `:` da linha).
    assert_eq!(
        emit(&parse("x: \"a: b\"\n").unwrap_or(Value::Bool(false))),
        "x: a: b\n"
    );
    assert_eq!(
        emit(&parse("x: \"linha\\nquebrada\"\n").unwrap_or(Value::Bool(false))),
        "x: \"linha\\nquebrada\"\n"
    );
    // `true` textual precisa de aspas para não virar booleano.
    assert_eq!(
        emit(&Value::map([(
            "x".to_string(),
            Value::Str("true".to_string())
        )])),
        "x: \"true\"\n"
    );
}

#[test]
fn unicode_whitespace_is_quoted() {
    // U+2000 nas pontas exige aspas: o parser usa `trim` Unicode.
    let value = Value::map([("x".to_string(), Value::Str("\u{2000}".to_string()))]);
    let text = emit(&value);
    assert_eq!(text, "x: \"\u{2000}\"\n");
    assert_eq!(parse(&text).ok(), Some(value));
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let src = "# comentário\na: 1   # trailing\n\nb: 2\n";
    let parsed = parse(src).unwrap_or(Value::Bool(false));
    assert_eq!(emit(&parsed), "a: 1\nb: 2\n");
}

#[test]
fn duplicate_keys_are_rejected() {
    assert!(parse("a: 1\na: 2\n").is_err());
}

#[test]
fn odd_indentation_is_rejected() {
    assert!(parse("a:\n   b: 1\n").is_err());
}

#[test]
fn split_frontmatter_handles_both_forms() {
    let (frontmatter, body) = split_frontmatter("---\na: 1\n---\ncorpo\n").unwrap_or_default();
    assert_eq!(frontmatter, "a: 1\n");
    assert_eq!(body, "corpo\n");

    let (frontmatter, body) = split_frontmatter("só corpo").unwrap_or_default();
    assert!(frontmatter.is_empty());
    assert_eq!(body, "só corpo");
}

#[test]
fn split_frontmatter_requires_closing_fence() {
    assert!(split_frontmatter("---\na: 1\n").is_err());
}

#[test]
fn detect_version_reads_schema_version() {
    assert_eq!(detect_version("schema_version: 1\n"), Some(1));
    assert_eq!(detect_version("a: 1\n"), None);
}

use proptest::prelude::*;

fn arb_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}".prop_map(String::from)
}

fn arb_scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<String>().prop_map(Value::Str),
        any::<i64>().prop_map(Value::Int),
        any::<bool>().prop_map(Value::Bool),
        (-1.0e6f64..1.0e6)
            .prop_filter("não inteiro", |value| value.fract() != 0.0)
            .prop_map(Value::Float),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn emit_parse_round_trips_maps(pairs in prop::collection::vec((arb_key(), arb_scalar()), 0..8)) {
        let mut map = indexmap::IndexMap::new();
        for (key, value) in pairs {
            map.insert(key, value);
        }
        let value = Value::Map(map);
        let text = emit(&value);
        let back = parse(&text);
        prop_assert!(back.is_ok(), "parse falhou: {back:?} / texto: {text:?}");
        prop_assert_eq!(back.ok(), Some(value));
    }

    #[test]
    fn emit_parse_round_trips_lists(items in prop::collection::vec(arb_scalar(), 1..8)) {
        let value = Value::map([("k".to_string(), Value::List(items))]);
        let text = emit(&value);
        let back = parse(&text);
        prop_assert!(back.is_ok(), "parse falhou: {back:?} / texto: {text:?}");
        prop_assert_eq!(back.ok(), Some(value));
    }
}
