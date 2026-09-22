//! `.gitattributes`: `merge=union` para o log de eventos (D31, E04-T06).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;

use super::block::{read_text, upsert};
use super::persistence::Persistence;

/// Início do bloco gerenciado.
pub const MARKER_BEGIN: &str = "# knudge:start";
/// Fim do bloco gerenciado.
pub const MARKER_END: &str = "# knudge:end";
/// Regra de merge para os segmentos do log (o `id` do evento torna o union seguro — D26/D28).
pub const UNION_LINE: &str = ".knudge/eventos/events*.jsonl merge=union";
/// Nome do arquivo.
pub const FILE: &str = ".gitattributes";

/// Garante (ou remove) o bloco de `merge=union`. Devolve `true` se mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de escrita.
pub fn apply(fs: &dyn Fs, root: &Path, persistence: Persistence) -> Result<bool> {
    let path = root.join(FILE);
    let original = read_text(fs, &path)?;
    let block = format!("{MARKER_BEGIN}\n{UNION_LINE}\n{MARKER_END}\n");
    let replacement = if persistence.is_versioned() {
        Some(block.as_str())
    } else {
        None
    };
    let updated = upsert(&original, MARKER_BEGIN, MARKER_END, replacement);
    if updated == original {
        return Ok(false);
    }
    fs.write_atomic(&path, updated.as_bytes())?;
    Ok(true)
}
