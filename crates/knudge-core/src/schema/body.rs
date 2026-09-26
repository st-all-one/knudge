//! Normalização de texto e `body_hash` (D06).
//!
//! `normalize` = **NFC + trim + colapso de whitespace** para um único espaço ASCII. A mesma
//! normalização alimenta o `id` (D95) e o `body_hash`, garantindo que variações puramente
//! tipográficas produzam o mesmo endereço.

use unicode_normalization::UnicodeNormalization;

use crate::schema::hash;

/// Compacta `input` em NFC, sem espaços nas pontas e com whitespace interno colapsado.
#[must_use]
pub fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    normalize_into(input, &mut out);
    out
}

/// Anexa a forma normalizada de `input` a `out` (mesma regra de [`normalize`]).
///
/// ASCII é identidade em NFC, então o caminho quente evita a coleta intermediária (O4.1).
pub fn normalize_into(input: &str, out: &mut String) {
    if input.is_ascii() {
        push_collapsed(input, out);
    } else {
        let nfc: String = input.nfc().collect();
        push_collapsed(&nfc, out);
    }
}

fn push_collapsed(input: &str, out: &mut String) {
    let mut pending_space = false;
    let mut written = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            pending_space = written;
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
            written = true;
        }
    }
}

/// Hash do corpo: `hex8(SHA-256(normalize(statement) + LF + normalize(body)))` (D06).
#[must_use]
pub fn body_hash(statement: &str, body: &str) -> String {
    let head = normalize(statement);
    let tail = normalize(body);
    hash::hex8_value(hash::short_hash_parts(&[
        head.as_bytes(),
        b"\n",
        tail.as_bytes(),
    ]))
}
