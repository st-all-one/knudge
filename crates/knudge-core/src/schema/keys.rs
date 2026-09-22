//! Chaves canônicas do frontmatter (D04/D13/D98/D100).
//!
//! A **ordem** é contrato: [`Frontmatter::to_value`](super::frontmatter::Frontmatter::to_value)
//! sempre emite nesta ordem, independentemente da ordem de inserção.

/// Ordem canônica das chaves do frontmatter (D04/D13/D100) — 28 chaves.
pub const CANONICAL_KEYS: [&str; 28] = [
    "id",
    "type",
    "statement",
    "created_at",
    "confidence",
    "body_hash",
    "schema_version",
    "tags",
    "source",
    "expires_at",
    "not_before",
    "superseded_by",
    "references",
    "depends_on",
    "contradicts",
    "supports",
    "extends",
    "replaces",
    "rejects",
    "results_in",
    "revision",
    "outcomes",
    "classification",
    "anchors",
    "status",
    "scope",
    "checks",
    "evidence",
];

/// Chaves obrigatórias: ou a nota é válida, ou não existe (D05).
pub const REQUIRED_KEYS: [&str; 7] = [
    "id",
    "type",
    "statement",
    "created_at",
    "confidence",
    "body_hash",
    "schema_version",
];
