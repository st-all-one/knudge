//! Formatação dos hits do `recall` no contrato de pipe (D39).
//!
//! `id|statement|score|why` (completo) ou `id|statement` (`--brief`). O `statement` é
//! sanitizado (sem `|` nem quebras) para manter as colunas parseáveis.

use super::RecallHit;

/// Formata um hit no contrato `id|statement|score|why` (D39).
#[must_use]
pub fn format_hit(hit: &RecallHit) -> String {
    format!(
        "{}|{}|{:.2}|{}",
        hit.id,
        sanitize(&hit.statement),
        hit.score,
        hit.why.as_str()
    )
}

/// Formata um hit no contrato reduzido `id|statement` (D39, `--brief`).
#[must_use]
pub fn format_brief(hit: &RecallHit) -> String {
    format!("{}|{}", hit.id, sanitize(&hit.statement))
}

fn sanitize(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut pending = false;
    for ch in statement.chars() {
        if ch == '|' || ch.is_whitespace() {
            pending = !out.is_empty();
        } else {
            if pending {
                out.push(' ');
                pending = false;
            }
            out.push(ch);
        }
    }
    out
}
