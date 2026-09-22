//! Loop stdio do MCP (E14-T04, D71/D73).
//!
//! Framing: **uma linha JSON por mensagem** (sem `Content-Length`). EOF e `EPIPE` encerram com
//! sucesso; nenhum log vai para stdout.

use std::io::{BufRead, Write};

use serde_json::Value;

use crate::jsonrpc;
use crate::server::McpServer;

/// Serve `input` até EOF, escrevendo respostas em `output`.
///
/// Devolve `true` quando terminou por EOF/`EPIPE` (exit 0) e `false` em erro irrecuperável de
/// leitura.
pub fn serve<R: BufRead, W: Write>(server: &mut McpServer, input: R, output: &mut W) -> bool {
    for line in input.lines() {
        let Ok(line) = line else {
            return false;
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(value) = respond(server, trimmed)
            && write_message(output, &value).is_err()
        {
            return true;
        }
    }
    true
}

/// Decodifica e despacha uma linha; erros de parse viram resposta com `id: null`.
#[must_use]
pub fn respond(server: &mut McpServer, line: &str) -> Option<Value> {
    match jsonrpc::parse(line) {
        Ok(request) => server.handle(&request),
        Err(error) => Some(jsonrpc::error(None, &error)),
    }
}

fn write_message<W: Write>(output: &mut W, value: &Value) -> std::io::Result<()> {
    let text = serde_json::to_string(value).map_err(std::io::Error::other)?;
    output.write_all(text.as_bytes())?;
    output.write_all(b"\n")?;
    output.flush()
}
