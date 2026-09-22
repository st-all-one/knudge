//! Relógio real do sistema.

use crate::ports::Clock;
use crate::time::Timestamp;

/// Relógio de parede em UTC, com precisão de milissegundos.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    /// Cria o relógio.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de relógio real; o domínio usa a porta Clock (D65)"
    )]
    fn now(&self) -> Timestamp {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => {
                let millis = i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
                Timestamp::from_millis(millis)
            }
            Err(_) => Timestamp::EPOCH,
        }
    }
}
