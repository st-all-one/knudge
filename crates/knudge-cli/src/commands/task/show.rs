//! Exibição completa de tarefas: `kd task show` e `list --full-content` (D137).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::schema::{Scope, Status, Value};
use knudge_core::store::{Note, Store};
use knudge_core::task::{TaskContext, TaskRef, context_of};
use knudge_core::write::history;
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::ShowMode;

/// `kd task show <ID> [<ID> ...]` (T5/D104): separa por `\n---\n`; ids ausentes viram aviso.
///
/// # Errors
/// Propaga o erro do store quando **nenhum** id pôde ser lido.
pub(super) fn show(session: &Session, ids: &[String], mode: ShowMode) -> Result<Output> {
    let store = session.store();
    let graph = session.graph()?;
    let mut blocks = Vec::new();
    let mut data = Vec::new();
    let mut warnings = Vec::new();
    let mut last_error = None;
    for id in ids {
        match show_one(&store, &graph, id, mode) {
            Ok((text, value)) => {
                blocks.push(text);
                data.push(value);
            }
            Err(error) => {
                warnings.push(format!("{id}: {error}"));
                last_error = Some(error);
            }
        }
    }
    if data.is_empty() {
        let Some(error) = last_error else {
            return Err(Error::not_found("nenhuma tarefa encontrada"));
        };
        return Err(error);
    }
    Ok(Output::new(blocks.join("\n---\n"), json!({ "tasks": data })).with_warnings(warnings))
}

/// Bloco (texto + JSON) de uma tarefa; reusado por `list --full-content`.
pub(super) fn show_one(
    store: &Store<'_>,
    graph: &Graph,
    id: &str,
    mode: ShowMode,
) -> Result<(String, serde_json::Value)> {
    let note = store.read(id)?;
    let chain = if mode == ShowMode::History {
        history(store, id)?
    } else {
        Vec::new()
    };
    let ids: Vec<String> = chain
        .iter()
        .map(|note| note.frontmatter.id().unwrap_or_default().to_string())
        .collect();
    let context = context_of(store, graph, id)?;
    let text = block_text(id, &note, &context, &ids)?;
    let data = block_json(id, &note, &context, &ids)?;
    Ok((text, data))
}

/// Bloco de texto completo de `show`/`list --full-content` (D137).
fn block_text(id: &str, note: &Note, context: &TaskContext, ids: &[String]) -> Result<String> {
    let statement = note.frontmatter.statement().unwrap_or_default();
    let mut lines = vec![format!("{id}|{statement}")];
    if let Some(scope) = note.frontmatter.scope()? {
        lines.push(format!("scope: {}", scope.as_str()));
    }
    lines.push(format!("tipo: {}", note.frontmatter.note_type()?.as_str()));
    lines.push(format!("status: {}", note.frontmatter.status()?.as_str()));
    if !note.body.trim().is_empty() {
        lines.push("corpo:".to_string());
        lines.extend(note.body.lines().map(|line| format!("  {line}")));
    }
    push_list(
        &mut lines,
        "checks",
        &note.frontmatter.string_list("checks")?,
    );
    push_list(
        &mut lines,
        "ancoras",
        &note.frontmatter.string_list("anchors")?,
    );
    push_list(&mut lines, "tags", &note.frontmatter.string_list("tags")?);
    push_list(&mut lines, "outcomes", &outcome_labels(note));
    lines.extend(context_lines(id, context));
    if !ids.is_empty() {
        lines.push(format!("historico: {}", ids.join(" -> ")));
    }
    Ok(lines.join("\n"))
}

/// Acrescenta `rotulo: a, b` quando a lista não está vazia.
fn push_list<S: AsRef<str>>(lines: &mut Vec<String>, label: &str, items: &[S]) {
    if items.is_empty() {
        return;
    }
    let joined = items
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!("{label}: {joined}"));
}

