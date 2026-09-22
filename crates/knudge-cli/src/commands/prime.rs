//! `kd prime` — protocolo estático, byte-idêntico por versão (D57, E12-T01).

use knudge_core::schema::{CANONICAL_KEYS, NoteType};
use serde_json::json;

use crate::output::Output;

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocolo estático (byte-idêntico por versão; cacheável).
const PRIME_TEXT: &str = "\
knudge (kd) — memória otimizada para LLM
Uso: kd <comando> [opções]
Comandos: init, prime, rewind, ask, write, task, maintenance, config, forget, sync, self
Sem argumentos, kd executa `prime`.
";

/// Formato de `prime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimeFormat {
    /// Protocolo curto (byte-idêntico).
    Short,
    /// Protocolo + schema/gramática.
    Long,
}

/// Resposta de `prime` (estática; `Long` anexa schema/gramática).
#[must_use]
pub fn run(format: PrimeFormat) -> Output {
    let text = match format {
        PrimeFormat::Short => PRIME_TEXT.to_string(),
        PrimeFormat::Long => format!("{PRIME_TEXT}{}", long_section()),
    };
    let data = match format {
        PrimeFormat::Short => json!({ "protocol": text, "version": VERSION }),
        PrimeFormat::Long => json!({
            "protocol": text,
            "version": VERSION,
            "types": NoteType::ALL.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            "keys": CANONICAL_KEYS,
        }),
    };
    Output::new(text, data)
}

fn long_section() -> String {
    let types: Vec<&str> = NoteType::ALL.iter().map(|t| t.as_str()).collect();
    let keys: Vec<&str> = CANONICAL_KEYS.to_vec();
    format!(
        "\ntipos: {}\nchaves canônicas ({}): {}\n",
        types.join(", "),
        keys.len(),
        keys.join(", ")
    )
}
