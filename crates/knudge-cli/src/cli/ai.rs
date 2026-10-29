//! Argumentos de `kd ai` — área fina sobre uma **fonte** de memória de agente (D218/D220).
//!
//! A área é **posicional** (`memory` | `session-log`; default `memory`) e o **default é consulta**
//! (D130: nunca sai vazio em silêncio). Sem query, lista as notas da fonte; com query, faz recall
//! sobre ela. A fonte é sempre **read-only** — o agente possui o store, o knudge indexa-o.
//!
//! A memória v2 vive em `.knudge/ai/<area>/<agent_id>.jsonl` (um shard por agente, D220). Não há
//! caminho legado nem migração: `.agents/memory` deixa de ser lido (D224).

use std::fmt;

use clap::{Args, ValueEnum};
use knudge_core::{Error, Result};

/// Área de memória consultada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum AiArea {
    /// Memória do agente — `.knudge/ai/memory/*.jsonl`.
    #[default]
    Memory,
    /// Projeção do log de sessão (notas derivadas, `activity=derive`) —
    /// `.knudge/ai/session-log/*.jsonl`.
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

    /// Diretório convencional dos shards, relativo à raiz do projeto (D220).
    #[must_use]
    pub const fn default_path(self) -> &'static str {
        match self {
            Self::Memory => ".knudge/ai/memory",
            Self::SessionLog => ".knudge/ai/session-log",
        }
    }

    /// Interpreta o nome estável.
    ///
    /// # Errors
    /// `ErrorKind::InvalidInput` para área desconhecida.
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "memory" => Ok(Self::Memory),
            "session-log" => Ok(Self::SessionLog),
            other => Err(Error::invalid_input(format!(
                "área desconhecida: {other:?} (memory|session-log)"
            ))),
        }
    }
}

impl fmt::Display for AiArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ordem de saída do `kd ai` (D222).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum AiSort {
    /// `id` ascendente — ordem determinística histórica (default).
    #[default]
    Id,
    /// `(created_at, agent_id, id)` ascendente — eixo temporal (D222).
    Time,
}

impl AiSort {
    /// Rótulo estável.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Time => "time",
        }
    }

    /// Interpreta o rótulo estável.
    ///
    /// # Errors
    /// `ErrorKind::InvalidInput` para ordem desconhecida.
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "id" => Ok(Self::Id),
            "time" => Ok(Self::Time),
            other => Err(Error::invalid_input(format!(
                "ordem desconhecida: {other:?} (id|time)"
            ))),
        }
    }
}

impl fmt::Display for AiSort {
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
    /// Objeto JSON com a consulta e os filtros (`-` lê stdin) — D222.
    #[arg(long, value_name = "JSON")]
    pub params: Option<String>,
    /// Fonte explícita (ficheiro ou diretório; default: o diretório da área).
    #[arg(long, value_name = "PATH")]
    pub source: Option<String>,
    /// Limite de resultados do recall (default: config `recall.default_limit`).
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
    /// Ordem da lista/hits (`id` compatível; `time` por `created_at, agent_id, id`) — D222.
    #[arg(long, value_enum, default_value_t = AiSort::Id)]
    pub sort: AiSort,
    /// Filtro por instância do agente (`provenance.agent_id`; repetível/vírgula) — D221.
    #[arg(long = "agent-id", value_name = "ID", value_delimiter = ',')]
    pub agent_id: Vec<String>,
    /// Filtro por papel (`main|subagent`; repetível/vírgula) — D221.
    #[arg(long = "agent-role", value_name = "PAPEL", value_delimiter = ',')]
    pub agent_role: Vec<String>,
}
