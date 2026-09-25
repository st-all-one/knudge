//! Snippet do corpo e detecção de match no corpo (D161).
//!
//! Funções puras, sem índice: usam a tokenização ASCII do BM25 (D36) e devolvem um trecho
//! determinístico em torno do primeiro termo de conteúdo casado.

use crate::retrieval::token::content_terms;

/// `true` se algum termo de conteúdo da consulta aparece no corpo (case-insensitive ASCII).
#[must_use]
pub fn body_matches(body: &str, query: &str) -> bool {
    body_snippet(body, query, 0).is_some()
}

/// Trecho do corpo em torno do primeiro termo casado, com até `max_chars` caracteres.
///
/// `None` se a consulta não tem termos de conteúdo, o corpo é vazio ou não há match. O trecho é
/// delimitado por `…` quando cortado. Determinístico.
#[must_use]
pub fn body_snippet(body: &str, query: &str, max_chars: usize) -> Option<String> {
    let terms = content_terms(query);
    if terms.is_empty() || body.trim().is_empty() {
        return None;
    }
    let lower = body.to_ascii_lowercase();
    let match_byte = terms
        .iter()
        .filter_map(|term| lower.find(term.as_ref()))
        .min()?;
    Some(window(body, match_byte, max_chars))
}

/// Janela de até `max_chars` caracteres em torno do byte `match_byte` (char-safe).
#[allow(
    clippy::arithmetic_side_effects,
    reason = "divisão por constante 2 e soma com `saturating_add` em domínio limitado"
)]
fn window(body: &str, match_byte: usize, max_chars: usize) -> String {
    let chars: Vec<char> = body.chars().collect();
    if max_chars == 0 || chars.len() <= max_chars {
        return body.trim().to_string();
    }
    let match_char = body
        .char_indices()
        .take_while(|(index, _)| *index < match_byte)
        .count();
    let start = match_char.saturating_sub(max_chars / 2);
    let end = start.saturating_add(max_chars).min(chars.len());
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    if let Some(slice) = chars.get(start..end) {
        let text: String = slice.iter().collect();
        out.push_str(text.trim());
    }
    if end < chars.len() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_terms_in_body_ignoring_case() {
        assert!(body_matches("usa PostgreSQL no primário", "postgresql"));
        assert!(!body_matches("usa PostgreSQL", "sqlite"));
        assert!(!body_matches("", "postgres"));
    }

    #[test]
    fn snippet_is_none_without_terms_or_match() {
        assert_eq!(body_snippet("corpo qualquer", "de para", 20), None);
        assert_eq!(body_snippet("corpo qualquer", "ausente", 20), None);
        assert_eq!(body_snippet("", "algo", 20), None);
    }

    #[test]
    fn snippet_centers_on_match_and_marks_truncation() {
        let body = "a".repeat(200) + " alvo " + &"b".repeat(200);
        let Some(snippet) = body_snippet(&body, "alvo", 40) else {
            return;
        };
        assert!(snippet.contains("alvo"));
        assert!(snippet.starts_with('…'));
        assert!(snippet.ends_with('…'));
        assert!(snippet.chars().count() <= 42);
    }

    #[test]
    fn snippet_returns_whole_short_body() {
        assert_eq!(
            body_snippet("corpo curto alvo", "alvo", 80).as_deref(),
            Some("corpo curto alvo")
        );
    }
}
