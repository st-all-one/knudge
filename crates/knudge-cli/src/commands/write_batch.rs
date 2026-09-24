//! `kd write --batch` — lote de rascunhos JSONL (K4/D110).

use std::io::Read;
use std::path::Path;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::jsonl;
use knudge_core::write::{BatchMode, batch_jsonl};
use serde_json::json;

use crate::commands::input;
use crate::output::Output;
use crate::session::Session;

/// Aplica um lote de rascunhos JSONL (`-` lê stdin) e devolve `action|id` por item.
///
/// # Errors
/// `ErrorKind::InvalidInput` quando o lote excede `write.batch_max`.
pub fn batch_note(session: &Session, source: &str, mode: BatchMode) -> Result<Output> {
    let text = read_source(session, source)?;
    apply_text(session, &text, mode)
}

/// `kd write --params '{json}'`: item único de um lote (D147).
///
/// # Errors
/// `ErrorKind::InvalidInput` para JSON inválido; propaga I/O e schema.
pub fn params_note(session: &Session, params: &str, mode: BatchMode) -> Result<Output> {
    let text = if params == "-" {
        input::read_stdin()?
    } else {
        params.to_string()
    };
    let line = jsonl::encode(&jsonl::decode(&text)?)?;
    apply_text(session, &line, mode)
}

/// Aplica um lote já em texto (uma linha por item).
fn apply_text(session: &Session, text: &str, mode: BatchMode) -> Result<Output> {
    let max = session
        .config()
        .get_int("write.batch_max")
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(100);
    let count = jsonl::lines(text).count();
    if count > max {
        return Err(Error::invalid_input(format!(
            "lote com {count} itens excede write.batch_max={max}"
        )));
    }
    let ctx = session.write_context()?;
    let out = batch_jsonl(&ctx, text, &session.thresholds()?, mode);
    let dry_run = mode == BatchMode::DryRun;
    let prefix = if dry_run { "dry-run|" } else { "" };
    let lines = out
        .items
        .iter()
        .map(|item| match item.revision {
            Some(rev) => format!("{prefix}{}|{}|r{rev}", item.action.as_str(), item.id),
            None => format!("{prefix}{}|{}", item.action.as_str(), item.id),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "dry_run": dry_run,
        "items": out.items.iter().map(|item| json!({
            "action": item.action.as_str(),
            "id": item.id,
            "revision": item.revision,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(lines, data).with_warnings(out.warnings))
}

fn read_source(session: &Session, source: &str) -> Result<String> {
    if source == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| Error::io("stdin", error))?;
        return Ok(buffer);
    }
    let bytes = session.fs_dyn().read(Path::new(source))?;
    String::from_utf8(bytes).map_err(|_| Error::config(format!("lote não é UTF-8: {source}")))
}
