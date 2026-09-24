//! `kd knowledge` — mapa/digestão/ranking/vocabulário de conhecimento (D128/D145/D146).

pub mod digest;
pub mod hub;
pub mod map;
pub mod promote;
pub mod rank;
pub mod suggest;
pub mod tags;

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
        KnowledgeCommand::Map(args) => map::run(session, args),
        KnowledgeCommand::Digest(args) => digest::run(session, args),
        KnowledgeCommand::Rank(args) => rank::run(session, args),
        KnowledgeCommand::Tags(args) => tags::run(session, args),
        KnowledgeCommand::Suggest(args) => suggest::run(session, args),
        KnowledgeCommand::Promote { command } => promote::run(session, command),
    }
}
