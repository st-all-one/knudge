//! Listagem de tarefas (`kd task list`) (E12-T01, D137).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::block_reason;
use knudge_core::schema::Scope;
use knudge_core::task::{impact, is_actionable};
use serde_json::json;

use crate::cli::{TaskListArgs, TaskSort};
use crate::output::Output;
use crate::session::Session;

use super::ShowMode;
use super::render::{ListFilters, Row, passes_filters, render_row};
use super::show::show_one;

/// `kd task list` (views `--ready`/`--blocked`, `--explain`, `--sort impact` e filtros).
///
/// # Errors
/// Propaga erros de leitura do store e valida a combinação de flags.
pub(super) fn list(session: &Session, args: &TaskListArgs) -> Result<Output> {
    validate_list_args(args)?;
    let needs_graph = args.ready
        || args.blocked
        || args.sort == Some(TaskSort::Impact)
        || args.full_content
        || args
            .scope
            .as_deref()
            .is_some_and(|text| text.parse::<Scope>().is_err());
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
    let separator = if args.full_content { "\n---\n" } else { "\n" };
    Ok(Output::new(lines.join(separator), json!({ "tasks": data })))
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
        let Some(note) = session.store().read_optional(&id)? else {
            continue;
        };
        if !passes_filters(&note, filters, graph)? {
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
        let (mut line, data) = if args.full_content {
            let Some(graph) = graph else {
                return Err(Error::internal("grafo ausente no `--full-content`"));
            };
            show_one(&session.store(), graph, &id, ShowMode::Plain)?
        } else {
            render_row(&note, &id, reason.as_ref(), unblocks)?
        };
        if args.explain
            && let Some(unblocks) = unblocks
        {
            line.push_str(if args.full_content {
                "\nimpacto: "
            } else {
                "|unblocks="
            });
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
    if !args.universe && !has_scope(args) {
        return Err(Error::invalid_input(
            "`task list` exige um filtro (--scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor) ou --universe",
        ));
    }
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

/// `true` se algum filtro de escopo foi pedido (`--sort`/`--explain` não contam — D144).
fn has_scope(args: &TaskListArgs) -> bool {
    args.scope.is_some()
        || args.status.is_some()
        || args.kind.is_some()
        || args.parent.is_some()
        || args.ready
        || args.blocked
        || !args.tag.is_empty()
        || !args.anchor.is_empty()
}
