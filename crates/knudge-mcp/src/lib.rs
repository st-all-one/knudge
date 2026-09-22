//! # knudge-mcp
//!
//! Servidor MCP do knudge (E12-T03). O **motor de gatilhos** é puro e testável; o transporte
//! JSON-RPC fica na borda (E13).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod triggers;

pub use triggers::{DEFAULT_HINTS_CAP, Hint, HintEngine, HintKind, Trigger};

/// Nome do servidor anunciado ao cliente MCP.
pub const SERVER_NAME: &str = "knudge";

#[cfg(test)]
mod tests;
