//! `kd task` — epic/issue/task (E12-T01, D93).

mod create;
mod graph;
mod mutate;
mod plan;
mod query;
mod render;

use knudge_core::Result;

use crate::cli::TaskCommand;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd task <subcomando>`.
///
/// # Errors
/// Propaga erros de hierarquia, validação e I/O.
pub fn run(session: &Session, command: &TaskCommand) -> Result<Output> {
    match command {
        TaskCommand::New(args) => create::new_task(session, args),
        TaskCommand::List(args) => query::list(session, args),
        TaskCommand::Show { ids, history } => query::show(
            session,
            ids,
            if *history {
                ShowMode::History
            } else {
                ShowMode::Plain
            },
        ),
        TaskCommand::Update {
            id,
            statement,
            status,
            parent,
            checks,
        } => mutate::update(
            session,
            id,
            statement.as_deref(),
            status.as_deref(),
            parent.as_deref(),
            checks,
        ),
        TaskCommand::Close { id, outcome, note } => {
            mutate::close(session, id, outcome.as_deref(), note.as_deref())
        }
        TaskCommand::Graph { program, root } => {
            graph::graph_tree(session, program.as_deref(), root.as_deref())
        }
        TaskCommand::Plan(args) => plan::run(session, args),
    }
}

/// Modo de `kd task show`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShowMode {
    /// Só a tarefa.
    Plain,
    /// Tarefa + histórico de supersessão.
    History,
}
