//! Lexer de linhas do subconjunto TOON.
//!
//! Quebra o documento em linhas significativas (sem comentário, sem blank), medindo a
//! indentação em espaços. Tabs e indentação ímpar são erros.

#![allow(
    clippy::arithmetic_side_effects,
    reason = "numeração de linha (1-based) com limite conhecido"
)]

use std::borrow::Cow;

use super::fail;
use crate::Result;

/// Linha significativa: indentação (em espaços) e conteúdo sem comentário.
pub(crate) struct Line<'a> {
    /// Número de espaços à esquerda.
    pub(crate) indent: usize,
    /// Conteúdo já sem indentação e sem comentário.
    pub(crate) text: Cow<'a, str>,
}

/// Divide `src` em linhas significativas, validando indentação.
pub(crate) fn split_lines(src: &str) -> Result<Vec<Line<'_>>> {
    let mut out = Vec::with_capacity(src.lines().count());
    for (number, raw) in src.lines().enumerate() {
        if raw.contains('\t') {
            return Err(fail(format!("tab na linha {}: use espaços", number + 1)));
        }
        let stripped = strip_comment(raw);
        let indent = stripped
            .len()
            .saturating_sub(stripped.trim_start_matches(' ').len());
        let text = match stripped {
            Cow::Borrowed(borrowed) => Cow::Borrowed(borrowed.trim_matches(' ')),
            Cow::Owned(owned) => Cow::Owned(owned.trim_matches(' ').to_string()),
        };
        if text.is_empty() {
            continue;
        }
        if !indent.is_multiple_of(2) {
            return Err(fail(format!(
                "indentação ímpar na linha {} (use múltiplos de 2)",
                number + 1
            )));
        }
        out.push(Line { indent, text });
    }
    Ok(out)
}

/// Remove um comentário `#` iniciado fora de aspas (precedido por espaço ou no começo).
///
/// Devolve `Cow::Borrowed` quando não há `#` (o caso comum), evitando alocar por linha (O4.4).
fn strip_comment(raw: &str) -> Cow<'_, str> {
    if !raw.contains('#') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut in_quotes = false;
    let mut escaped = false;
    let mut previous_space = true;
    for ch in raw.chars() {
        if in_quotes {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            previous_space = false;
            continue;
        }
        if ch == '"' {
            in_quotes = true;
            out.push(ch);
            previous_space = false;
            continue;
        }
        if ch == '#' && previous_space {
            break;
        }
        previous_space = ch.is_whitespace();
        out.push(ch);
    }
    Cow::Owned(out)
}
