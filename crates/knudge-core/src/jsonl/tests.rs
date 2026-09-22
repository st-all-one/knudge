//! Testes do codec JSON e do leitor JSONL.

use proptest::collection::{btree_map, vec};
use proptest::prelude::*;

use super::*;
use crate::schema::Value;

#[test]
fn maps_are_encoded_with_sorted_keys() -> crate::Result<()> {
    let value = Value::map([
        ("b".to_string(), Value::Int(2)),
        ("a".to_string(), Value::Int(1)),
    ]);
    assert_eq!(encode(&value)?, r#"{"a":1,"b":2}"#);
    Ok(())
}

#[test]
fn strings_escape_control_and_quotes() -> crate::Result<()> {
    let value = Value::Str("a\"b\\c\nd\t\u{1}".to_string());
    assert_eq!(encode(&value)?, r#""a\"b\\c\nd\t\u0001""#);
    Ok(())
}

#[test]
fn floats_keep_their_type() -> crate::Result<()> {
    assert_eq!(encode(&Value::Float(1.0))?, "1.0");
    assert_eq!(decode("1.0")?, Value::Float(1.0));
    assert_eq!(decode("1")?, Value::Int(1));
    assert_eq!(encode(&Value::Float(0.5))?, "0.5");
    Ok(())
}

#[test]
fn unicode_escapes_and_surrogates() -> crate::Result<()> {
    assert_eq!(decode(r#""\u00e9""#)?, Value::Str("é".to_string()));
    assert_eq!(decode(r#""\ud83d\ude00""#)?, Value::Str("😀".to_string()));
    Ok(())
}

#[test]
fn decode_rejects_null_and_trailing() {
    assert!(decode("null").is_err());
    assert!(decode("1 2").is_err());
    assert!(decode("{").is_err());
}

#[test]
fn lines_skip_blank_and_crlf() {
    let source = "{\"a\":1}\r\n\n{\"b\":2}\n";
    let items: Vec<&str> = lines(source).collect();
    assert_eq!(items, vec![r#"{"a":1}"#, r#"{"b":2}"#]);
}

fn round_trips(value: &Value) -> bool {
    let Ok(text) = encode(value) else {
        return false;
    };
    let Ok(back) = decode(&text) else {
        return false;
    };
    matches!(encode(&back), Ok(again) if again == text)
}

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        any::<String>().prop_map(Value::Str),
        any::<i64>().prop_map(Value::Int),
        any::<bool>().prop_map(Value::Bool),
        any::<f64>()
            .prop_filter("finito", |f| f.is_finite())
            .prop_map(Value::Float),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            vec(inner.clone(), 0..8).prop_map(Value::List),
            btree_map(any::<String>(), inner, 0..8)
                .prop_map(|map| Value::Map(map.into_iter().collect())),
        ]
    })
}

proptest! {
    #[test]
    fn encode_decode_is_canonical(value in arb_value()) {
        prop_assert!(round_trips(&value));
    }
}
