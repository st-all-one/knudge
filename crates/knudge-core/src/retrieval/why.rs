//! Motivo (`why`) de um hit de `recall` (D39).
//!
//! Conjunto **fechado**: a 4ª coluna do contrato `recall` só pode assumir um destes valores.

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Por que um hit apareceu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Why {
    /// Âncora casou com um arquivo do working set (`file_match`).
    FileMatch,
    /// Âncora casou exatamente com um id (`anchor_match`).
    AnchorMatch,
    /// A nota pertence ao escopo pedido (`tracker_match`).
    TrackerMatch,
    /// Há confirmação registrada em `outcomes` (`stars`).
    Stars,
    /// Veio pelo **canal vetorial** (`semantic`, D121).
    Semantic,
    /// Criada dentro da janela de recência (`recent`).
    Recent,
    /// Canal lexical geral, sem motivo mais forte (`universal`).
    Universal,
}

impl Why {
    /// Todos os motivos, na ordem canônica.
    pub const ALL: [Self; 7] = [
        Self::FileMatch,
        Self::AnchorMatch,
        Self::TrackerMatch,
        Self::Stars,
        Self::Semantic,
        Self::Recent,
        Self::Universal,
    ];

    /// Rótulo estável de máquina.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileMatch => "file_match",
            Self::AnchorMatch => "anchor_match",
            Self::TrackerMatch => "tracker_match",
            Self::Stars => "stars",
            Self::Semantic => "semantic",
            Self::Recent => "recent",
            Self::Universal => "universal",
        }
    }
}

impl fmt::Display for Why {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Why {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|why| why.as_str() == s)
            .ok_or_else(|| Error::invalid_input(format!("why desconhecido: {s:?}")))
    }
}
