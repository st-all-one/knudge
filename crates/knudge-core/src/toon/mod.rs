//! Escopo `toon`: parser/emissor do frontmatter TOON (D74/D75).
//!
//! A gramática completa está em `TOON.md`. Aqui ficam o parser ([`parse`]), o emissor
//! ([`emit`]) e a separação frontmatter/corpo ([`split_frontmatter`]).

pub mod emit;
pub mod flow;
pub mod lex;
pub mod parse;

#[cfg(test)]
mod tests;

pub use emit::emit;
pub use parse::parse;

use crate::{Error, Result};

/// Linha delimitadora do frontmatter.
pub const FENCE: &str = "---";

/// Erro de schema para falhas de parsing TOON.
pub(crate) fn fail(message: impl Into<String>) -> Error {
    Error::schema(message.into())
}

/// Converte `\r\n`/`\r` em `\n` (D12).
#[must_use]
pub fn normalize_newlines(src: &str) -> String {
    src.replace("\r\n", "\n").replace('\r', "\n")
}

/// Separa o frontmatter do corpo.
///
/// Sem `---` inicial, o frontmatter é vazio e o corpo é o documento inteiro. Um `---` inicial
/// sem fechamento é erro (nota malformada).
pub fn split_frontmatter(src: &str) -> Result<(String, String)> {
    let normalized = normalize_newlines(src);
    let Some(rest) = normalized.strip_prefix("---\n") else {
        return Ok((String::new(), normalized));
    };
    let mut frontmatter = String::new();
    let mut body = String::new();
    let mut closed = false;
    for line in rest.split_inclusive('\n') {
        if !closed && line.trim_end_matches('\n') == FENCE {
            closed = true;
            continue;
        }
        if closed {
            body.push_str(line);
        } else {
            frontmatter.push_str(line);
        }
    }
    if !closed {
        return Err(Error::schema("frontmatter sem `---` de fechamento"));
    }
    Ok((frontmatter, body))
}

/// Lê `schema_version` do frontmatter, se presente e válido.
#[must_use]
pub fn detect_version(frontmatter: &str) -> Option<u32> {
    let value = parse(frontmatter).ok()?;
    let version = value.as_map()?.get("schema_version")?.as_int()?;
    u32::try_from(version).ok()
}
