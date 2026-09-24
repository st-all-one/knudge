//! `kd knowledge digest` — digere o conteúdo num vetor (ex-`maintenance index`, D145).

use knudge_core::Result;
use serde_json::json;

use crate::cli::KnowledgeDigestArgs;
use crate::commands::embedder;
use crate::output::Output;
use crate::session::Session;

/// `kd knowledge digest` — status/drain da fila de embeddings (um lote por `--drain`).
///
/// # Errors
/// Propaga erros de leitura do índice e de execução do provedor.
pub fn run(session: &Session, args: &KnowledgeDigestArgs) -> Result<Output> {
    if args.drain {
        return drain_queue(session);
    }
    let pending = embedder::pending(session)?;
    let data = json!({ "pending": pending });
    Ok(Output::new(format!("pending: {pending}"), data))
}

fn drain_queue(session: &Session) -> Result<Output> {
    let Some(outcome) = embedder::drain_once(session)? else {
        return Ok(Output::new(
            "embeddings desligado (provider = none)",
            json!({ "enabled": false, "indexed": 0, "pending": 0 }),
        ));
    };
    let text = format!(
        "indexed={} pending={} stale={} cache_hits={}",
        outcome.indexed, outcome.pending, outcome.stale, outcome.cache_hits
    );
    let data = json!({
        "enabled": true,
        "indexed": outcome.indexed,
        "pending": outcome.pending,
        "stale": outcome.stale,
        "cache_hits": outcome.cache_hits,
    });
    Ok(Output::new(text, data).with_warnings(outcome.warnings))
}
