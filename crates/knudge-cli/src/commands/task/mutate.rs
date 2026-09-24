//! Atualização e fechamento de tarefas (`kd task update`/`close`) (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::health::close_task;
use knudge_core::schema::{Status, Value};
use knudge_core::store::{Event, Note};
use knudge_core::task::{
    OutcomeStatus, TaskAction, apply, epic_of, membership, outcome, progress_of, validate_parent,
    validate_transition,
};
use knudge_core::write::WriteContext;
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::super::validators;

/// `kd task update`.
///
/// # Errors
/// Propaga erros de validação/escrita.
#[allow(
    clippy::too_many_arguments,
    reason = "campos independentes do patch de tarefa"
)]
pub(super) fn update(
    session: &Session,
    id: &str,
    statement: Option<&str>,
    status: Option<&str>,
    parent: Option<&str>,
    checks: &[String],
) -> Result<Output> {
    let ctx = session.write_context()?;
    let mut note = ctx.store().read(id)?;
    if let Some(statement) = statement {
        note.frontmatter
            .set("statement", Value::Str(statement.to_string()))?;
    }
    if let Some(status) = status {
        let target: Status = status.parse()?;
        validate_transition(note.frontmatter.status()?, target)?;
        note.frontmatter
            .set("status", Value::Str(target.as_str().to_string()))?;
    }
    if !checks.is_empty() {
        let items = checks.iter().map(|c| Value::Str(c.clone())).collect();
        note.frontmatter.set("checks", Value::List(items))?;
    }
    if let Some(parent) = parent {
        reparent(&ctx, &mut note, parent)?;
    }
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.refresh_body_hash()?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    let event = Event::new("task", ctx.now_ms())
        .with_note_id(id)
        .with_data("action", Value::Str("update".to_string()));
    ctx.events().append(&event)?;
    let data = json!({ "id": id, "revision": revision });
    Ok(Output::new(format!("updated|{id}|r{revision}"), data))
}

fn reparent(ctx: &WriteContext<'_>, note: &mut Note, parent: &str) -> Result<()> {
    let parent_note = ctx.store().read(parent)?;
    let parent_scope = parent_note
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema(format!("pai {parent} não tem `scope`")))?;
    let child_scope = note
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema("nota não tem `scope`"))?;
    validate_parent(parent_scope, child_scope)?;
    let blocks = membership::parse(&note.body).and_then(|marker| marker.blocks);
    note.body = membership::set(&note.body, parent, blocks);
    Ok(())
}

/// `kd task close` (com `--outcome` ou por validators).
///
/// # Errors
/// Propaga erros de evidência/validação e I/O.
pub(super) fn close(
    session: &Session,
    id: &str,
    outcome_arg: Option<&str>,
    note_arg: Option<&str>,
) -> Result<Output> {
    if note_arg.is_some() && outcome_arg.is_none() {
        return Err(Error::invalid_input(
            "`--note` exige `--outcome` (o motivo entra em `outcomes[].notes`)",
        ));
    }
    let ctx = session.write_context()?;
    if let Some(value) = outcome_arg {
        let status = value.parse::<OutcomeStatus>()?;
        let _ignored = outcome(&ctx, id, status, note_arg)?;
        let revision = apply(&ctx, id, TaskAction::Review)?;
        let data = json!({ "id": id, "outcome": status.as_str(), "revision": revision });
        return finish_close(
            session,
            id,
            format!("closed|{id}|{}|r{revision}", status.as_str()),
            data,
            Vec::new(),
        );
    }
    let note = ctx.store().read(id)?;
    let run = validators::run(session, &note)?;
    if run.outcomes.is_empty() {
        return Err(Error::invalid_input(
            "sem validators; use `--outcome` ou configure `.knudge/validators.toml` (D54)",
        ));
    }
    let closed = close_task(&ctx, id, &run.outcomes, None)?;
    let data = json!({
        "id": closed.id,
        "outcome": closed.status.as_str(),
        "revision": closed.revision,
        "checks": run.outcomes.iter().map(|check| json!({
            "name": check.name,
            "result": check.result.as_str(),
            "severity": check.severity.as_str(),
        })).collect::<Vec<_>>(),
    });
    finish_close(
        session,
        id,
        format!(
            "closed|{id}|{}|r{}",
            closed.status.as_str(),
            closed.revision
        ),
        data,
        run.warnings,
    )
}

/// Monta a saída de `close` com o rollup do épico (D127).
fn finish_close(
    session: &Session,
    id: &str,
    mut text: String,
    mut data: serde_json::Value,
    warnings: Vec<String>,
) -> Result<Output> {
    if let Some((epic_text, epic_json)) = epic_rollup(session, id)? {
        text.push('\n');
        text.push_str(&epic_text);
        if let Some(map) = data.as_object_mut() {
            let _ignored = map.insert("epic".to_string(), epic_json);
        }
    }
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Épico mais próximo + progresso derivado; `None` se o item não pertence a um épico (D127).
fn epic_rollup(session: &Session, id: &str) -> Result<Option<(String, serde_json::Value)>> {
    let graph = session.graph()?;
    let Some(epic_id) = epic_of(&graph, id) else {
        return Ok(None);
    };
    let progress = progress_of(&graph, &epic_id);
    let statement = session
        .store()
        .read(&epic_id)
        .ok()
        .and_then(|note| note.frontmatter.statement().ok().map(str::to_string))
        .unwrap_or_default();
    let text = format!("epico: {epic_id}|{statement} ({})", progress.label());
    let value = json!({
        "id": epic_id,
        "statement": statement,
        "done": progress.done,
        "total": progress.total,
    });
    Ok(Some((text, value)))
}
