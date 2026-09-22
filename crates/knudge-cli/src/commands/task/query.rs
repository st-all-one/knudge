//! Listagem e exibição de tarefas (`kd task list`/`show`) (E12-T01).

use std::collections::BTreeSet;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::{BlockReason, block_reason, compute_views_at};
use knudge_core::schema::{Scope, Status};
use knudge_core::store::Note;
use knudge_core::task::{is_task, parent_of, root_for_path, subtree};
use knudge_core::time::Timestamp;
use knudge_core::write::history;
use serde_json::json;

use crate::cli::TaskListArgs;
use crate::output::Output;
use crate::session::Session;

use super::ShowMode;

/// Conjunto de ids permitidos por uma view + o grafo (para `--explain`).
type View = (Graph, BTreeSet<String>);

/// `kd task list` (com views `--ready`/`--blocked` e `--explain` — D104).
///
/// # Errors
/// Propaga erros de leitura do store e valida a combinação de flags.
pub(super) fn list(session: &Session, args: &TaskListArgs) -> Result<Output> {
    validate_list_args(args)?;
    let store = session.store();
    let scope = args.scope.as_deref().map(str::parse::<Scope>).transpose()?;
    let status = args
        .status
        .as_deref()
        .map(str::parse::<Status>)
        .transpose()?;
    let view = list_view(session, args)?;
    let mut rows = Vec::new();
    let mut json_rows = Vec::new();
    for id in store.list_ids()? {
        let note = store.read(&id)?;
        if !passes_filters(&note, scope, status, args.parent.as_deref(), view.as_ref())? {
            continue;
        }
        let reason = if args.explain {
            view.as_ref()
                .and_then(|(graph, _)| block_reason(graph, &id, session.now_ms()))
        } else {
            None
        };
        let (line, row) = render_row(&note, &id, reason.as_ref())?;
        rows.push(line);
        json_rows.push(row);
    }
    Ok(Output::new(rows.join("\n"), json!({ "tasks": json_rows })))
}

fn validate_list_args(args: &TaskListArgs) -> Result<()> {
    if args.ready && args.blocked {
        return Err(Error::invalid_input(
            "`--ready` e `--blocked` são mutuamente exclusivos",
        ));
    }
    if args.explain && !args.blocked {
        return Err(Error::invalid_input("`--explain` exige `--blocked`"));
    }
    Ok(())
}

fn list_view(session: &Session, args: &TaskListArgs) -> Result<Option<View>> {
    if !args.ready && !args.blocked {
        return Ok(None);
    }
    let graph = session.graph()?;
    let views = compute_views_at(&graph, session.now_ms());
    let allowed = if args.blocked {
        views.blocked
    } else {
        views.ready
    };
    Ok(Some((graph, allowed)))
}

fn passes_filters(
    note: &Note,
    scope: Option<Scope>,
    status: Option<Status>,
    parent: Option<&str>,
    view: Option<&View>,
) -> Result<bool> {
    if !is_task(note) {
        return Ok(false);
    }
    if let Some(scope) = scope
        && note.frontmatter.scope()? != Some(scope)
    {
        return Ok(false);
    }
    if let Some(status) = status
        && note.frontmatter.status()? != status
    {
        return Ok(false);
    }
    if let Some(parent) = parent
        && parent_of(note).as_deref() != Some(parent)
    {
        return Ok(false);
    }
    if let Some((_, allowed)) = view
        && !allowed.contains(note.id()?)
    {
        return Ok(false);
    }
    Ok(true)
}

fn render_row(
    note: &Note,
    id: &str,
    reason: Option<&BlockReason>,
) -> Result<(String, serde_json::Value)> {
    let statement = note.frontmatter.statement().unwrap_or_default().to_string();
    let scope = note.frontmatter.scope()?;
    let status = note.frontmatter.status()?;
    let mut line = format!(
        "{id}|{}|{}|{statement}",
        scope.map_or("", Scope::as_str),
        status.as_str()
    );
    if let Some(reason) = reason {
        line.push('|');
        line.push_str(&reason_label(reason));
    }
    let row = json!({
        "id": id,
        "scope": scope.map(Scope::as_str),
        "status": status.as_str(),
        "statement": statement,
        "parent": parent_of(note),
        "blocked_reason": reason.map(reason_label),
    });
    Ok((line, row))
}

fn reason_label(reason: &BlockReason) -> String {
    match reason {
        BlockReason::Dependency(id) => format!("blocked_by={id}"),
        BlockReason::Scheduled(ms) => {
            format!("not_before={}", Timestamp::from_millis(*ms).to_rfc3339())
        }
        BlockReason::Cycle => "cycle".to_string(),
    }
}

/// `kd task graph --program` — árvore do programa externo (`plan/*.md`) — D119.
///
/// # Errors
/// Retorna `ErrorKind::NotFound` se o programa não tiver Épico-raiz; propaga erros de I/O.
pub(super) fn program_tree(session: &Session, path: &str) -> Result<Output> {
    let store = session.store();
    let mut notes = Vec::new();
    for id in store.list_ids()? {
        notes.push(store.read(&id)?);
    }
    let Some(root) = root_for_path(&notes, path)? else {
        return Err(Error::not_found(format!(
            "nenhum Épico-raiz ancorado a {path}"
        )));
    };
    let graph = session.graph()?;
    let tree = subtree(&graph, &root);
    let mut lines = vec![path.to_string()];
    let mut json_rows = Vec::new();
    for entry in &tree {
        let note = store.read(&entry.id)?;
        let statement = note.frontmatter.statement().unwrap_or_default().to_string();
        let scope = note.frontmatter.scope()?;
        let status = note.frontmatter.status()?;
        let indent = "  ".repeat(entry.depth.saturating_add(1));
        lines.push(format!(
            "{indent}{}|{}|{}|{statement}",
            entry.id,
            scope.map_or("", Scope::as_str),
            status.as_str()
        ));
        json_rows.push(json!({
            "id": entry.id,
            "depth": entry.depth,
            "scope": scope.map(Scope::as_str),
            "status": status.as_str(),
            "statement": statement,
        }));
    }
    let data = json!({ "program": path, "root": root, "nodes": json_rows });
    Ok(Output::new(lines.join("\n"), data))
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
