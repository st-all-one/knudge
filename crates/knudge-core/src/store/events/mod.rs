//! Log de eventos append-only (E03-T04/T09).
//!
//! Um evento é uma linha JSON em `eventos/events.jsonl`. A leitura é **tolerante** (linha
//! malformada → skip + warning) e faz **dedup on-read** por `id` (D26), o que torna o arquivo
//! idempotente sob `merge=union` do git (D28/D31). Quando o segmento ativo passa de
//! `max_bytes`, ele é **rotacionado** para `eventos/events-NNNN.jsonl` (R13).

mod event;
mod log;

pub use event::Event;
pub use log::{EventLog, EventRead};

/// Prefixo dos ids de evento.
pub const EVENT_PREFIX: &str = "evt";
