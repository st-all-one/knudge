//! RNG real, semeado pelo sistema operacional (usado só para jitter — R12).

use std::hash::{BuildHasher, Hasher};

use crate::ports::Rng;

/// RNG xorshift64 semeado com entropia do `RandomState`.
#[derive(Debug, Clone)]
pub struct ThreadRng {
    /// Estado interno.
    state: u64,
}

impl ThreadRng {
    /// Cria um RNG semeado pelo SO.
    #[must_use]
    pub fn new() -> Self {
        let seed = std::hash::RandomState::new().build_hasher().finish();
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }
}

impl Default for ThreadRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng for ThreadRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
