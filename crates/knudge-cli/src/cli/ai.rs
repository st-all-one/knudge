//! Argumentos de `kd ai` — área fina sobre uma **fonte** de memória de agente (D218).
//!
//! A área é **posicional** (`memory` | `session-log`; default `memory`) e o **default é consulta**
//! (D130: nunca sai vazio em silêncio). Sem query, lista as notas da fonte; com query, faz recall
//! sobre ela. A fonte é sempre **read-only** — o agente possui o store, o knudge indexa-o.

use std::fmt;

use clap::{Args, ValueEnum};

/// Área de memória consultada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum AiArea {
    /// Memória do agente — `.agents/memory/notes.jsonl`.
    #[default]
    Memory,
    /// Projeção do log de sessão (notas derivadas, `activity=derive`) —
    /// `.agents/memory/session-log.jsonl`.
    SessionLog,
}

impl AiArea {
    /// Nome estável (contrato de máquina e de texto).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::SessionLog => "session-log",
        }
    }

    /// Caminho convencional da fonte, relativo à raiz do projeto (D218).
    #[must_use]
    pub const fn default_path(self) -> &'static str {
        match self {
            Self::Memory => ".agents/memory/notes.jsonl",
            Self::SessionLog => ".agents/memory/session-log.jsonl",
        }
    }
}

impl fmt::Display for AiArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Argumentos de `kd ai`.
#[derive(Debug, Clone, Default, Args)]
pub struct AiArgs {
    /// Área consultada (`memory` | `session-log`; default `memory`).
    #[arg(value_name = "AREA", value_enum, default_value_t = AiArea::Memory)]
    pub area: AiArea,
    /// Consulta textual; sem ela, lista as notas da fonte (`id|tipo|statement`).
    #[arg(value_name = "QUERY")]
    pub query: Vec<String>,
    /// Fonte JSONL explícita (default: o caminho convencional da área).
    #[arg(long, value_name = "PATH")]
    pub source: Option<String>,
    /// Limite de resultados do recall (default: config `recall.default_limit`).
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}
