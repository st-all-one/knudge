//! Schema canônico do knudge (E02).
//!
//! Fixa o **contrato de bytes** do frontmatter: ordem das chaves (D04/D13), enums fechados,
//! normalização/hash (D06/D95), IDs endereçados por conteúdo (D01–D03) e contagem de
//! `statement` (D08). A serialização concreta vive em [`crate::toon`].

pub mod body;
pub mod frontmatter;
pub mod hash;
pub mod id;
pub mod text;
pub mod types;
pub mod value;

#[cfg(test)]
mod tests;

/// Versão atual do schema (D15).
pub const SCHEMA_VERSION: u32 = 1;

pub use frontmatter::{CANONICAL_KEYS, Frontmatter, REQUIRED_KEYS};
pub use types::{Classification, NoteType, Scope, Status};
pub use value::Value;
