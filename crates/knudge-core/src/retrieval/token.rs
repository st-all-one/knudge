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

/// Palavras funcionais (PT+EN) descartadas do canal lexical (D122).
///
/// **Ordenadas** para busca binária. São termos de altíssima frequência: no `ask` criavam
/// votos lexicais espúrios (ex.: `de` casando quase todo o corpus) que afogavam o canal
/// vetorial — um sinônimo puro subia no ranking por causa de `de`, não da semântica.
pub const STOPWORDS: &[&str] = &[
    "a", "and", "ao", "aos", "are", "as", "at", "be", "by", "com", "como", "da", "das", "de", "do",
    "dos", "e", "em", "essa", "esse", "esta", "este", "foi", "for", "from", "how", "in", "is",
    "isso", "isto", "it", "mais", "mas", "mesmo", "na", "nas", "no", "nos", "nossa", "nosso",
    "num", "o", "of", "on", "or", "os", "ou", "para", "pela", "pelo", "por", "qual", "que", "quem",
    "se", "sem", "ser", "seu", "sua", "tem", "that", "the", "this", "to", "um", "uma", "was",
    "what", "when", "where", "which", "who", "will", "with",
];

/// `true` se o termo é palavra funcional (D122).
#[must_use]
pub fn is_stopword(term: &str) -> bool {
    STOPWORDS.binary_search(&term).is_ok()
}

/// Termos de **conteúdo** da consulta: sem stopwords e sem fragmentos de 1 caractere (D122).
///
/// O tokenizador ASCII (D36) quebra palavras acentuadas em fragmentos (`são` → `s`,`o`), então
/// descartar tokens de 1 caractere remove ruído que casaria quase todo o corpus.
#[must_use]
pub fn content_terms(input: &str) -> Vec<Cow<'_, str>> {
    query_terms(input)
        .into_iter()
        .filter(|term| term.chars().count() >= 2 && !is_stopword(term))
        .collect()
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
