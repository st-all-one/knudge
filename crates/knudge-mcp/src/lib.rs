//! # knudge-mcp
//!
//! Servidor MCP do knudge (E12-T03). Esqueleto: ainda não abre I/O nem fala o protocolo MCP.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Nome do servidor anunciado ao cliente MCP.
pub const SERVER_NAME: &str = "knudge";
