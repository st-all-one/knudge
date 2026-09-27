//! Blocking MinHash/LSH para o dedup em corpora grandes (E19/T07/D204).
//!
//! A peneira lexical (`Sieve`) é **exata**, mas degenera em O(N²) quando o vocabulário é denso
//! (quase todo documento compartilha algum termo com quase todos). Acima de [`MIN_LSH_CORPUS`] o
//! dedup usa **assinaturas `MinHash`** + *banding* LSH para reduzir os pares candidatos; abaixo do
//! limiar, a peneira exata (byte-idêntica) continua valendo.
//!
//! LSH é **aproximado**: um par com Jaccard `s` tem probabilidade `1 − (1 − s^ROWS)^BANDS` de
//! compartilhar ao menos uma banda. Com `ROWS=4`/`BANDS=16`, pares com `s ≥ 0,92` (limiar de
//! merge) são praticamente certos, e pares distantes são descartados.

use std::collections::{BTreeMap, BTreeSet};

/// Comprimento da assinatura (nº de permutações).
pub const SIGNATURE_LEN: usize = 64;
/// Bandas por assinatura.
pub const BANDS: usize = 16;
/// Linhas por banda (`BANDS * ROWS == SIGNATURE_LEN`).
pub const ROWS: usize = 4;
/// Abaixo deste corpus a peneira lexical exata é usada (barata e byte-idêntica).
pub const MIN_LSH_CORPUS: usize = 256;

/// Assinatura `MinHash`: para cada permutação `i`, o menor `hash(termo, i)` do conjunto.
///
/// Determinística e independente de ordem (o mínimo comuta). Conjunto vazio devolve `u64::MAX`
/// em todas as posições.
#[must_use]
pub fn signature<'a>(terms: impl IntoIterator<Item = &'a str>) -> [u64; SIGNATURE_LEN] {
    let mut signature = [u64::MAX; SIGNATURE_LEN];
    for term in terms {
        let base = fnv1a(term.as_bytes());
        for (slot, value) in signature.iter_mut().enumerate() {
            let candidate = permute(base, slot);
            if candidate < *value {
                *value = candidate;
            }
        }
    }
    signature
}

/// Pares `(i, j)` com `i < j` que compartilham ao menos uma banda, em ordem canônica.
#[must_use]
pub fn candidate_pairs(signatures: &[[u64; SIGNATURE_LEN]]) -> Vec<(usize, usize)> {
    let mut pairs: BTreeSet<(usize, usize)> = BTreeSet::new();
    for band in 0..BANDS {
        let start = band.saturating_mul(ROWS);
        let mut buckets: BTreeMap<[u64; ROWS], Vec<usize>> = BTreeMap::new();
        for (position, signature) in signatures.iter().enumerate() {
            let mut key = [0_u64; ROWS];
            for (slot, value) in key.iter_mut().enumerate() {
                *value = signature
                    .get(start.saturating_add(slot))
                    .copied()
                    .unwrap_or(0);
            }
            buckets.entry(key).or_default().push(position);
        }
        for positions in buckets.values() {
            for (offset, &left) in positions.iter().enumerate() {
                for &right in positions.iter().skip(offset.saturating_add(1)) {
                    let pair = if left < right {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    let _ignored = pairs.insert(pair);
                }
            }
        }
    }
    pairs.into_iter().collect()
}

/// FNV-1a de 64 bits: estável entre execuções/plataformas.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Deriva a permutação `slot` de um mesmo `base` (splitmix64).
fn permute(base: u64, slot: usize) -> u64 {
    let slot = u64::try_from(slot).unwrap_or(u64::MAX);
    mix(base ^ slot.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Finalizador splitmix64: mistura forte e determinística.
fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
