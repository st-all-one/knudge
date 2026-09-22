//! Escopo `maintenance`: `diff`, `learn` e `compact` (E08-T05/T06/T08).
//!
//! Operações de passado e de consolidação: `diff` lê a auditoria de eventos, `learn` propõe
//! notas/links/merges a partir de eventos + âncoras e `compact` propõe fusões. Nada aqui escreve
//! sem aceite (D33/D47).

pub mod compact;
pub mod diff;
pub mod learn;

#[cfg(test)]
mod tests;

pub use compact::{ApplyOutcome, CompactProposal, CompactStrategy, apply_compact, propose_compact};
pub use diff::{DiffEntry, diff};
pub use learn::{LearnInput, LearnKind, LearnProposal, learn};
