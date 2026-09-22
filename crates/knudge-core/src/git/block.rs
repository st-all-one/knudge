//! Inserção/substituição idempotente de blocos delimitados por marcadores (D60).
//!
//! Usado por `.gitattributes` (E04-T06) e `AGENTS.md` (E04-T04). Os marcadores casam a **linha
//! inteira** (após `trim`), então reexecutar a operação não duplica nem reescreve o bloco.

use std::path::Path;

use crate::ports::Fs;
use crate::{Error, Result};

/// Lê um arquivo como texto, devolvendo string vazia se não existir.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` em falha de leitura ou UTF-8 inválido.
pub fn read_text(fs: &dyn Fs, path: &Path) -> Result<String> {
    if !fs.exists(path) {
        return Ok(String::new());
    }
    String::from_utf8(fs.read(path)?)
        .map_err(|_| Error::config(format!("arquivo não é UTF-8: {}", path.display())))
}

/// Localiza a linha do marcador e devolve `(início_da_linha, após_o_fim_de_linha)`.
fn find_marker(text: &str, marker: &str) -> Option<(usize, usize)> {
    let mut offset: usize = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset = offset.saturating_add(line.len());
        if line.trim_end_matches(['\r', '\n']).trim() == marker {
            return Some((start, offset));
        }
    }
    None
}

/// Substitui o bloco entre `begin` e `end` pelo texto dado, ou o insere/remove.
///
/// - `block = Some(texto)` garante que o bloco exista (substitui o antigo se houver).
/// - `block = None` remove o bloco, se existir.
///
/// O texto é idempotente: aplicar duas vezes produz o mesmo resultado.
#[must_use]
pub fn upsert(text: &str, begin: &str, end: &str, block: Option<&str>) -> String {
    let begin_pos = find_marker(text, begin);
    let end_pos = find_marker(text, end);

    if let Some((begin_start, _)) = begin_pos {
        let end_after = match end_pos {
            Some((end_start, end_after)) if end_start >= begin_start => Some(end_after),
            _ => Some(text.len()),
        };
        if let Some(end_after) = end_after {
            let mut out =
                String::with_capacity(text.len().saturating_add(block.map_or(0, str::len)));
            out.push_str(&text[..begin_start]);
            if let Some(block) = block {
                out.push_str(block);
            }
            out.push_str(&text[end_after..]);
            return tidy(&out);
        }
    }

    match block {
        None => text.to_string(),
        Some(block) => {
            let mut out = text.to_string();
            if !out.is_empty() {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                if !out.ends_with("\n\n") {
                    out.push('\n');
                }
            }
            out.push_str(block);
            tidy(&out)
        }
    }
}

/// Garante exatamente uma `\n` final e no máximo uma linha em branco entre blocos.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(1));
    let mut pending_newlines: usize = 0;
    for ch in text.chars() {
        if ch == '\n' {
            pending_newlines = pending_newlines.saturating_add(1);
            if pending_newlines <= 2 {
                out.push(ch);
            }
        } else {
            pending_newlines = 0;
            out.push(ch);
        }
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEGIN: &str = "# begin";
    const END: &str = "# end";
    const BLOCK: &str = "# begin\na = 1\n# end\n";

    #[test]
    fn inserts_once_and_is_idempotent() {
        let once = upsert("user = 1\n", BEGIN, END, Some(BLOCK));
        assert!(once.contains("a = 1"));
        let twice = upsert(&once, BEGIN, END, Some(BLOCK));
        assert_eq!(once, twice);
    }

    #[test]
    fn replaces_existing_block() {
        let original = "x = 1\n# begin\na = 0\n# end\ny = 2\n";
        let updated = upsert(original, BEGIN, END, Some(BLOCK));
        assert!(updated.contains("a = 1"));
        assert!(!updated.contains("a = 0"));
        assert!(updated.contains("x = 1"));
        assert!(updated.contains("y = 2"));
    }

    #[test]
    fn removes_block() {
        let original = "x = 1\n# begin\na = 1\n# end\ny = 2\n";
        let removed = upsert(original, BEGIN, END, None);
        assert!(!removed.contains("a = 1"));
        assert!(removed.contains("x = 1"));
        assert!(removed.contains("y = 2"));
    }

    #[test]
    fn begin_without_end_replaces_to_eof() {
        let original = "x = 1\n# begin\na = 0\n";
        let updated = upsert(original, BEGIN, END, Some(BLOCK));
        assert!(updated.contains("x = 1"));
        assert!(updated.contains("a = 1"));
        assert!(!updated.contains("a = 0"));
    }
}
