//! Auto-drain ocioso (E11-T03): no fim da invocação, drena um lote se `mode = lazy`.
//!
//! Roda **depois** de a saída do comando ser emitida — nunca atrasa a resposta —, é limitado a
//! **um lote** (`embeddings.batch`) e é *best-effort*: falha não altera o código de saída nem os
//! `warnings[]` (portanto não interage com `strict`). Comandos de `maintenance` ficam de fora,
//! pois o `--drain` explícito já é o caminho manual. O `mode = manual` desliga este caminho.

use knudge_core::embeddings::EmbeddingMode;
use knudge_core::ports::Env;

use crate::cli::{Cli, Command};
use crate::session::Session;

use super::embedder;

/// Drena um lote pendente quando o modo efetivo é `lazy`.
pub fn maybe_drain(cli: &Cli) {
    if matches!(
        cli.command,
        Some(Command::Maintenance { .. } | Command::Doctor(_))
    ) {
        return;
    }
    let Ok(session) = Session::open() else {
        return;
    };
    // Escape hatch: testes e ambientes controlados desligam o auto-drain.
    if session.env().var("KNUDGE_NO_IDLE").is_some() {
        return;
    }
    let mode = session
        .config()
        .get_str("embeddings.mode")
        .unwrap_or("lazy");
    if EmbeddingMode::parse(mode).map_or(true, |mode| !mode.drains_on_idle()) {
        return;
    }
    match embedder::drain_once(&session) {
        Ok(Some(outcome)) if outcome.indexed > 0 => {
            tracing::debug!(
                indexed = outcome.indexed,
                pending = outcome.pending,
                "auto-drain ocioso"
            );
        }
        Ok(_ignored) => {}
        Err(err) => tracing::debug!(error = %err, "auto-drain ocioso ignorado"),
    }
}
