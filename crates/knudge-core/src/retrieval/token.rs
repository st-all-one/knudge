//! Tokenização ASCII explícita (D36).
//!
//! A tokenização replica `\w` **restrito a ASCII**: só `[a-z0-9_]` forma termos. Qualquer byte
//! `>= 0x80` (inclusive acentos) é separador, então `café` vira `caf`. Isso é **documentado**
//! (D36) — não tentamos normalizar diacríticos na busca, só no [`crate::schema::body`].
//!
//! Tokens são emprestados quando já estão em minúsculas (`Cow`), evitando alocação no caminho
//! quente do `recall` (R15).

use std::borrow::Cow;
use std::collections::BTreeSet;

/// `true` para os bytes que formam termo (`[A-Za-z0-9_]`).
#[must_use]
pub const fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Tokeniza `input` em termos minúsculos.
#[must_use]
pub fn tokenize(input: &str) -> Vec<Cow<'_, str>> {
    let bytes = input.as_bytes();
    let mut out: Vec<Cow<'_, str>> = Vec::new();
    let _reserved = out.try_reserve(bytes.len() / 8);
    let mut start: Option<usize> = None;
    for (index, byte) in bytes.iter().enumerate() {
        if is_word(*byte) {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(begin) = start.take() {
            push_token(input, begin, index, &mut out);
        }
    }
    if let Some(begin) = start {
        push_token(input, begin, bytes.len(), &mut out);
    }
    out
}

/// Termos únicos da consulta, preservando a ordem de primeira aparição.
#[must_use]
pub fn query_terms(input: &str) -> Vec<Cow<'_, str>> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<Cow<'_, str>> = Vec::new();
    for token in tokenize(input) {
        if seen.insert(token.to_string()) {
            out.push(token);
        }
    }
    out
}

fn push_token<'a>(input: &'a str, begin: usize, end: usize, out: &mut Vec<Cow<'a, str>>) {
    let Some(slice) = input.get(begin..end) else {
        return;
    };
    if slice.bytes().any(|byte| byte.is_ascii_uppercase()) {
        out.push(Cow::Owned(slice.to_ascii_lowercase()));
    } else {
        out.push(Cow::Borrowed(slice));
    }
}
