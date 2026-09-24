//! `.gitattributes`: regras explícitas para todos os arquivos do `.knudge/` (D31, E04-T06).
//!
//! O bloco é gerenciado (marcadores) e idempotente. Em `persist_in_project = false` ele é
//! removido. As regras cobrem: notas (`notas/**`), configuração (`config.toml`,
//! `templates.toml`, `validators.toml`), log de eventos (`eventos/events*.jsonl`) e o derivado
//! descartável (`.idx/`, `cache/`, `.locks/`). `eol=lf` mantém `body_hash`/id estáveis entre
//! plataformas; `merge=union` só no log append-only (o `id` do evento torna o union seguro —
//! D26/D28/D31); o derivado nunca é auto-mergeado.

use std::path::Path;

use crate::Result;
use crate::ports::Fs;

use super::block::{read_text, upsert};
use super::persistence::Persistence;

/// Início do bloco gerenciado.
pub const MARKER_BEGIN: &str = "# knudge:start";
/// Fim do bloco gerenciado.
pub const MARKER_END: &str = "# knudge:end";
/// Regra de merge para os segmentos do log (append-only; D26/D28/D31).
pub const UNION_LINE: &str = "/.knudge/eventos/events*.jsonl text eol=lf merge=union";
/// Regra de merge do cache vetorial versionado (chave `(body_hash, model)`; D148/D153).
pub const CACHE_UNION_LINE: &str = "/.knudge/emb_cache.jsonl text eol=lf merge=union";
/// Nome do arquivo.
pub const FILE: &str = ".gitattributes";

/// Regras gerenciadas, na ordem de emissão (linha vazia separa os grupos).
pub const RULES: &[&str] = &[
    "# Notas canônicas: LF mantém `body_hash`/id estáveis (D06/D95).",
    "/.knudge/notas/** text eol=lf",
    "",
    "# Configuração do projeto: texto normalizado.",
    "/.knudge/config.toml text eol=lf",
    "/.knudge/templates.toml text eol=lf",
    "/.knudge/validators.toml text eol=lf",
    "",
    "# Log de eventos: append-only; o `id` do conteúdo torna o union seguro (D26/D28/D31).",
    UNION_LINE,
    "",
    "# Cache vetorial versionado: chave `(body_hash, model)`; union + dedup no loader (D148/D153).",
    CACHE_UNION_LINE,
    "",
    "# Derivado e runtime (reconstruíveis, nunca versionados): sem diff/merge automático.",
    "/.knudge/.idx/** binary linguist-generated",
    "/.knudge/cache/** binary linguist-generated",
    "/.knudge/.locks/** binary linguist-generated",
    "/.knudge/**/*.tmp binary linguist-generated",
];

/// Garante (ou remove) o bloco gerenciado. Devolve `true` se mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de escrita.
pub fn apply(fs: &dyn Fs, root: &Path, persistence: Persistence) -> Result<bool> {
    let path = root.join(FILE);
    let original = read_text(fs, &path)?;
    let replacement = if persistence.is_versioned() {
        Some(render())
    } else {
        None
    };
    let updated = upsert(&original, MARKER_BEGIN, MARKER_END, replacement.as_deref());
    if updated == original {
        return Ok(false);
    }
    fs.write_atomic(&path, updated.as_bytes())?;
    Ok(true)
}

/// Renderiza o bloco gerenciado (marcadores + [`RULES`]).
#[must_use]
pub fn render() -> String {
    let mut out = String::from(MARKER_BEGIN);
    out.push('\n');
    for rule in RULES {
        out.push_str(rule);
        out.push('\n');
    }
    out.push_str(MARKER_END);
    out.push('\n');
    out
}
