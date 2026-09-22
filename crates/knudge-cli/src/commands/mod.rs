//! Dispatch dos verbos do `kd` para o domínio (E12-T01).

pub mod ask;
pub mod config_cmd;
pub mod embedder;
pub mod forget_sync;
pub mod hooks;
pub mod init;
pub mod maintenance;
pub mod parse;
pub mod prime;
pub mod rewind;
pub mod self_cmd;
pub mod task;
pub mod validators;
pub mod write_cmd;

use knudge_core::{Error, Result};

use crate::cli::{Cli, Command, SelfCommand};
use crate::output::Output;
use crate::session::Session;

/// Executa o comando e devolve a saída (texto + JSON + avisos).
///
/// # Errors
/// Propaga o erro do domínio; `strict` promove avisos a erro (D94).
pub fn run(cli: &Cli) -> Result<Output> {
    match cli.command.as_ref() {
        None => Ok(prime::run(prime::PrimeFormat::Short)),
        Some(Command::Prime(args)) => Ok(prime::run(if args.long {
            prime::PrimeFormat::Long
        } else {
            prime::PrimeFormat::Short
        })),
        Some(Command::SelfCmd {
            command: SelfCommand::Version,
        }) => Ok(self_cmd::version()),
        Some(Command::SelfCmd {
            command: SelfCommand::Completions { shell },
        }) => self_cmd::completions(shell),
        Some(command) => run_session(command),
    }
}

fn run_session(command: &Command) -> Result<Output> {
    let session = Session::open()?;
    let output = dispatch(&session, command)?;
    if session.config().strict() && !output.warnings.is_empty() {
        return Err(Error::config(format!(
            "modo estrito: {}",
            output.warnings.join("; ")
        )));
    }
    Ok(output)
}

fn dispatch(session: &Session, command: &Command) -> Result<Output> {
    match command {
        Command::Init(args) => init::run(session, args),
        Command::Rewind(args) => rewind::run(session, args),
        Command::Ask(args) => ask::run(session, args),
        Command::Write(args) => write_cmd::run(session, args),
        Command::Task { command } => task::run(session, command),
        Command::Maintenance { command } => maintenance::run(session, command),
        Command::Config { command } => config_cmd::run(session, command),
        Command::Forget(args) => forget_sync::forget(session, args),
        Command::Sync(args) => forget_sync::sync(session, args),
        Command::SelfCmd { command } => self_cmd::setup(session, command),
        Command::Prime(args) => Ok(prime::run(if args.long {
            prime::PrimeFormat::Long
        } else {
            prime::PrimeFormat::Short
        })),
    }
}
