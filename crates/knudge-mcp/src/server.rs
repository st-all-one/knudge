//! Dispatcher MCP puro (E14-T02/T03, D68).
//!
//! Sem I/O: recebe uma [`Request`] já decodificada e devolve a resposta (ou `None` para
//! notificações). O transporte fica em [`crate::transport`].

use serde_json::{Value, json};

use crate::jsonrpc::{self, INVALID_PARAMS, METHOD_NOT_FOUND, Request, RpcError};
use crate::protocol::{self, SERVER_NAME};
use crate::tools;
use crate::triggers::HintEngine;

/// Versão do servidor (a do crate).
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Servidor MCP sem I/O, com o estado do motor de hints.
pub struct McpServer {
    engine: HintEngine,
    initialized: bool,
    protocol_version: &'static str,
}

impl McpServer {
    /// Cria o servidor com cap de hints e sessões de observação.
    #[must_use]
    pub fn new(cap: usize, observation_sessions: u32) -> Self {
        Self {
            engine: HintEngine::new(cap, observation_sessions),
            initialized: false,
            protocol_version: protocol::PROTOCOL_VERSION,
        }
    }

    /// Versão do protocolo negociada.
    #[must_use]
    pub const fn protocol_version(&self) -> &'static str {
        self.protocol_version
    }

    /// `true` após `notifications/initialized`.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Processa uma mensagem; `None` = notificação (sem resposta).
    pub fn handle(&mut self, request: &Request) -> Option<Value> {
        if request.is_notification() {
            self.handle_notification(&request.method);
            return None;
        }
        let id = request.id.as_ref()?;
        let response = match self.dispatch(&request.method, &request.params) {
            Ok(result) => jsonrpc::result(id, &result),
            Err(error) => jsonrpc::error(Some(id), &error),
        };
        Some(response)
    }

    fn handle_notification(&mut self, method: &str) {
        if method == "notifications/initialized" {
            self.initialized = true;
        }
    }

    fn dispatch(&mut self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "initialize" => Ok(self.initialize(params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tools::list()),
            "tools/call" => self.call_tool(params),
            other => Err(RpcError::new(
                METHOD_NOT_FOUND,
                format!("método desconhecido: {other}"),
            )),
        }
    }

    fn initialize(&mut self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        self.protocol_version = protocol::negotiate(requested);
        json!({
            "protocolVersion": self.protocol_version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
        })
    }

    fn call_tool(&mut self, params: &Value) -> Result<Value, RpcError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcError::new(INVALID_PARAMS, "campo name ausente"))?;
        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
        Ok(tools::call(&mut self.engine, name, &arguments))
    }
}
