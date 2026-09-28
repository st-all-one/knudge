//! `FromStr` dos enums fechados do schema, com "did-you-mean" (D212).
//!
//! Separado de `types.rs` para manter o teto de 300 linhas. O erro lista as possibilidades e
//! sugere a mais provável via [`super::suggest`].

use std::str::FromStr;

use super::suggest;
use super::types::{Classification, NoteType, Scope, Status};
use crate::{Error, Result};

/// Erro de valor desconhecido com a lista de possibilidades e a mais provável (D212).
fn unknown(what: &str, raw: &str, valid: &[&str]) -> Error {
    Error::schema(format!("{what}: {raw:?} ({})", suggest::hint(raw, valid)))
}

impl FromStr for NoteType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|note_type| note_type.as_str() == s)
            .ok_or_else(|| unknown("tipo desconhecido", s, &Self::ALL.map(Self::as_str)))
    }
}

impl FromStr for Scope {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|scope| scope.as_str() == s)
            .ok_or_else(|| unknown("scope desconhecido", s, &Self::ALL.map(Self::as_str)))
    }
}

impl FromStr for Classification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.as_str() == s)
            .ok_or_else(|| {
                unknown(
                    "classification desconhecida",
                    s,
                    &Self::ALL.map(Self::as_str),
                )
            })
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.as_str() == s)
            .ok_or_else(|| unknown("status desconhecido", s, &Self::ALL.map(Self::as_str)))
    }
}
