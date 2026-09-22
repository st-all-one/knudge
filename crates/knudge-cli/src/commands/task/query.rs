//! Listagem e exibição de tarefas (`kd task list`/`show`) (E12-T01).

use knudge_core::Result;
use knudge_core::schema::{Scope, Status};
use knudge_core::task::{is_task, parent_of};
use knudge_core::write::history;
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::ShowMode;

/// `kd task list`.
///
/// # Errors
/// Propaga erros de leitura do store.
pub(super) fn list(
    session: &Session,
    scope: Option<&str>,
    status: Option<&str>,
    parent: Option<&str>,
) -> Result<Output> {
    let store = session.store();
    let scope = scope.map(str::parse::<Scope>).transpose()?;
    let status = status.map(str::parse::<Status>).transpose()?;
    let mut rows = Vec::new();
    let mut json_rows = Vec::new();
    for id in store.list_ids()? {
        let note = store.read(&id)?;
        if !is_task(&note) {
            continue;
        }
        if let Some(scope) = scope
            && note.frontmatter.scope()? != Some(scope)
        {
            continue;
        }
        if let Some(status) = status
            && note.frontmatter.status()? != status
        {
            continue;
        }
        if let Some(parent) = parent
            && parent_of(&note).as_deref() != Some(parent)
        {
            continue;
        }
        let statement = note.frontmatter.statement().unwrap_or_default().to_string();
        let scope = note.frontmatter.scope()?;
        let status = note.frontmatter.status()?;
        rows.push(format!(
            "{id}|{}|{}|{statement}",
            scope.map_or("", Scope::as_str),
            status.as_str()
        ));
        json_rows.push(json!({
            "id": id,
            "scope": scope.map(Scope::as_str),
            "status": status.as_str(),
            "statement": statement,
            "parent": parent_of(&note),
        }));
    }
    Ok(Output::new(rows.join("\n"), json!({ "tasks": json_rows })))
}

/// `kd task show`.
///
/// # Errors
/// Propaga erros de leitura do store e de histórico.
pub(super) fn show(session: &Session, id: &str, mode: ShowMode) -> Result<Output> {
    let store = session.store();
    let note = store.read(id)?;
    let statement = note.frontmatter.statement().unwrap_or_default().to_string();
    let scope = note.frontmatter.scope()?;
    let status = note.frontmatter.status()?;
    let chain = if mode == ShowMode::History {
        history(&store, id)?
    } else {
        Vec::new()
    };
    let ids: Vec<String> = chain
        .iter()
        .map(|note| note.frontmatter.id().unwrap_or_default().to_string())
        .collect();
    let text = if ids.is_empty() {
        format!("{id}|{statement}")
    } else {
        format!("{id}|{statement}\nhistorico: {}", ids.join(" -> "))
    };
    let data = json!({
        "id": id,
        "statement": statement,
        "scope": scope.map(Scope::as_str),
        "status": status.as_str(),
        "body": note.body,
        "history": ids,
    });
    Ok(Output::new(text, data))
}
