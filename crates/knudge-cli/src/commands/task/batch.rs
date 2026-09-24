//! `kd task new --batch`/`--params` — lote de tarefas (D141).

use std::path::Path;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::jsonl;
use knudge_core::task::{TaskBatchMode, TaskBatchOutput, batch_jsonl};
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::super::input;

/// Origem do lote: `--params` (objeto único) ou `--batch` (JSONL/arquivo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskInput {
    /// Objeto JSON único.
    Params,
    /// Lote JSONL (arquivo ou stdin).
    Batch,
}

/// Executa o lote (`--batch`) ou o item único (`--params`) e devolve a saída auto-suficiente.
///
/// # Errors
/// `ErrorKind::InvalidInput` quando o lote excede `task.batch_max`; propaga I/O e schema.
pub(super) fn run(
    session: &Session,
    source: &str,
    input: TaskInput,
    mode: TaskBatchMode,
) -> Result<Output> {
    let text = if input == TaskInput::Params {
        if source == "-" {
            input::read_stdin()?
        } else {
            source.to_string()
        }
    } else {
        read_source(session, source)?
    };
    let text = if input == TaskInput::Params {
        jsonl::encode(&jsonl::decode(&text)?)?
    } else {
        text
    };
    let max = session
        .config()
        .get_int("task.batch_max")
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(100);
    let count = jsonl::lines(&text).count();
    if count > max {
        return Err(Error::invalid_input(format!(
            "lote com {count} itens excede task.batch_max={max}"
        )));
    }
    let ctx = session.write_context()?;
    let out = batch_jsonl(&ctx, &text, mode, max)?;
    Ok(render(out, mode))
}

/// Uma linha `key|id|scope|status|statement` por item + `--json` com `items[]`/`keys`.
fn render(out: TaskBatchOutput, mode: TaskBatchMode) -> Output {
    let dry_run = mode == TaskBatchMode::DryRun;
    let prefix = if dry_run { "dry-run|" } else { "" };
    let lines = out
        .items
        .iter()
        .map(|item| {
            let key = item.key.as_deref().unwrap_or("-");
            format!(
                "{prefix}{key}|{}|{}|{}|{}",
                item.id,
                item.scope.as_str(),
                item.status.as_str(),
                item.statement
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "dry_run": dry_run,
        "items": out.items.iter().map(|item| json!({
            "action": item.action,
            "key": item.key,
            "id": item.id,
            "scope": item.scope.as_str(),
            "kind": item.kind.as_str(),
            "status": item.status.as_str(),
            "statement": item.statement,
            "parent": item.parent,
            "edges": item.edges.iter().map(|(kind, to)| json!({
                "kind": kind.as_str(),
                "to": to,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "keys": out.keys,
    });
    Output::new(lines, data).with_warnings(out.warnings)
}

fn read_source(session: &Session, source: &str) -> Result<String> {
    if source == "-" {
        return input::read_stdin();
    }
    let bytes = session.fs_dyn().read(Path::new(source))?;
    String::from_utf8(bytes).map_err(|_| Error::config(format!("lote não é UTF-8: {source}")))
}
