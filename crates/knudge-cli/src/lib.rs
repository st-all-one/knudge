//! # knudge-cli
//!
//! Implementação do binário `kd` (superfície v2). O contrato dos verbos está em
//! `plan/implementation/16_cli_surface.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod commands;
pub mod envelope;
pub mod logging;
pub mod output;
pub mod session;

use std::process::ExitCode;

use clap::Parser;
use knudge_core::{Error, ErrorKind};

use crate::cli::{Cli, Command};
use crate::envelope::Envelope;
use crate::output::{Output, emit_stderr, emit_stdout};

/// Ponto de entrada do binário.
#[must_use]
pub fn run() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => return handle_parse_error(&err),
    };
    logging::init(&cli);
    let name = command_name(cli.command.as_ref());
    match commands::run(&cli) {
        Ok(output) => emit_success(&cli, name, &output),
        Err(err) => emit_error(&cli, name, &err, &[]),
    }
}

/// Nome canônico do comando (ou `prime` quando ausente).
fn command_name(command: Option<&Command>) -> &'static str {
    command.map_or("prime", Command::name)
}

/// Emite o resultado (texto no pipe ou envelope JSON).
fn emit_success(cli: &Cli, command: &str, output: &Output) -> ExitCode {
    if cli.json {
        let envelope =
            Envelope::success(command, Some(output.json.clone()), output.warnings.clone());
        emit_stdout(format!("{}\n", envelope.to_json_line()).as_bytes())
    } else {
        emit_stdout(format!("{}\n", output.text).as_bytes())
    }
}

/// Emite o erro (stderr no pipe ou envelope JSON) e devolve o exit code.
fn emit_error(cli: &Cli, command: &str, err: &Error, warnings: &[String]) -> ExitCode {
    if cli.json {
        let envelope = Envelope::failure(command, err, warnings.to_vec());
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
