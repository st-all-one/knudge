//! `kd maintenance` — compact, learn, prune e watch-service (E12-T01).
//!
//! Saúde do corpus (`doctor`/auditoria) é verbo de topo: `kd doctor` (D163).

pub mod extra;
pub mod proposals;
pub mod watch;

use knudge_core::Result;

use crate::cli::MaintenanceCommand;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd maintenance <subcomando>`.
///
/// # Errors
/// Propaga erros de leitura/escrita do domínio.
pub fn run(session: &Session, command: &MaintenanceCommand) -> Result<Output> {
    match command {
        MaintenanceCommand::Compact {
            scope,
            corpus,
            verify,
        } => extra::compact(
            session,
            scope.as_deref(),
            corpus,
            if *verify {
                proposals::VerifyMode::Run
            } else {
                proposals::VerifyMode::Skip
            },
        ),
        MaintenanceCommand::Learn {
            scope,
            corpus,
            verify,
        } => extra::learn_cmd(
            session,
            scope.as_deref(),
            corpus,
            if *verify {
                proposals::VerifyMode::Run
            } else {
                proposals::VerifyMode::Skip
            },
        ),
        MaintenanceCommand::Prune { scope, corpus, .. } => {
            extra::prune(session, scope.as_deref(), corpus)
        }
        MaintenanceCommand::WatchService(args) => watch::run(session, args),
    }
}
