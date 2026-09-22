//! Testes do codec JSON-RPC (E14-T01).

use crate::jsonrpc::{self, Id, RpcError};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn parses_request_with_number_id() -> TestResult {
    let request = jsonrpc::parse(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#)?;
    assert_eq!(request.id, Some(Id::Number(7)));
    assert_eq!(request.method, "ping");
    assert!(!request.is_notification());
    Ok(())
}

#[test]
fn parses_request_with_text_id() -> TestResult {
    let request = jsonrpc::parse(r#"{"jsonrpc":"2.0","id":"abc","method":"tools/list"}"#)?;
    assert_eq!(request.id, Some(Id::Text("abc".to_string())));
    Ok(())
}

#[test]
fn missing_id_is_notification() -> TestResult {
    let request = jsonrpc::parse(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)?;
    assert!(request.is_notification());
    assert_eq!(request.params, serde_json::Value::Null);
    Ok(())
}

#[test]
fn null_id_is_notification() -> TestResult {
    let request = jsonrpc::parse(r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#)?;
    assert!(request.is_notification());
    Ok(())
}

#[test]
fn rejects_bad_json() {
    let error = jsonrpc::parse("{").err();
    assert_eq!(
        error.as_ref().map(|error| error.code),
        Some(jsonrpc::PARSE_ERROR)
    );
}

#[test]
fn rejects_wrong_version() {
    let error = jsonrpc::parse(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#).err();
    assert_eq!(
        error.as_ref().map(|error| error.code),
        Some(jsonrpc::INVALID_REQUEST)
    );
}

#[test]
fn rejects_missing_method() {
    let error = jsonrpc::parse(r#"{"jsonrpc":"2.0","id":1}"#).err();
    assert_eq!(
        error.as_ref().map(|error| error.code),
        Some(jsonrpc::INVALID_REQUEST)
    );
}

#[test]
fn rejects_non_object() {
    let error = jsonrpc::parse("[1,2,3]").err();
    assert_eq!(
        error.as_ref().map(|error| error.code),
        Some(jsonrpc::INVALID_REQUEST)
    );
}

#[test]
fn rejects_boolean_id() {
    let error = jsonrpc::parse(r#"{"jsonrpc":"2.0","id":true,"method":"ping"}"#).err();
    assert_eq!(
        error.as_ref().map(|error| error.code),
        Some(jsonrpc::INVALID_REQUEST)
    );
}

#[test]
fn emits_result_with_id() {
    let value = jsonrpc::result(&Id::Number(1), &serde_json::json!({ "ok": true }));
    assert_eq!(
        value.get("jsonrpc").and_then(serde_json::Value::as_str),
        Some("2.0")
    );
    assert_eq!(value.get("id").and_then(serde_json::Value::as_i64), Some(1));
    assert_eq!(
        value
            .pointer("/result/ok")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[test]
fn emits_error_with_null_id() {
    let value = jsonrpc::error(None, &RpcError::new(jsonrpc::PARSE_ERROR, "ruim"));
    assert_eq!(value.get("id"), Some(&serde_json::Value::Null));
    assert_eq!(
        value
            .pointer("/error/code")
            .and_then(serde_json::Value::as_i64),
        Some(jsonrpc::PARSE_ERROR)
    );
}

#[test]
fn error_data_is_included() {
    let error = RpcError::new(jsonrpc::INVALID_PARAMS, "faltou").with_data(serde_json::json!(1));
    let value = error.to_value();
    assert_eq!(
        value.pointer("/data").and_then(serde_json::Value::as_i64),
        Some(1)
    );
}
