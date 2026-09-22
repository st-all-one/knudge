//! Redação de segredos no *layer* de log (R22).
//!
//! A regra é **allowlist**: só é redigido o que for segredo conhecido (valores de `[secrets]`)
//! ou o valor de chaves sensíveis conhecidas (`Authorization`, `token`, …). Corpo de nota nunca
//! é passado ao logger — a redação é a segunda linha de defesa.

use std::borrow::Cow;

/// Marcador usado no lugar do valor sensível.
pub const REDACTED: &str = "[REDACTED]";

/// Chaves cujo valor é sempre redigido.
const SENSITIVE_KEYS: &[&str] = &[
    "authorization",
    "access_token",
    "api_key",
    "apikey",
    "bearer",
    "password",
    "passwd",
    "secret",
    "token",
];

/// Redator configurável.
#[derive(Debug, Clone, Default)]
pub struct Redactor {
    /// Segredos literais (valores de `[secrets]`), do maior para o menor.
    secrets: Vec<String>,
}

impl Redactor {
    /// Cria um redator a partir dos segredos conhecidos.
    #[must_use]
    pub fn new(secrets: impl IntoIterator<Item = String>) -> Self {
        let mut secrets: Vec<String> = secrets.into_iter().filter(|s| !s.is_empty()).collect();
        secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
        secrets.dedup();
        Self { secrets }
    }

    /// Redator sem segredos literais (ainda redige chaves sensíveis).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Redige segredos literais e valores de chaves sensíveis.
    #[must_use]
    pub fn redact<'a>(&self, input: &'a str) -> Cow<'a, str> {
        let mut current: Option<String> = None;
        for secret in &self.secrets {
            let source = current.as_deref().unwrap_or(input);
            if source.contains(secret.as_str()) {
                current = Some(source.replace(secret.as_str(), REDACTED));
            }
        }

        let text = current.as_deref().unwrap_or(input);
        if let Some(redacted) = redact_sensitive_lines(text) {
            return Cow::Owned(redacted);
        }
        match current {
            Some(owned) => Cow::Owned(owned),
            None => Cow::Borrowed(input),
        }
    }
}

/// Redige, linha a linha, o valor após `chave:` ou `chave=`.
fn redact_sensitive_lines(text: &str) -> Option<String> {
    let mut changed = false;
    let mut out = String::with_capacity(text.len());
    for segment in text.split_inclusive('\n') {
        let (line, newline) = match segment.strip_suffix('\n') {
            Some(l) => (l, "\n"),
            None => (segment, ""),
        };
        match redact_line(line) {
            Some(redacted) => {
                changed = true;
                out.push_str(&redacted);
            }
            None => out.push_str(line),
        }
        out.push_str(newline);
    }
    changed.then_some(out)
}

/// Redige o restante da linha após uma chave sensível.
#[allow(
    clippy::arithmetic_side_effects,
    reason = "índices de bytes ASCII limitados pela linha"
)]
fn redact_line(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    for key in SENSITIVE_KEYS {
        let mut start = 0;
        while let Some(pos) = lower.get(start..).and_then(|haystack| haystack.find(key)) {
            let abs = start + pos;
            let after = abs + key.len();
            let before = line.as_bytes().get(abs.wrapping_sub(1)).copied();
            let boundary = abs == 0 || !is_word_byte(before);
            if boundary {
                let bytes = line.as_bytes();
                let mut i = after;
                while matches!(bytes.get(i), Some(b' ' | b'\t')) {
                    i += 1;
                }
                if matches!(bytes.get(i), Some(b'=')) {
                    let mut value_start = i.saturating_add(1);
                    while matches!(bytes.get(value_start), Some(b' ' | b'\t')) {
                        value_start += 1;
                    }
                    let mut value_end = value_start;
                    while bytes
                        .get(value_end)
                        .is_some_and(|b| !b.is_ascii_whitespace())
                    {
                        value_end += 1;
                    }
                    let mut out = String::with_capacity(line.len() + REDACTED.len());
                    out.push_str(line.get(..value_start).unwrap_or_default());
                    out.push_str(REDACTED);
                    out.push_str(line.get(value_end..).unwrap_or_default());
                    return Some(out);
                }
                if matches!(bytes.get(i), Some(b':')) {
                    let mut value_start = i.saturating_add(1);
                    while matches!(bytes.get(value_start), Some(b' ' | b'\t')) {
                        value_start += 1;
                    }
                    let mut out = String::with_capacity(line.len() + REDACTED.len());
                    out.push_str(line.get(..value_start).unwrap_or_default());
                    out.push_str(REDACTED);
                    return Some(out);
                }
            }
            start = after;
        }
    }
    None
}

/// `true` se o byte pertence a um identificador (para exigir fronteira antes da chave).
fn is_word_byte(byte: Option<u8>) -> bool {
    byte.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_secret_is_redacted() {
        let r = Redactor::new(["super-secreto-123".to_string()]);
        let out = r.redact("token é super-secreto-123 e pronto");
        assert!(!out.contains("super-secreto-123"), "vazou: {out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn authorization_header_is_redacted() {
        let r = Redactor::empty();
        let out = r.redact("Authorization: Bearer abc.def.ghi");
        assert!(!out.contains("abc.def.ghi"), "vazou: {out}");
        assert!(out.starts_with("Authorization: "));
    }

    #[test]
    fn field_style_token_is_redacted() {
        let r = Redactor::empty();
        let out = r.redact("evento token=deadbeef concluído");
        assert!(!out.contains("deadbeef"), "vazou: {out}");
    }

    #[test]
    fn clean_text_borrows_without_allocation() {
        let r = Redactor::empty();
        let out = r.redact("mensagem sem segredo");
        assert!(matches!(out, Cow::Borrowed(_)));
    }

    #[test]
    fn preserves_newlines_when_redacting() {
        let r = Redactor::new(["x123".to_string()]);
        let out = r.redact("linha1\nvalor x123\nlinha3");
        assert_eq!(out.lines().count(), 3);
        assert!(!out.contains("x123"));
    }
}
