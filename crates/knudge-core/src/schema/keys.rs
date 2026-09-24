//! Chaves canônicas do frontmatter (D04/D13/D98/D100/D135/D142).
//!
//! A **ordem** é contrato: [`Frontmatter::to_value`](super::frontmatter::Frontmatter::to_value)
//! sempre emite nesta ordem, independentemente da ordem de inserção.

/// Ordem canônica das chaves do frontmatter (D04/D13/D100/D135/D142) — 25 chaves.
pub const CANONICAL_KEYS: [&str; 25] = [
    "id",
    "type",
    "statement",
    "created_at",
    "body_hash",
    "schema_version",
    "tags",
    "source",
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
///
/// `type` **não** entra: é obrigatório para espécies, mas **omitido** quando `scope=epic`
/// (o tipo efetivo é derivado — D149). [`Frontmatter::note_type`] cobra a presença.
pub const REQUIRED_KEYS: [&str; 5] = [
    "id",
    "statement",
    "created_at",
    "body_hash",
    "schema_version",
];
