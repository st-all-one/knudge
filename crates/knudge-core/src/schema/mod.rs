//! Schema canônico do knudge (E02).
//!
//! Fixa o **contrato de bytes** do frontmatter: ordem das chaves (D04/D13), enums fechados,
//! normalização/hash (D06/D95), IDs endereçados por conteúdo (D01–D03) e contagem de
//! `statement` (D08). A serialização concreta vive em [`crate::toon`].

pub mod body;
pub mod claims;
pub mod edge;
pub mod frontmatter;
pub mod hash;
pub mod id;
pub mod keys;
pub mod outcomes;
pub mod provenance;
pub mod slots;
pub mod suggest;
pub mod text;
pub mod types;
pub mod value;

mod parse;

#[cfg(test)]
mod tests;

/// Versão atual do schema (D15/D207).
pub const SCHEMA_VERSION: u32 = 2;

pub use claims::{Claim, claims, claims_from_value, claims_to_value};
pub use edge::{EDGE_KEYS, Edge, EdgeKind};
pub use frontmatter::Frontmatter;
pub use keys::{CANONICAL_KEYS, REQUIRED_KEYS};
pub use outcomes::{OutcomeStats, outcome_stats};
pub use provenance::{Provenance, provenance, provenance_from_value};
pub use slots::{Slot, expected_slots, missing_slots};
pub use types::{Classification, NoteType, Scope, Status};
pub use value::Value;
