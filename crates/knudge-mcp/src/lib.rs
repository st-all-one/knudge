//! # knudge-mcp
//!
//! Servidor MCP do knudge (D68). O **motor de gatilhos** ([`triggers`]) é puro; o **codec**
//! ([`jsonrpc`]) e o **dispatcher** ([`server`]) não fazem I/O; o **transporte** ([`transport`])
//! fala JSON-RPC 2.0 sobre stdio, **uma linha por mensagem**.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod jsonrpc;
pub mod protocol;
pub mod server;
pub mod tools;
pub mod transport;
pub mod triggers;

pub use protocol::SERVER_NAME;
pub use server::McpServer;
pub use triggers::{DEFAULT_HINTS_CAP, Hint, HintEngine, HintKind, Trigger};

#[cfg(test)]
mod tests;
