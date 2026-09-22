//! Config do servidor MCP (E14-T05, D68).
//!
//! Lê `.knudge/config.toml` do diretório atual e cai nos defaults quando ausente/ilegível —
//! o servidor nunca falha por falta de config.

use std::path::PathBuf;

use knudge_core::adapters::StdFs;
use knudge_core::config::Config;

/// Cap de hints default.
pub const DEFAULT_HINTS_CAP: usize = 3;
/// Sessões de observação default.
pub const DEFAULT_OBSERVATION_SESSIONS: u32 = 3;

/// Parâmetros efetivos do servidor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpConfig {
    /// Cap de hints por gatilho.
    pub hints_cap: usize,
    /// Sessões em modo observação (`0` desliga).
    pub observation_sessions: u32,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            hints_cap: DEFAULT_HINTS_CAP,
            observation_sessions: DEFAULT_OBSERVATION_SESSIONS,
        }
    }
}

impl McpConfig {
    /// Lê `.knudge/config.toml` do diretório atual; defaults se ausente/ilegível.
    #[must_use]
    pub fn load_from_cwd() -> Self {
        let path = PathBuf::from(".knudge").join("config.toml");
        let fs = StdFs::new();
        let Ok(Some(config)) = Config::load(&fs, &path) else {
            return Self::default();
        };
        Self::from_config(&config)
    }

    /// Deriva os parâmetros de uma config efetiva.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let hints_cap = config
            .get_int("mcp.hints_cap")
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(DEFAULT_HINTS_CAP);
        let observing = config.get_bool("mcp.observation_mode").unwrap_or(true);
        let sessions = config
            .get_int("mcp.observation_sessions")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(DEFAULT_OBSERVATION_SESSIONS);
        Self {
            hints_cap,
            observation_sessions: if observing { sessions } else { 0 },
        }
    }
}
