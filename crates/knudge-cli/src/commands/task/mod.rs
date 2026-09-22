//! `kd task` — plan/epic/issue/task (E12-T01, D93).

mod create;
mod mutate;
mod query;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::Scope;
use knudge_core::task::{TaskAction, TaskSpec, apply, reorder, submit};
use serde_json::json;

use crate::cli::TaskCommand;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd task <subcomando>`.
///
/// # Errors
/// Propaga erros de hierarquia, validação e I/O.
#[allow(clippy::too_many_lines, reason = "dispatch de subcomandos de tarefa")]
pub fn run(session: &Session, command: &TaskCommand) -> Result<Output> {
    match command {
        TaskCommand::New(args) => create::new_task(session, args),
        TaskCommand::List(args) => query::list(session, args),
        TaskCommand::Show { id, history } => query::show(
            session,
            id,
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
        TaskCommand::Close { id, outcome } => mutate::close(session, id, outcome.as_deref()),
        TaskCommand::Graph { program } => query::program_tree(session, program),
        TaskCommand::Plan {
            id,
            steps,
            submit,
            adopt,
            reorder,
            release,
            review,
        } => {
            let intent = if *submit {
                PlanIntent::Submit
            } else if *adopt {
                PlanIntent::Adopt
            } else if *release {
                PlanIntent::Release
            } else if *review {
                PlanIntent::Review
            } else if let Some(blocks) = reorder {
                PlanIntent::Reorder(*blocks)
            } else {
                PlanIntent::None
            };
            plan(session, id, steps, intent)
        }
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

/// Intenção de `kd task plan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanIntent {
    /// Submete os `--step` como filhos.
    Submit,
    /// Adota o plano.
    Adopt,
    /// Libera o plano.
    Release,
    /// Marca para revisão.
    Review,
    /// Reordena para a posição 1-based.
    Reorder(u32),
    /// Nenhuma flag.
    None,
}

fn plan(session: &Session, id: &str, steps: &[String], intent: PlanIntent) -> Result<Output> {
    let ctx = session.write_context()?;
    if !steps.is_empty() {
        let mut ids = Vec::new();
        for (index, step) in steps.iter().enumerate() {
            let mut spec = TaskSpec::new(Scope::Issue, step.clone());
            spec.parent = Some(id.to_string());
            spec.blocks = Some(u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX));
            ids.push(submit(&ctx, &spec)?.id);
        }
        let text = ids.join("\n");
        return Ok(Output::new(text, json!({ "plan": id, "children": ids })));
    }
    let action = match intent {
        PlanIntent::Adopt => Some(TaskAction::Adopt),
        PlanIntent::Release => Some(TaskAction::Release),
        PlanIntent::Review => Some(TaskAction::Review),
        PlanIntent::Submit | PlanIntent::Reorder(_) | PlanIntent::None => None,
    };
    if let Some(action) = action {
        let revision = apply(&ctx, id, action)?;
        let data = json!({ "id": id, "action": action.as_str(), "revision": revision });
        return Ok(Output::new(
            format!("{}|{id}|r{revision}", action.as_str()),
            data,
        ));
    }
    if let PlanIntent::Reorder(blocks) = intent {
        let revision = reorder(&ctx, id, blocks)?;
        let data = json!({ "id": id, "action": "reorder", "blocks": blocks, "revision": revision });
        return Ok(Output::new(
            format!("reorder|{id}|{blocks}|r{revision}"),
            data,
        ));
    }
    if intent == PlanIntent::Submit {
        return Err(Error::invalid_input("`--submit` exige `--step` (D53)"));
    }
    Err(Error::invalid_input(
        "use `--step`, `--adopt`, `--release`, `--review` ou `--reorder`",
    ))
}
