//! Codec JSON-RPC 2.0 (E14-T01, D68).
//!
//! Puro e sem I/O: decodifica uma mensagem (requisição ou notificação) e emite resposta ou
//! erro. O framing de linha e o loop de stdio ficam em [`crate::transport`].

use serde_json::{Map, Value, json};

/// Versão do protocolo JSON-RPC.
pub const JSONRPC_VERSION: &str = "2.0";

/// Erro de parse (`-32700`).
pub const PARSE_ERROR: i64 = -32700;
/// Requisição inválida (`-32600`).
pub const INVALID_REQUEST: i64 = -32600;
/// Método desconhecido (`-32601`).
pub const METHOD_NOT_FOUND: i64 = -32601;
/// Parâmetros inválidos (`-32602`).
pub const INVALID_PARAMS: i64 = -32602;
/// Erro interno (`-32603`).
pub const INTERNAL_ERROR: i64 = -32603;

/// Identificador de mensagem (número inteiro ou string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    /// Id numérico.
    Number(i64),
    /// Id textual.
    Text(String),
}

impl Id {
    /// Extrai um id de um `Value` (número inteiro ou string).
    #[must_use]
    pub fn from_value(value: &Value) -> Option<Self> {
        if let Some(number) = value.as_i64() {
            return Some(Self::Number(number));
        }
        value.as_str().map(|text| Self::Text(text.to_string()))
    }

    /// Serializa o id.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Number(number) => json!(number),
            Self::Text(text) => json!(text),
        }
    }
}

/// Mensagem decodificada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Id (`None` = notificação, que não recebe resposta).
    pub id: Option<Id>,
    /// Método (`initialize`, `tools/list`, …).
    pub method: String,
    /// Parâmetros (`Null` quando ausentes).
    pub params: Value,
}

impl Request {
    /// `true` quando a mensagem não espera resposta.
    #[must_use]
    pub const fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// Erro JSON-RPC com código canônico.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "`Value` não implementa `Eq` (contém f64)"
)]
#[derive(Debug, Clone, PartialEq)]
pub struct RpcError {
    /// Código canônico (`-32700`…`-32603`).
    pub code: i64,
    /// Mensagem curta.
    pub message: String,
    /// Dados adicionais opcionais.
    pub data: Option<Value>,
}

impl RpcError {
    /// Cria um erro sem `data`.
    #[must_use]
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Anexa `data`.
    #[must_use]
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Serializa o erro.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut object = Map::new();
        let _ignored = object.insert("code".to_string(), json!(self.code));
        let _ignored = object.insert("message".to_string(), json!(self.message));
        if let Some(data) = &self.data {
            let _ignored = object.insert("data".to_string(), data.clone());
        }
        Value::Object(object)
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} (código {})", self.message, self.code)
    }
}

impl std::error::Error for RpcError {}

/// Decodifica uma linha JSON em uma mensagem.
///
/// # Errors
/// Devolve [`RpcError`] com o código canônico para JSON inválido, `jsonrpc` diferente de
/// `"2.0"`, `method` ausente ou `id` de tipo inválido.
pub fn parse(line: &str) -> Result<Request, RpcError> {
    let value: Value = serde_json::from_str(line)
        .map_err(|error| RpcError::new(PARSE_ERROR, format!("json inválido: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| RpcError::new(INVALID_REQUEST, "esperado objeto JSON-RPC"))?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some(JSONRPC_VERSION) {
        return Err(RpcError::new(
            INVALID_REQUEST,
            "campo jsonrpc deve ser \"2.0\"",
        ));
    }
    let method = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::new(INVALID_REQUEST, "campo method ausente"))?
        .to_string();
    let id = match object.get("id") {
        None | Some(Value::Null) => None,
        Some(raw) => Some(Id::from_value(raw).ok_or_else(|| {
            RpcError::new(
                INVALID_REQUEST,
                "campo id deve ser número inteiro ou string",
            )
        })?),
    };
    let params = object.get("params").cloned().unwrap_or(Value::Null);
    Ok(Request { id, method, params })
}

/// Monta uma resposta de sucesso.
#[must_use]
pub fn result(id: &Id, result: &Value) -> Value {
    json!({ "jsonrpc": JSONRPC_VERSION, "id": id.to_value(), "result": result })
}

/// Monta uma resposta de erro (`id` nulo quando o parse não permitiu recuperá-lo).
#[must_use]
pub fn error(id: Option<&Id>, error: &RpcError) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id.map_or(Value::Null, Id::to_value),
        "error": error.to_value(),
    })
}
