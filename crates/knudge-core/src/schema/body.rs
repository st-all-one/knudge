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
    let nfc: String = input.nfc().collect();
    let mut out = String::with_capacity(nfc.len());
    let mut pending_space = false;
    for ch in nfc.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
        }
    }
    out
}

/// Hash do corpo: `hex8(SHA-256(normalize(statement) + LF + normalize(body)))` (D06).
#[must_use]
pub fn body_hash(statement: &str, body: &str) -> String {
    let mut data = normalize(statement);
    data.push('\n');
    data.push_str(&normalize(body));
    hash::hex8(data.as_bytes())
}
