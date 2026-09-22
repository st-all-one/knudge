//! Âncoras com `content_hash` derivado e verify-on-hit (D86, E09-T06).
//!
//! O `path` vive no frontmatter (`anchors`); o **hash do conteúdo** vive só no derivado
//! `.idx/anchors.jsonl` e **nunca** na nota. No hit, o hash vigente é rechecado: âncora
//! **`cited`** (citada no corpo) invalida a nota quando muda; âncora **`context`** não. Nota
//! stale é **sinalizada, nunca apagada** (D86).

mod store;
mod verify;

pub use store::{ANCHOR_FILE, AnchorRecord, AnchorRole, AnchorStore, StaleAnchor, StaleReason};
pub use verify::{anchor_role, hash_file, invalidated_notes, is_glob, refresh, verify};
