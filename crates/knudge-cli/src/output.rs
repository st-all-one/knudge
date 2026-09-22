//! Escrita em stdout/stderr com o contrato **stdout = dados, stderr = logs** (R20).
//!
//! `EPIPE` (pipe fechado, ex.: `kd … | head`) termina com **exit 0** (D73).

use std::io::Write;
use std::process::ExitCode;

use knudge_core::ErrorKind;
use serde_json::Value;

/// Resultado de um comando: texto para o pipe (LLM) + JSON para máquinas + avisos (R33).
#[derive(Debug)]
pub struct Output {
    /// Representação para o pipe (LLM).
    pub text: String,
    /// Representação para o envelope JSON.
    pub json: Value,
    /// Avisos de degradação graciosa (R33); `strict` os promove a erro (D94).
    pub warnings: Vec<String>,
}

impl Output {
    /// Saída com texto e JSON equivalentes.
    #[must_use]
    pub fn new(text: impl Into<String>, json: Value) -> Self {
        Self {
            text: text.into(),
            json,
            warnings: Vec::new(),
        }
    }

    /// Saída só com texto (JSON = `{ "text": ... }`).
    #[must_use]
    pub fn text_only(text: impl Into<String>) -> Self {
        let text = text.into();
        let json = serde_json::json!({ "text": text });
        Self {
            text,
            json,
            warnings: Vec::new(),
        }
    }

    /// Anexa avisos (degradação graciosa).
    #[must_use]
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

/// Escreve bytes em stdout.
///
/// Retorna `SUCCESS` em caso de `EPIPE`; `ErrorKind::Io` para outras falhas.
#[must_use]
pub fn emit_stdout(bytes: &[u8]) -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(ErrorKind::Io.exit_code()),
    }
}

/// Escreve bytes em stderr, ignorando falhas (logs nunca quebram o comando).
pub fn emit_stderr(bytes: &[u8]) {
    let mut stderr = std::io::stderr().lock();
    let _ignored = stderr.write_all(bytes);
    let _ignored = stderr.flush();
}
