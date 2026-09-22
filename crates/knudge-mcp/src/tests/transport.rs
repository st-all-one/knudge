//! Testes do transporte stdio (E14-T04).

use std::io::{Cursor, Write};

use serde_json::Value;

use crate::jsonrpc;
use crate::server::McpServer;
use crate::transport;

fn server() -> McpServer {
    McpServer::new(3, 0)
}

#[test]
fn serves_initialize_notification_and_tools_list() {
    let mut server = server();
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\"}}\n",
        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n",
    );
    let mut output = Vec::new();
    let ok = transport::serve(&mut server, Cursor::new(input.as_bytes()), &mut output);
    assert!(ok);
    let text = String::from_utf8(output).unwrap_or_default();
    assert_eq!(text.lines().count(), 2);
    assert!(server.is_initialized());
}

#[test]
fn parse_error_gets_null_id() {
    let mut server = server();
    let mut output = Vec::new();
    let _ignored = transport::serve(&mut server, Cursor::new(b"{".to_vec()), &mut output);
    let text = String::from_utf8(output).unwrap_or_default();
    let value: Value = serde_json::from_str(text.trim()).unwrap_or(Value::Null);
    assert_eq!(value.get("id"), Some(&Value::Null));
    assert_eq!(
        value.pointer("/error/code").and_then(Value::as_i64),
        Some(jsonrpc::PARSE_ERROR)
    );
}

#[test]
fn blank_lines_are_ignored() {
    let mut server = server();
    let mut output = Vec::new();
    let input = "\n\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n";
    let ok = transport::serve(&mut server, Cursor::new(input.as_bytes()), &mut output);
    assert!(ok);
    let text = String::from_utf8(output).unwrap_or_default();
    assert_eq!(text.lines().count(), 1);
}

#[test]
fn eof_without_messages_is_success() {
    let mut server = server();
    let mut output = Vec::new();
    let ok = transport::serve(&mut server, Cursor::new(Vec::new()), &mut output);
    assert!(ok);
    assert!(output.is_empty());
}

#[test]
fn broken_pipe_terminates_with_success() {
    let mut server = server();
    let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
    let ok = transport::serve(
        &mut server,
        Cursor::new(input.as_bytes()),
        &mut BrokenWriter,
    );
    assert!(ok);
}

struct BrokenWriter;

impl Write for BrokenWriter {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "fechado",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
