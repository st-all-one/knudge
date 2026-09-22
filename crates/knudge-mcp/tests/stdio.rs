//! Integração do binário `knudge-mcp` via stdio (E14-T06).

use std::io::{self, Write};
use std::process::{Command, Stdio};

use serde_json::Value;

fn missing() -> io::Error {
    io::Error::other("resposta ausente")
}

#[test]
fn handshake_and_tools_over_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_knudge-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\"}}\n",
        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"knudge_pre_write\",\"arguments\":{\"candidates\":[{\"id\":\"fact_1\",\"score\":0.9}]}}}\n",
    );
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
        stdin.flush()?;
    }
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "respostas inesperadas: {stdout}");

    let init: Value = serde_json::from_str(lines.first().copied().ok_or_else(missing)?)?;
    assert_eq!(
        init.pointer("/result/protocolVersion")
            .and_then(Value::as_str),
        Some("2025-06-18")
    );

    let list: Value = serde_json::from_str(lines.get(1).copied().ok_or_else(missing)?)?;
    assert_eq!(
        list.pointer("/result/tools")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(4)
    );

    let call: Value = serde_json::from_str(lines.get(2).copied().ok_or_else(missing)?)?;
    assert_eq!(
        call.pointer("/result/structuredContent/hints/0/ids/0")
            .and_then(Value::as_str),
        Some("fact_1")
    );
    Ok(())
}

#[test]
fn help_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_knudge-mcp"))
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("knudge-mcp"), "ajuda inesperada: {stdout}");
    Ok(())
}
