//! # knudge-cli
//!
//! Implementação do binário `kd` (superfície v2). O contrato dos verbos está em
//! `plan/implementation/16_cli_surface.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod envelope;
pub mod logging;
pub mod output;

use std::process::ExitCode;

use clap::Parser;
use knudge_core::{Error, ErrorKind};

use crate::cli::{Cli, Command, SelfCommand};
use crate::envelope::Envelope;
use crate::output::{Payload, emit_stderr, emit_stdout};

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocolo estático (placeholder de E01; finalizado em E12-T01).
const PRIME_TEXT: &str = "\
knudge (kd) — memória otimizada para LLM
Uso: kd <comando> [opções]
Comandos: init, prime, rewind, ask, write, task, maintenance, config, forget, sync, self
Sem argumentos, kd executa `prime`.
";

/// Ponto de entrada do binário.
#[must_use]
pub fn run() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => return handle_parse_error(&err),
    };
    logging::init(&cli);
    let name = command_name(cli.command.as_ref());
    match dispatch(&cli) {
        Ok(payload) => emit_success(&cli, name, &payload),
        Err(err) => emit_error(&cli, name, &err),
    }
}

/// Nome canônico do comando (ou `prime` quando ausente).
fn command_name(command: Option<&Command>) -> &'static str {
    command.map_or("prime", Command::name)
}

/// Executa o comando. Esqueleto de E01: só `prime` e `self version` respondem.
fn dispatch(cli: &Cli) -> Result<Payload, Error> {
    match cli.command.as_ref() {
        None | Some(Command::Prime(_)) => Ok(prime_payload()),
        Some(Command::SelfCmd {
            command: SelfCommand::Version,
        }) => Ok(version_payload()),
        Some(other) => Err(Error::internal(format!(
            "subcomando `{}` ainda não implementado",
            other.name()
        ))),
    }
}

/// Carga útil de `prime`.
fn prime_payload() -> Payload {
    Payload {
        text: PRIME_TEXT.to_string(),
        json: serde_json::json!({ "protocol": PRIME_TEXT, "version": VERSION }),
    }
}

/// Carga útil de `self version`.
fn version_payload() -> Payload {
    Payload {
        text: format!("kd {VERSION}"),
        json: serde_json::json!({ "version": VERSION }),
    }
}

/// Emite o resultado (texto no pipe ou envelope JSON).
fn emit_success(cli: &Cli, command: &str, payload: &Payload) -> ExitCode {
    if cli.json {
        let envelope = Envelope::success(command, Some(payload.json.clone()));
        emit_stdout(format!("{}\n", envelope.to_json_line()).as_bytes())
    } else {
        emit_stdout(format!("{}\n", payload.text).as_bytes())
    }
}

/// Emite o erro (stderr no pipe ou envelope JSON) e devolve o exit code.
fn emit_error(cli: &Cli, command: &str, err: &Error) -> ExitCode {
    if cli.json {
        let envelope = Envelope::failure(command, err);
        let _ignored = emit_stdout(format!("{}\n", envelope.to_json_line()).as_bytes());
    } else {
        emit_stderr(format!("erro: {err}\n").as_bytes());
    }
    ExitCode::from(err.kind().exit_code())
}

/// Trata erros de parsing do `clap` (help/version em stdout; erros em stderr).
fn handle_parse_error(err: &clap::Error) -> ExitCode {
    let to_stderr = err.use_stderr();
    let rendered = err.render().to_string();
    if to_stderr {
        emit_stderr(rendered.as_bytes());
        ExitCode::from(ErrorKind::InvalidInput.exit_code())
    } else {
        emit_stdout(rendered.as_bytes())
    }
}
