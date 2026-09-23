//! Filtros e renderização de `kd task list` (E12-T01, D104/D109).

use std::collections::{BTreeMap, BTreeSet};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::ports::Env;
use knudge_core::retrieval::{BlockReason, compute_views_at};
use knudge_core::schema::{NoteType, Scope, Status};
use knudge_core::store::Note;
use knudge_core::task::{is_task, parent_of};
use knudge_core::time::Timestamp;
use serde_json::json;

use crate::cli::TaskListArgs;
use crate::session::Session;

use super::AGENT_ENV;

/// Filtros resolvidos de `kd task list` (evita excesso de parâmetros).
pub(super) struct ListFilters<'a> {
    scope: Option<Scope>,
    status: Option<Status>,
    kind: Option<NoteType>,
    parent: Option<&'a str>,
    pub(super) owner: Option<String>,
    allowed: Option<BTreeSet<String>>,
}

impl<'a> ListFilters<'a> {
    pub(super) fn resolve(
        session: &Session,
        args: &'a TaskListArgs,
        graph: Option<&Graph>,
    ) -> Result<Self> {
        let owner = if args.mine {
            Some(session.env().var(AGENT_ENV).ok_or_else(|| {
                Error::invalid_input(format!("`--mine` exige a variável {AGENT_ENV}"))
            })?)
        } else {
            args.owner.clone()
        };
        Ok(Self {
            scope: args.scope.as_deref().map(str::parse::<Scope>).transpose()?,
            status: args
                .status
                .as_deref()
                .map(str::parse::<Status>)
                .transpose()?,
            kind: args
                .kind
                .as_deref()
                .map(str::parse::<NoteType>)
                .transpose()?,
            parent: args.parent.as_deref(),
            owner,
            allowed: allowed_ids(session, args, graph),
        })
    }
}

/// Ids permitidos pela view pedida (`--ready`/`--blocked`); `None` sem view.
fn allowed_ids(
    session: &Session,
    args: &TaskListArgs,
    graph: Option<&Graph>,
) -> Option<BTreeSet<String>> {
    let graph = graph?;
    if !args.ready && !args.blocked {
        return None;
    }
    let views = compute_views_at(graph, session.now_ms());
    Some(if args.blocked {
        views.blocked
    } else {
        views.ready
    })
}

/// Linha de `kd task list` com as chaves de ordenação derivadas (D109).
pub(super) struct Row {
    pub(super) line: String,
    pub(super) data: serde_json::Value,
    pub(super) impact: usize,
    pub(super) created: i64,
    pub(super) id: String,
}

/// `true` se a nota passa por todos os filtros de `kd task list`.
pub(super) fn passes_filters(
    note: &Note,
    filters: &ListFilters<'_>,
    owners: &BTreeMap<String, Option<String>>,
) -> Result<bool> {
    if !is_task(note) {
        return Ok(false);
    }
    let id = note.id()?;
    if let Some(scope) = filters.scope
        && note.frontmatter.scope()? != Some(scope)
    {
        return Ok(false);
    }
    if let Some(status) = filters.status
        && note.frontmatter.status()? != status
    {
        return Ok(false);
    }
    if let Some(kind) = filters.kind
        && note.frontmatter.note_type()? != kind
    {
        return Ok(false);
    }
    if let Some(parent) = filters.parent
        && parent_of(note).as_deref() != Some(parent)
    {
        return Ok(false);
    }
    if let Some(owner) = &filters.owner
        && owners.get(id).and_then(Option::as_deref) != Some(owner.as_str())
    {
        return Ok(false);
    }
    if let Some(allowed) = &filters.allowed
        && !allowed.contains(id)
    {
        return Ok(false);
    }
    Ok(true)
}

/// Linha de pipe + objeto JSON de uma tarefa (o `impact` só vem com `--sort impact`).
pub(super) fn render_row(
    note: &Note,
    id: &str,
    reason: Option<&BlockReason>,
    owner: Option<&str>,
    impact: Option<usize>,
) -> Result<(String, serde_json::Value)> {
    let statement = note.frontmatter.statement().unwrap_or_default().to_string();
    let scope = note.frontmatter.scope()?;
    let kind = note.frontmatter.note_type()?;
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
        "kind": kind.as_str(),
        "status": status.as_str(),
        "statement": statement,
        "parent": parent_of(note),
        "owner": owner,
        "blocked_reason": reason.map(reason_label),
        "impact": impact,
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