/// Objeto JSON completo de `show`/`list --full-content` (D137).
fn block_json(
    id: &str,
    note: &Note,
    context: &TaskContext,
    ids: &[String],
) -> Result<serde_json::Value> {
    let statement = note.frontmatter.statement().unwrap_or_default();
    let scope = note.frontmatter.scope()?;
    let kind = note.frontmatter.note_type()?;
    let status = note.frontmatter.status()?;
    let checks = note.frontmatter.string_list("checks")?;
    let anchors = note.frontmatter.string_list("anchors")?;
    let tags = note.frontmatter.string_list("tags")?;
    let outcome_values = outcome_json(note);
    Ok(json!({
        "id": id,
        "statement": statement,
        "scope": scope.map(Scope::as_str),
        "kind": kind.as_str(),
        "status": status.as_str(),
        "body": &note.body,
        "checks": checks,
        "anchors": anchors,
        "tags": tags,
        "outcomes": outcome_values,
        "history": ids,
        "parent": context.parent.as_ref().map(ref_json),
        "blocked_by": context.blocked_by.iter().map(ref_json).collect::<Vec<_>>(),
        "blocks": context.blocks.iter().map(ref_json).collect::<Vec<_>>(),
        "children": context.children.iter().map(ref_json).collect::<Vec<_>>(),
        "epic": context.epic.as_ref().map(|entry| json!({
            "id": entry.epic.id,
            "statement": entry.epic.statement,
            "scope": entry.epic.scope.map(Scope::as_str),
            "status": entry.epic.status.as_str(),
            "done": entry.progress.done,
            "total": entry.progress.total,
        })),
    }))
}

/// Rótulos legíveis de `outcomes` para o texto (`status`/`status(nota)`).
fn outcome_labels(note: &Note) -> Vec<String> {
    note.frontmatter
        .get("outcomes")
        .and_then(Value::as_list)
        .into_iter()
        .flatten()
        .filter_map(Value::as_map)
        .map(|map| {
            let status = map.get("status").and_then(Value::as_str).unwrap_or("?");
            match map.get("notes").and_then(Value::as_str) {
                Some(notes) => format!("{status}({notes})"),
                None => status.to_string(),
            }
        })
        .collect()
}

/// `outcomes` como objetos JSON (`status`/`recorded_at`/`notes`).
fn outcome_json(note: &Note) -> Vec<serde_json::Value> {
    note.frontmatter
        .get("outcomes")
        .and_then(Value::as_list)
        .into_iter()
        .flatten()
        .filter_map(Value::as_map)
        .map(|map| {
            json!({
                "status": map.get("status").and_then(Value::as_str),
                "recorded_at": map.get("recorded_at").and_then(Value::as_str),
                "notes": map.get("notes").and_then(Value::as_str),
            })
        })
        .collect()
}

/// Linhas de contexto: `pai:`/`bloqueado_por:`/`bloqueia:`/`filhos:`/`epico:` (D125/D127).
fn context_lines(id: &str, context: &TaskContext) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(parent) = &context.parent {
        lines.push(format!("pai: {}", render_ref(parent)));
    }
    if !context.blocked_by.is_empty() {
        lines.push(format!(
            "bloqueado_por: {}",
            render_refs(&context.blocked_by)
        ));
    }
    if !context.blocks.is_empty() {
        lines.push(format!("bloqueia: {}", render_refs(&context.blocks)));
    }
    if !context.children.is_empty() {
        lines.push(format!("filhos: {}", render_refs(&context.children)));
    }
    if let Some(entry) = &context.epic {
        if entry.epic.id == id {
            lines.push(format!("progresso: {}", entry.progress.label()));
        } else {
            lines.push(format!(
                "epico: {} ({})",
                render_ref(&entry.epic),
                entry.progress.label()
            ));
        }
    }
    lines
}

/// `id|statement [status]` — status omitido quando `active` (economia de tokens).
fn render_ref(reference: &TaskRef) -> String {
    if reference.status == Status::Active {
        format!("{}|{}", reference.id, reference.statement)
    } else {
        format!(
            "{}|{} [{}]",
            reference.id,
            reference.statement,
            reference.status.as_str()
        )
    }
}

fn render_refs(refs: &[TaskRef]) -> String {
    refs.iter().map(render_ref).collect::<Vec<_>>().join(", ")
}

fn ref_json(reference: &TaskRef) -> serde_json::Value {
    json!({
        "id": reference.id,
        "statement": reference.statement,
        "scope": reference.scope.map(Scope::as_str),
        "status": reference.status.as_str(),
    })
}
