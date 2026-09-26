//! Auto-drain ocioso (E11-T03): no fim da invocação, drena um lote se `mode = lazy`.
//!
//! Roda **depois** de a saída do comando ser emitida — nunca atrasa a resposta —, é limitado a
//! **um lote** (`embeddings.batch`) e é *best-effort*: falha não altera o código de saída nem os
//! `warnings[]` (portanto não interage com `strict`). Comandos de manutenção/saúde, `prime` e
//! `self` ficam de fora: o `--drain` explícito já é o caminho manual e os dois últimos não têm
//! relação com a fila (E15-T03/O1.5). O `mode = manual` desliga este caminho.

use knudge_core::adapters::StdEnv;
use knudge_core::embeddings::EmbeddingMode;
use knudge_core::ports::Env;

use crate::cli::{Cli, Command};
use crate::session::Session;

use super::embedder;

/// Drena um lote pendente quando o modo efetivo é `lazy`.
pub fn maybe_drain(cli: &Cli) {
    if matches!(
        cli.command,
        Some(
            Command::Maintenance { .. }
                | Command::Doctor(_)
                | Command::Drain(_)
                | Command::Prime(_)
                | Command::SelfCmd { .. }
        )
    ) {
        return;
    }
    // Escape hatch checado **antes** de `Session::open` (E15-T03/O1.5): testes/bench não pagam
    // a resolução de projeto só para descobrir que o auto-drain está desligado.
    if StdEnv::new().var("KNUDGE_NO_IDLE").is_some() {
        return;
    }
    let Ok(session) = Session::open() else {
        return;
    };
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
