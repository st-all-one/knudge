//! Hash curto do knudge (D95).
//!
//! Um único algoritmo para `id` e `body_hash`: **SHA-256 truncado aos 4 primeiros bytes**
//! (`u32` big-endian). O `body_hash` sai como `hex8`; o `id` como `base36(8)`.

use sha2::{Digest, Sha256};

/// Primeiros 4 bytes big-endian de `SHA-256(data)`, como `u32`.
#[must_use]
pub fn short_hash(data: &[u8]) -> u32 {
    short_hash_parts(&[data])
}

/// Como [`short_hash`], mas sobre várias partes **sem concatená-las** (O4.2/O4.3).
#[must_use]
pub fn short_hash_parts(parts: &[&[u8]]) -> u32 {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    let digest = hasher.finalize();
    match digest.first_chunk::<4>() {
        Some(chunk) => u32::from_be_bytes(*chunk),
        None => 0,
    }
}

/// `short_hash` formatado como 8 dígitos hexadecimais minúsculos.
#[must_use]
pub fn hex8(data: &[u8]) -> String {
    hex8_value(short_hash(data))
}

/// Formata um `u32` como 8 dígitos hexadecimais minúsculos.
#[must_use]
pub fn hex8_value(value: u32) -> String {
    format!("{value:08x}")
}

/// `u32` em `base36`, alinhado à direita com zeros até 8 caracteres.
#[allow(
    clippy::arithmetic_side_effects,
    reason = "conversão de base com `u32` e radix 36 constante"
)]
#[must_use]
pub fn base36_8(mut value: u32) -> String {
    const RADIX: u32 = 36;
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut buffer = [b'0'; 8];
    let mut index = buffer.len();
    loop {
        index = index.saturating_sub(1);
        let digit = usize::try_from(value % RADIX).unwrap_or(0);
        if let Some(slot) = buffer.get_mut(index) {
            *slot = DIGITS.get(digit).copied().unwrap_or(b'0');
        }
        value /= RADIX;
        if value == 0 || index == 0 {
            break;
        }
    }
    match std::str::from_utf8(&buffer) {
        Ok(text) => text.to_string(),
        Err(_) => String::from("00000000"),
    }
}
