//! Escrita em stdout/stderr com o contrato **stdout = dados, stderr = logs** (R20).
//!
//! `EPIPE` (pipe fechado, ex.: `kd … | head`) termina com **exit 0** (D73).

use std::io::Write;
use std::process::ExitCode;

use knudge_core::ErrorKind;
use serde_json::Value;

/// Carga útil de um comando bem-sucedido.
#[derive(Debug)]
pub struct Payload {
    /// Representação para o pipe (LLM).
    pub text: String,
    /// Representação para o envelope JSON.
    pub json: Value,
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
