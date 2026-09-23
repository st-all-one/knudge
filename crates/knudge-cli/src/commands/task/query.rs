//! Listagem e exibição de tarefas (`kd task list`/`show`) (E12-T01).

use std::collections::BTreeMap;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::block_reason;
use knudge_core::schema::Scope;
use knudge_core::task::{impact, is_actionable, ownership};
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
    let filters = ListFilters::resolve(session, args, graph.as_ref())?;
    let owners = if filters.owner.is_some() {
        owners_of(session)?
    } else {
        BTreeMap::new()
    };
    let mut rows = collect_rows(session, args, graph.as_ref(), &filters, &owners)?;
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
    owners: &BTreeMap<String, Option<String>>,
) -> Result<Vec<Row>> {
    let sort_impact = args.sort == Some(TaskSort::Impact);
    let mut rows = Vec::new();
    for id in session.store().list_ids()? {
        let note = session.store().read(&id)?;
        if !passes_filters(&note, filters, owners)? {
            continue;
        }
        if sort_impact && !is_actionable(graph.and_then(|graph| graph.status(&id))) {
            continue;
        }
        let owner = owners.get(&id).and_then(Option::as_deref);
        let reason = if args.explain && args.blocked {
            graph.and_then(|graph| block_reason(graph, &id, session.now_ms()))
        } else {
            None
        };
        let unblocks = if sort_impact {
            Some(graph.map_or(0, |graph| impact(graph, &id)))
        } else {
            None
        };
        let (mut line, data) = render_row(&note, &id, reason.as_ref(), owner, unblocks)?;
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

/// Dono derivado (`claim`/`release`) por id (D114).
fn owners_of(session: &Session) -> Result<BTreeMap<String, Option<String>>> {
    let (events, _warnings) = session.events().read_all()?;
    let mut owners = BTreeMap::new();
    for id in session.store().list_ids()? {
        let _ignored = owners.insert(id.clone(), ownership(&events, &id));
    }
    Ok(owners)
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
