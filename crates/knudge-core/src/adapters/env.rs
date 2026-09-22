//! Ambiente real do processo.

use std::path::PathBuf;

use crate::Result;
use crate::ports::Env;

/// Acesso ao ambiente via `std::env`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdEnv;

impl StdEnv {
    /// Cria o adaptador.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Env for StdEnv {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de ambiente real; o domínio usa a porta Env (D65)"
    )]
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn args(&self) -> Vec<String> {
        std::env::args().skip(1).collect()
    }

    fn current_dir(&self) -> Result<PathBuf> {
        std::env::current_dir().map_err(|e| crate::Error::io(".", e))
    }
}
