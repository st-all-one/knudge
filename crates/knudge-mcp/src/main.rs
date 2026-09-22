//! Entrada do servidor MCP `knudge-mcp` (E14-T05, D68).

#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

use knudge_core::ErrorKind;
use knudge_mcp::config::McpConfig;
use knudge_mcp::server::{McpServer, SERVER_VERSION};
use knudge_mcp::transport;

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
