//! `doctor [--fix]`: diagnóstico e reparo do reversível (D19, E09-T04).
//!
//! Dez checks determinísticos cobrem schema/TOON, integridade, ciclos, âncoras, duplicatas,
//! locks stale, config, `body_hash` desatualizado, `eventos.jsonl` malformado e **divergência
//! canônico↔derivado** (D84). `--fix` corrige só o **reversível** e é **idempotente**: rodar
//! duas vezes não muda nada na segunda.

mod checks;
mod fix;

pub use fix::doctor_fix;

use std::path::Path;

use crate::Result;
use crate::config::Config;
use crate::graph::Graph;
use crate::ports::Fs;
use crate::retrieval::Index;
use crate::schema::body;
use crate::store::{EventLog, Note, Store};
use crate::write::dedup::DedupThresholds;

use super::tolerant::read_tolerant;
use checks::{
    anchors_check, body_hash_check, config_check, cycles_check, derived_check, duplicates_check,
    events_check, integrity_check, locks_check, schema_check,
};

/// Identificador estável de um check do `doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckId {
    /// Schema/TOON e notas malformadas.
    Schema,
    /// Integridade do grafo.
    Integrity,
    /// Ciclos de supersessão/dependência.
    Cycles,
    /// Âncoras quebradas/stale.
    Anchors,
    /// Quase-duplicatas.
    Duplicates,
    /// Locks stale.
    Locks,
    /// Configuração válida.
    Config,
    /// `body_hash` desatualizado.
    BodyHash,
    /// Eventos malformados.
    Events,
    /// Divergência canônico↔derivado.
    Derived,
}

impl CheckId {
    /// Todos os checks, na ordem de exibição.
    pub const ALL: [Self; 10] = [
        Self::Schema,
        Self::Integrity,
        Self::Cycles,
        Self::Anchors,
        Self::Duplicates,
        Self::Locks,
        Self::Config,
        Self::BodyHash,
        Self::Events,
        Self::Derived,
    ];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Integrity => "integrity",
            Self::Cycles => "cycles",
            Self::Anchors => "anchors",
            Self::Duplicates => "duplicates",
            Self::Locks => "locks",
            Self::Config => "config",
            Self::BodyHash => "body_hash",
            Self::Events => "events",
            Self::Derived => "derived",
        }
    }
}

/// Resultado de um check.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "`ok` e `fixable` são flags ortogonais do mesmo check"
)]
pub struct DoctorCheck {
    /// Identificador.
    pub id: CheckId,
    /// `true` se passou.
    pub ok: bool,
    /// Detalhe legível (com orientação quando falha).
    pub detail: String,
    /// `true` se o `--fix` consegue corrigir.
    pub fixable: bool,
}

/// Relatório do `doctor`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorReport {
    /// Checks, na ordem de [`CheckId::ALL`].
    pub checks: Vec<DoctorCheck>,
    /// Warnings de leitura tolerante/eventos.
    pub warnings: Vec<String>,
    /// Reparos aplicados pelo `--fix` (vazio no modo leitura).
    pub fixed: Vec<String>,
}

impl DoctorReport {
    /// `true` se todos os checks passaram.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|check| check.ok)
    }

    /// Check pelo identificador.
    #[must_use]
    pub fn check(&self, id: CheckId) -> Option<&DoctorCheck> {
        self.checks.iter().find(|check| check.id == id)
    }
}

/// Contexto do `doctor`.
pub struct DoctorInput<'a> {
    /// Porta de FS.
    pub fs: &'a dyn Fs,
    /// Raiz do `.knudge/`.
    pub root: &'a Path,
    /// Raiz do projeto (âncoras relativas).
    pub project_root: &'a Path,
    /// Store de notas.
    pub store: &'a Store<'a>,
    /// Log de eventos.
    pub events: &'a EventLog<'a>,
    /// Config efetiva.
    pub config: &'a Config,
    /// Grafo de arestas.
    pub graph: &'a Graph,
    /// Instante atual (ms).
    pub now_ms: i64,
    /// Idade a partir da qual um lock é stale (ms).
    pub lock_stale_ms: i64,
    /// Limiares de dedup.
    pub thresholds: &'a DedupThresholds,
}

/// Executa os dez checks (somente leitura).
///
/// # Errors
/// Propaga erros de I/O de listagem/leitura do derivado.
pub fn doctor(input: &DoctorInput<'_>) -> Result<DoctorReport> {
    let read = read_tolerant(input.store)?;
    let expected = Index::build(&read.notes)?;
    let mut warnings = read.warnings.clone();
    let checks = vec![
        schema_check(&read.skipped),
        integrity_check(input),
        cycles_check(input),
        anchors_check(input)?,
        duplicates_check(&expected, input),
        locks_check(input)?,
        config_check(input),
        body_hash_check(&read.notes)?,
        events_check(input),
        derived_check(input, &expected, &mut warnings),
    ];
    Ok(DoctorReport {
        checks,
        warnings,
        fixed: Vec::new(),
    })
}

pub(crate) fn body_hash_mismatch(note: &Note) -> Result<bool> {
    let expected = body::body_hash(note.frontmatter.statement()?, &note.body);
    let stored = note.frontmatter.get("body_hash").and_then(|v| v.as_str());
    Ok(stored != Some(expected.as_str()))
}

pub(crate) fn derived_diverges(
    input: &DoctorInput<'_>,
    expected: &Index,
    warnings: &mut Vec<String>,
) -> bool {
    match Index::load(input.fs, input.root, warnings) {
        Ok(Some(persisted)) => persisted.docs != expected.docs,
        Ok(None) | Err(_) => true,
    }
}
