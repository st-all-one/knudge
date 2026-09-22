//! Escopo `git`: worktree principal, persistência e `sync` (E04).
//!
//! Módulo **puro**: só usa [`crate::ports::Git`]/[`crate::ports::Fs`]. Aqui vivem a resolução
//! do worktree principal (D29), a exclusão via `.git/info/exclude` (D30), o `merge=union` do
//! log (D31), o `AGENTS.md` idempotente (D60) e o `sync` (D32). Nada de shell (R12).

pub mod agent_md;
pub mod attributes;
pub mod block;
pub mod exclude;
pub mod onboard;
pub mod persistence;
pub mod project;
pub mod sync;

#[cfg(test)]
mod tests;

pub use agent_md::protocol_block;
pub use block::upsert;
pub use exclude::{DERIVED_PATTERNS, KNUDGE_PATTERN, exclude_path};
pub use onboard::{LAYOUT_DIRS, OnboardOptions, OnboardReport, onboard};
pub use persistence::Persistence;
pub use project::{KNUDGE_DIR, Project, is_valid_name, logical_name};
pub use sync::{SyncReport, sync};
