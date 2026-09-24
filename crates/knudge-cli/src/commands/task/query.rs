//! Listagem e exibição de tarefas (`kd task list`/`show`) (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::block_reason;
use knudge_core::schema::{Scope, Status};
use knudge_core::store::Store;
use knudge_core::task::{TaskContext, TaskRef, context_of, impact, is_actionable};
use knudge_core::write::history;
use serde_json::json;

use crate::cli::{TaskListArgs, TaskSort};
use crate::output::Output;
use crate::session::Session;

use super::ShowMode;
use super::render::{ListFilters, Row, passes_filters, render_row};

/// `kd task list` (views `--ready`/`--blocked`, `--explain`, `--sort impact` e filtros).
///
/// # Errors
/// Propaga erros de leitura do store e valida a combinação de flags.
pub(super) fn list(session: &Session, args: &TaskListArgs) -> Result<Output> {
    validate_list_args(args)?;
    let needs_graph = args.ready || args.blocked || args.sort == Some(TaskSort::Impact);
    let graph = needs_graph.then(|| session.graph()).transpose()?;
    let filters = ListFilters::resolve(args, graph.as_ref())?;
    let mut rows = collect_rows(session, args, graph.as_ref(), &filters)?;
    if args.sort == Some(TaskSort::Impact) {
        rows.sort_by(|left, right| {
            right
                .impact
                .cmp(&left.impact)
                .then_with(|| left.created.cmp(&right.created))
                .then_with(|| left.id.cmp(&right.id))
        });
    }
    let mut lines = Vec::with_capacity(rows.len());
    let mut data = Vec::with_capacity(rows.len());
    for row in rows {
        lines.push(row.line);
        data.push(row.data);
    }
    Ok(Output::new(lines.join("\n"), json!({ "tasks": data })))
}

/// Coleta as linhas que passam pelos filtros, com motivo/impacto derivados (D104/D109).
fn collect_rows(
    session: &Session,
    args: &TaskListArgs,
    graph: Option<&Graph>,
    filters: &ListFilters<'_>,
) -> Result<Vec<Row>> {
    let sort_impact = args.sort == Some(TaskSort::Impact);
    let mut rows = Vec::new();
    for id in session.store().list_ids()? {
        let note = session.store().read(&id)?;
        if !passes_filters(&note, filters)? {
            continue;
        }
        if sort_impact && !is_actionable(graph.and_then(|graph| graph.status(&id))) {
            continue;
        }
        let reason = if args.explain && args.blocked {
            graph.and_then(|graph| block_reason(graph, &id))
        } else {
            None
        };
        let unblocks = if sort_impact {
            Some(graph.map_or(0, |graph| impact(graph, &id)))
        } else {
            None
        };
        let (mut line, data) = render_row(&note, &id, reason.as_ref(), unblocks)?;
        if args.explain
            && let Some(unblocks) = unblocks
        {
            line.push_str("|unblocks=");
            line.push_str(&unblocks.to_string());
        }
        rows.push(Row {
            line,
            data,
            impact: unblocks.unwrap_or(0),
            created: note.frontmatter.created_at()?,
            id,
        });
    }
    Ok(rows)
}

fn validate_list_args(args: &TaskListArgs) -> Result<()> {
    if args.ready && args.blocked {
        return Err(Error::invalid_input(
            "`--ready` e `--blocked` são mutuamente exclusivos",
        ));
    }
    if args.explain && !args.blocked && args.sort != Some(TaskSort::Impact) {
        return Err(Error::invalid_input(
            "`--explain` exige `--blocked` ou `--sort impact`",
        ));
    }
    Ok(())
}

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

fn show_one(
    store: &Store<'_>,
    graph: &Graph,
    id: &str,
    mode: ShowMode,
) -> Result<(String, serde_json::Value)> {
    let note = store.read(id)?;
    let statement = note.frontmatter.statement().unwrap_or_default().to_string();
    let scope = note.frontmatter.scope()?;
    let status = note.frontmatter.status()?;
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
    let mut lines = vec![format!("{id}|{statement}")];
    lines.extend(context_lines(id, &context));
    if !ids.is_empty() {
        lines.push(format!("historico: {}", ids.join(" -> ")));
    }
    let data = json!({
        "id": id,
        "statement": statement,
        "scope": scope.map(Scope::as_str),
        "status": status.as_str(),
        "body": note.body,
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
    });
    Ok((lines.join("\n"), data))
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
