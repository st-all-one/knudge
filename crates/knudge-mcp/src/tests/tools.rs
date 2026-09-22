//! Testes das tools MCP (E14-T03).

use serde_json::{Value, json};

use crate::tools;
use crate::triggers::HintEngine;

fn len_of(value: &Value, pointer: &str) -> Option<usize> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(Vec::len)
}

#[test]
fn pre_write_defaults_statement_to_id() {
    let mut engine = HintEngine::new(3, 0);
    let result = tools::call(
        &mut engine,
        tools::PRE_WRITE,
        &json!({ "candidates": [ { "id": "fact_1", "score": 0.5 } ] }),
    );
    assert_eq!(
        result
            .pointer("/structuredContent/hints/0/kind")
            .and_then(Value::as_str),
        Some("duplicate")
    );
    assert_eq!(
        result
            .pointer("/structuredContent/hints/0/ids/0")
            .and_then(Value::as_str),
        Some("fact_1")
    );
}

#[test]
fn session_end_advances_session() {
    let mut engine = HintEngine::new(3, 0);
    let _ignored = tools::call(
        &mut engine,
        tools::SESSION_END,
        &json!({ "writes": 0, "proposals": [] }),
    );
    assert_eq!(engine.sessions_seen(), 1);
}

#[test]
fn session_end_with_writes_returns_no_hints() {
    let mut engine = HintEngine::new(3, 0);
    let result = tools::call(
        &mut engine,
        tools::SESSION_END,
        &json!({ "writes": 2, "proposals": [ { "kind": "link", "ids": ["a"] } ] }),
    );
    assert_eq!(len_of(&result, "/structuredContent/hints"), Some(0));
}

#[test]
fn session_end_maps_learn_kind() {
    let mut engine = HintEngine::new(3, 0);
    let result = tools::call(
        &mut engine,
        tools::SESSION_END,
        &json!({ "writes": 0, "proposals": [ { "kind": "merge", "ids": ["a", "b"], "score": 0.7 } ] }),
    );
    assert_eq!(
        result
            .pointer("/structuredContent/hints/0/kind")
            .and_then(Value::as_str),
        Some("merge")
    );
}

#[test]
fn unknown_tool_is_error() {
    let mut engine = HintEngine::new(3, 0);
    let result = tools::call(&mut engine, "nope", &json!({}));
    assert_eq!(result.get("isError").and_then(Value::as_bool), Some(true));
}

#[test]
fn status_reports_observation() {
    let mut engine = HintEngine::new(2, 5);
    let result = tools::call(&mut engine, tools::STATUS, &json!({}));
    assert_eq!(
        result
            .pointer("/structuredContent/observing")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        result
            .pointer("/structuredContent/cap")
            .and_then(Value::as_u64),
        Some(2)
    );
}
