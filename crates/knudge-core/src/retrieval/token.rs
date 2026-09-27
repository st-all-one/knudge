//! Tokenização ASCII com fold de diacríticos (D36/D172).
//!
//! A gramática de termo continua **restrita a ASCII**: só `[a-z0-9_]` forma termos. A diferença
//! (D172) é que o texto **não-ASCII** é decomposto em NFD e as marcas combinantes são
//! descartadas, de modo que `café` e `cafe` produzam o mesmo termo `cafe`. Caracteres sem
//! decomposição ASCII (cirílico, CJK, …) seguem separadores — o fold é para diacríticos latinos.
//!
//! O `normalize` do schema **não muda** (D06/D95): `id`/`body_hash` seguem NFC. Tokens ASCII já
//! em minúsculas são emprestados (`Cow`), evitando alocação no caminho quente do `recall` (R15);
//! só entradas não-ASCII pagam o fold.

use std::borrow::Cow;
use std::collections::BTreeSet;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// `true` para os bytes que formam termo (`[A-Za-z0-9_]`).
#[must_use]
pub const fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Tokeniza `input` em termos minúsculos, dobrando diacríticos latinos (D172).
///
/// ASCII puro usa o caminho emprestado (zero alocação); não-ASCII é decomposto em NFD, as marcas
/// combinantes são removidas e o resultado é minúsculo. Determinístico e idempotente.
#[must_use]
pub fn tokenize(input: &str) -> Vec<Cow<'_, str>> {
    if input.is_ascii() {
        tokenize_ascii(input)
    } else {
        tokenize_folded(input)
    }
}

/// Caminho ASCII (D36): cada token é um empréstimo de `input`, ou um `String` só se houver
/// maiúsculas.
fn tokenize_ascii(input: &str) -> Vec<Cow<'_, str>> {
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

/// Caminho não-ASCII (D172): dobra via [`folded_chars`] e devolve tokens próprios.
fn tokenize_folded(input: &str) -> Vec<Cow<'_, str>> {
    let mut out: Vec<Cow<'_, str>> = Vec::new();
    let _reserved = out.try_reserve(input.len() / 8);
    let mut current = String::new();
    for (ch, _) in folded_chars(input) {
        if ch == ' ' {
            flush(&mut current, &mut out);
        } else {
            current.push(ch);
        }
    }
    flush(&mut current, &mut out);
    out
}

/// Itera os caracteres dobrados de `input` (ASCII minúsculo) com o byte de origem.
///
/// A decomposição NFD separa cada diacrítico; as marcas combinantes somem. Separadores viram um
/// espaço, preservando os limites de palavra (`caf e` ≠ `cafe`).
pub(crate) fn folded_chars(input: &str) -> impl Iterator<Item = (char, usize)> + '_ {
    input.char_indices().flat_map(|(byte, ch)| {
        ch.nfd().filter_map(move |folded| {
            if folded.is_ascii() {
                let out = if folded.is_ascii_alphanumeric() || folded == '_' {
                    folded.to_ascii_lowercase()
                } else {
                    ' '
                };
                Some((out, byte))
            } else if is_combining_mark(folded) {
                None
            } else {
                Some((' ', byte))
            }
        })
    })
}

/// Forma dobrada de `input` (ASCII minúsculo), com separadores preservados como espaço.
#[must_use]
pub(crate) fn fold_ascii(input: &str) -> String {
    folded_chars(input).map(|(ch, _)| ch).collect()
}

/// Byte de `input` que originou o caractere dobrado de índice `folded_index`.
#[must_use]
pub(crate) fn folded_byte_to_original(input: &str, folded_index: usize) -> Option<usize> {
    folded_chars(input)
        .enumerate()
        .find(|(index, _)| *index == folded_index)
        .map(|(_, (_, byte))| byte)
}

/// Termos únicos da consulta, preservando a ordem de primeira aparição.
#[must_use]
pub fn query_terms(input: &str) -> Vec<Cow<'_, str>> {
    let mut seen: BTreeSet<Cow<'_, str>> = BTreeSet::new();
    let mut out: Vec<Cow<'_, str>> = Vec::new();
    for token in tokenize(input) {
        // `Cow::Borrowed` clona só o `&str` (sem alocar); `Owned` aloca uma vez, como antes.
        if seen.insert(token.clone()) {
            out.push(token);
        }
    }
    out
}

/// Palavras funcionais (PT+EN) descartadas do canal lexical (D122/D173).
///
/// **Ordenadas** para busca binária. São termos de altíssima frequência: no `ask` criavam
/// votos lexicais espúrios (ex.: `de` casando quase todo o corpus) que afogavam o canal
/// vetorial — um sinônimo puro subia no ranking por causa de `de`, não da semântica. Com o fold
/// (D172), as formas acentuadas do PT (`ja`, `sao`, `tambem`…) entram dobradas (D173).
pub const STOPWORDS: &[&str] = &[
    "a", "agora", "ainda", "alem", "ali", "and", "antes", "ao", "aos", "apos", "aqui", "are", "as",
    "at", "ate", "be", "by", "cada", "com", "como", "contra", "da", "dao", "das", "de", "depois",
    "desde", "do", "dos", "durante", "e", "ela", "ele", "em", "entao", "entre", "era", "eram",
    "essa", "esse", "esta", "estao", "este", "foi", "for", "from", "ha", "how", "in", "is", "isso",
    "isto", "it", "ja", "la", "lhe", "mais", "mas", "me", "menos", "mesmo", "muito", "na", "nao",
    "nas", "no", "nos", "nossa", "nosso", "num", "nunca", "o", "of", "on", "onde", "or", "os",
    "ou", "outra", "outro", "para", "pela", "pelo", "por", "porem", "porque", "pouco", "qual",
    "quando", "que", "quem", "sao", "se", "sem", "sempre", "ser", "sera", "seu", "so", "sobre",
    "sua", "talvez", "tambem", "tao", "te", "tem", "that", "the", "this", "to", "toda", "todas",
    "todo", "todos", "um", "uma", "vai", "vao", "voce", "vou", "was", "what", "when", "where",
    "which", "who", "will", "with",
];

/// `true` se o termo é palavra funcional (D122).
#[must_use]
pub fn is_stopword(term: &str) -> bool {
    STOPWORDS.binary_search(&term).is_ok()
}

/// Termos de **conteúdo** da consulta: sem stopwords e sem fragmentos de 1 caractere (D122).
///
/// Com o fold (D172), palavras acentuadas deixam de virar fragmentos (`são` → `sao`), então o
/// descarte de 1 caractere cobre só siglas de uma letra e resíduos de separadores.
#[must_use]
pub fn content_terms(input: &str) -> Vec<Cow<'_, str>> {
    query_terms(input)
        .into_iter()
        // Tokens são ASCII por construção (D36/D172), então `len()` em bytes = `chars().count()`.
        .filter(|term| term.len() >= 2 && !is_stopword(term))
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

fn flush(current: &mut String, out: &mut Vec<Cow<'_, str>>) {
    if !current.is_empty() {
        out.push(Cow::Owned(std::mem::take(current)));
    }
}
