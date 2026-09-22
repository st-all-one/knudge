//! Testes do dispatcher MCP (E14-T02/T03).

use serde_json::{Value, json};

use crate::jsonrpc::{self, Id, Request};
use crate::protocol;
use crate::server::McpServer;

fn request(method: &str, params: Value) -> Request {
    Request {
        id: Some(Id::Number(1)),
        method: method.to_string(),
        params,
    }
}

fn notification(method: &str) -> Request {
    Request {
        id: None,
        method: method.to_string(),
        params: Value::Null,
    }
}

fn handle(server: &mut McpServer, method: &str, params: Value) -> Value {
    server
        .handle(&request(method, params))
        .unwrap_or(Value::Null)
}

#[test]
fn initialize_negotiates_supported_version() {
    let mut server = McpServer::new(3, 0);
    let response = handle(
        &mut server,
        "initialize",
        json!({ "protocolVersion": "2024-11-05" }),
    );
    assert_eq!(
        response
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str),
        Some("2024-11-05")
    );
    assert_eq!(server.protocol_version(), "2024-11-05");
    assert_eq!(
        response
            .pointer("/result/serverInfo/name")
            .and_then(Value::as_str),
        Some(protocol::SERVER_NAME)
    );
}

#[test]
fn initialize_unknown_version_falls_back() {
    let mut server = McpServer::new(3, 0);
    let response = handle(
        &mut server,
        "initialize",
        json!({ "protocolVersion": "1999-01-01" }),
    );
    assert_eq!(
        response
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str),
        Some(protocol::PROTOCOL_VERSION)
    );
}

#[test]
fn initialized_notification_sets_flag_without_response() {
    let mut server = McpServer::new(3, 0);
    assert!(
        server
            .handle(&notification("notifications/initialized"))
            .is_none()
    );
    assert!(server.is_initialized());
}

#[test]
fn ping_returns_empty_object() {
    let mut server = McpServer::new(3, 0);
    let response = handle(&mut server, "ping", Value::Null);
    assert_eq!(response.get("result"), Some(&json!({})));
}

#[test]
fn tools_list_has_four_tools() {
    let mut server = McpServer::new(3, 0);
    let response = handle(&mut server, "tools/list", Value::Null);
    let count = response
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .map(Vec::len);
    assert_eq!(count, Some(4));
}

#[test]
fn tools_call_pre_write_returns_pointers() {
    let mut server = McpServer::new(3, 0);
    let params = json!({
        "name": "knudge_pre_write",
        "arguments": { "candidates": [ { "id": "fact_1", "score": 0.9 } ] }
    });
    let response = handle(&mut server, "tools/call", params);
    assert_eq!(
        response.pointer("/result/isError").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response
            .pointer("/result/structuredContent/hints/0/ids/0")
            .and_then(Value::as_str),
        Some("fact_1")
    );
}

#[test]
fn tools_call_invalid_arguments_is_error_result() {
    let mut server = McpServer::new(3, 0);
    let params = json!({ "name": "knudge_pre_write", "arguments": {} });
    let response = handle(&mut server, "tools/call", params);
    assert_eq!(
        response.pointer("/result/isError").and_then(Value::as_bool),
        Some(true)
    );
    assert!(response.get("error").is_none());
}

#[test]
fn missing_tool_name_is_invalid_params() {
    let mut server = McpServer::new(3, 0);
    let response = handle(&mut server, "tools/call", json!({ "arguments": {} }));
    assert_eq!(
        response.pointer("/error/code").and_then(Value::as_i64),
        Some(jsonrpc::INVALID_PARAMS)
    );
}

#[test]
fn unknown_method_is_not_found() {
    let mut server = McpServer::new(3, 0);
    let response = handle(&mut server, "nope", Value::Null);
    assert_eq!(
        response.pointer("/error/code").and_then(Value::as_i64),
        Some(jsonrpc::METHOD_NOT_FOUND)
    );
}
