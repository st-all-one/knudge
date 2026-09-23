//! # knudge-mcp
//!
//! Servidor MCP do knudge (D68). O **motor de gatilhos** (`triggers`) é puro; o **codec**
//! (`jsonrpc`) e o **dispatcher** (`server`) não fazem I/O; o **transporte** (`transport`)
//! fala JSON-RPC 2.0 sobre stdio, **uma linha por mensagem**.
//!
//! Este pacote é um **binário** (sem `[lib]`): o `knudge-core` é a biblioteca interna,
//! compartilhada com o binário `kd`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod jsonrpc;
pub mod protocol;
pub mod server;
pub mod tools;
pub mod transport;
pub mod triggers;

#[cfg(test)]
mod tests;

use std::io::{self, Write};
use std::process::ExitCode;

use knudge_core::ErrorKind;

use crate::config::McpConfig;
use crate::server::{McpServer, SERVER_VERSION};

/// Ajuda do binário.
const USAGE: &str = "knudge-mcp — servidor MCP do knudge (JSON-RPC 2.0 sobre stdio)

Uso:
  knudge-mcp [--stdio]

Opções:
  --stdio        Transporte stdio (padrão)
  -h, --help     Mostra esta ajuda
  -V, --version  Mostra a versão
";

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    if args
        .iter()
        .any(|arg| arg.to_str() == Some("-h") || arg.to_str() == Some("--help"))
    {
        return write_stdout(USAGE.as_bytes());
    }
    if args
        .iter()
        .any(|arg| arg.to_str() == Some("-V") || arg.to_str() == Some("--version"))
    {
        return write_stdout(format!("knudge-mcp {SERVER_VERSION}\n").as_bytes());
    }

    let config = McpConfig::load_from_cwd();
    let mut server = McpServer::new(config.hints_cap, config.observation_sessions);
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let _served = transport::serve(&mut server, stdin.lock(), &mut stdout);
    ExitCode::SUCCESS
}

/// Escreve em stdout tratando `EPIPE` como sucesso (D73).
fn write_stdout(bytes: &[u8]) -> ExitCode {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(ErrorKind::Io.exit_code()),
    }
}
