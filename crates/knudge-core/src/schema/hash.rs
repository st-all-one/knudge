//! Hash curto do knudge (D95).
//!
//! Um único algoritmo para `id` e `body_hash`: **SHA-256 truncado aos 4 primeiros bytes**
//! (`u32` big-endian). O `body_hash` sai como `hex8`; o `id` como `base36(8)`.

use sha2::{Digest, Sha256};

/// Primeiros 4 bytes big-endian de `SHA-256(data)`, como `u32`.
#[must_use]
pub fn short_hash(data: &[u8]) -> u32 {
    let digest = Sha256::digest(data);
    match digest.first_chunk::<4>() {
        Some(chunk) => u32::from_be_bytes(*chunk),
        None => 0,
    }
}

/// `short_hash` formatado como 8 dígitos hexadecimais minúsculos.
#[must_use]
pub fn hex8(data: &[u8]) -> String {
    format!("{:08x}", short_hash(data))
}

/// `u32` em `base36`, alinhado à direita com zeros até 8 caracteres.
#[allow(
    clippy::arithmetic_side_effects,
    reason = "conversão de base com `u32` e radix 36 constante"
)]
#[must_use]
pub fn base36_8(mut value: u32) -> String {
    const RADIX: u32 = 36;
    let mut digits: Vec<char> = Vec::with_capacity(8);
    loop {
        let digit = value % RADIX;
        digits.push(char::from_digit(digit, RADIX).unwrap_or('0'));
        value /= RADIX;
        if value == 0 {
            break;
        }
    }
    while digits.len() < 8 {
        digits.push('0');
    }
    digits.reverse();
    digits.into_iter().collect()
}
