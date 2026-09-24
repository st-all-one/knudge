//! `kd knowledge` — mapa de conhecimento (clusters estruturais e semânticos) — D128.

pub mod hub;
pub mod map;

use knudge_core::Result;

use crate::cli::KnowledgeCommand;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd knowledge <subcomando>`.
///
/// # Errors
/// Propaga erros do domínio (leitura de índice/store/config).
pub fn run(session: &Session, command: &KnowledgeCommand) -> Result<Output> {
    match command {
        KnowledgeCommand::Map {
            axis,
            scope,
            semantic,
            members,
            write,
        } => map::run(
            session,
            axis.as_deref(),
            scope.as_deref(),
            *semantic,
            *members,
            *write,
        ),
    }
}
