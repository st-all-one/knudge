//! Escopo `health`: validação, auditoria, `doctor`, leitura tolerante e evidência (E09).
//!
//! O corpus **não apodrece em silêncio**: sabe-se o que está quebrado, o que é stale e por quê,
//! e o reparo do reversível é automático (`doctor --fix`). Uma nota ruim nunca derruba um
//! comando (D16–D18); nada é apagado sem decisão (D52/D86).

pub mod anchors;
pub mod audit;
pub mod doctor;
pub mod evidence;
pub mod gate;
pub mod tolerant;
pub mod validator;

#[cfg(test)]
mod tests;

pub use anchors::{
    AnchorRecord, AnchorRole, AnchorStore, StaleAnchor, StaleReason, anchor_role, hash_file,
    invalidated_notes, is_glob, refresh, verify,
};
pub use audit::{
    AuditInput, AuditReport, BrokenAnchor, Duplicate, MissingEdge, StaleLock, audit,
    stale_lock_paths,
};
pub use doctor::{CheckId, DoctorCheck, DoctorInput, DoctorReport, doctor, doctor_fix};
pub use evidence::{CheckOutcome, CheckResult, CloseOutcome, close_task, infer_outcome};
pub use gate::{GateOutcome, accept};
pub use tolerant::{SkippedNote, TolerantRead, read_note_tolerant, read_tolerant};
pub use validator::{
    CATALOG_FILE, CheckSource, DEFAULT_TIMEOUT_MS, ResolvedCheck, ResolvedChecks, Severity,
    Validator, ValidatorCatalog, ValidatorKind, resolve_checks,
};
