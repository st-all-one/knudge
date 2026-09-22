//! Envelope JSON de máquina (D71/R31): `{success, command, data?, error?, warnings?}`.

use knudge_core::Error;
use serde::Serialize;
use serde_json::Value;

/// Envelope de resposta.
#[derive(Debug, Serialize)]
pub struct Envelope {
    /// `true` em sucesso.
    success: bool,
    /// Nome canônico do comando.
    command: String,
    /// Dados do comando (ausente em erro).
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    /// Erro estruturado (ausente em sucesso).
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
    /// Avisos não fatais (R33).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

/// Corpo do erro: código estável + `retryable` (R31).
#[derive(Debug, Serialize)]
struct ErrorBody {
    /// Código estável de máquina.
    code: &'static str,
    /// Mensagem humana.
    message: String,
    /// Se a operação pode ser repetida.
    retryable: bool,
    /// Detalhes estruturados opcionais (ex.: ids, contagens).
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

/// Fallback caso a serialização falhe (não deve acontecer).
const FALLBACK: &str = r#"{"success":false,"error":{"code":"internal","message":"falha ao serializar envelope","retryable":false}}"#;

impl Envelope {
    /// Envelope de sucesso.
    #[must_use]
    pub fn success(command: &str, data: Option<Value>, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            command: command.to_string(),
            data,
            error: None,
            warnings,
        }
    }

    /// Envelope de erro.
    #[must_use]
    pub fn failure(command: &str, err: &Error, warnings: Vec<String>) -> Self {
        Self {
            success: false,
            command: command.to_string(),
            data: None,
            error: Some(ErrorBody {
                code: err.kind().code(),
                message: err.to_string(),
                retryable: err.retryable(),
                details: None,
            }),
            warnings,
        }
    }

    /// Serializa em uma linha JSON.
    #[must_use]
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| FALLBACK.to_string())
    }
}
