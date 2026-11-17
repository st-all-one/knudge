//! Refs versionadas de handoff: `id@body_hash` e `anchor@content_hash` (D227).
//!
//! O texto padrão do `rewind` permanece byte-idêntico (D88). As refs só entram com
//! `rewind --with-refs`, e a verificação é **advisory**: nunca reescreve conhecimento.

use crate::Result;
use crate::health::{AnchorRecord, AnchorStore, hash_file};
use crate::ports::Fs;
use crate::schema::{Status, Value};
use crate::store::Store;
use std::collections::BTreeMap;
use std::path::Path;

/// Prefixo da linha que registra uma âncora versionada no handoff.
pub const ANCHOR_LINE_PREFIX: &str = "anchor|";

/// Separador entre nome e hash.
pub const REF_SEP: char = '@';

/// Tipo de ref citada no handoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    /// Nota por `id@body_hash`.
    Note,
    /// Âncora por `path@content_hash`.
    Anchor,
}

impl RefKind {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Anchor => "anchor",
        }
    }
}

/// Estado da verificação de uma ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefStatus {
    /// Ref confere com o corrente (não é reportada).
    Current,
    /// Mesmo nome, hash diferente.
    Changed,
    /// Nota substituída por outra.
    Superseded,
    /// Nota aposentada (`status=forgotten`).
    Retired,
    /// Nota/âncora inexistente.
    Dangling,
}

impl RefStatus {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Changed => "changed",
            Self::Superseded => "superseded",
            Self::Retired => "retired",
            Self::Dangling => "dangling",
        }
    }
}

/// Achado de verificação (somente divergências).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefFinding {
    /// Tipo de ref.
    pub kind: RefKind,
    /// Id da nota ou caminho da âncora.
    pub name: String,
    /// Hash citado no handoff.
    pub old_hash: String,
    /// Hash vigente, quando divergente.
    pub new_hash: Option<String>,
    /// Estado.
    pub status: RefStatus,
    /// Detalhe (novo id ou motivo).
    pub detail: Option<String>,
}

impl RefFinding {
    /// Renderiza um achado em stderr.
    #[must_use]
    pub fn render(&self) -> String {
        let cited = format!("{}{REF_SEP}{}", self.name, self.old_hash);
        match (self.status, &self.new_hash, &self.detail) {
            (RefStatus::Changed, Some(new), _) => format!("changed {cited} -> {new}"),
            (RefStatus::Superseded, _, Some(new_id)) => {
                format!("superseded {cited} por {new_id}")
            }
            (RefStatus::Retired, _, Some(reason)) => format!("retired {cited} ({reason})"),
            _ => format!("{} {cited}", self.status.as_str()),
        }
    }
}

/// Renderiza `nome@hash`.
#[must_use]
pub fn render_ref(name: &str, hash: &str) -> String {
    format!("{name}{REF_SEP}{hash}")
}

/// Parseia `nome@hash` (separa no último `@`; ambos não vazios).
#[must_use]
pub fn parse_ref(value: &str) -> Option<(&str, &str)> {
    let (name, hash) = value.rsplit_once(REF_SEP)?;
    (!name.is_empty() && !hash.is_empty()).then_some((name, hash))
}

/// Anota um handoff já renderizado com refs, sem mudar o default (D227).
///
/// Recebe os hashes das notas citadas e das âncoras vigentes. As linhas de âncora são
/// acrescentadas ao fim, em formato próprio (`anchor|path@hash`).
#[must_use]
pub fn annotate_refs(
    text: &str,
    body_hashes: &BTreeMap<String, String>,
    anchor_hashes: &BTreeMap<String, String>,
) -> String {
    let trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with(ANCHOR_LINE_PREFIX) {
            lines.push(line.to_string());
            continue;
        }
        let mut fields: Vec<String> = line.split('|').map(str::to_string).collect();
        if let Some(first) = fields.first_mut()
            && !first.contains(REF_SEP)
            && let Some(hash) = body_hashes.get(first.as_str())
        {
            let rendered = render_ref(first, hash);
            *first = rendered;
        }
        lines.push(fields.join("|"));
    }
    for (path, hash) in anchor_hashes {
        lines.push(format!("{ANCHOR_LINE_PREFIX}{}", render_ref(path, hash)));
    }
    let mut out = lines.join("\n");
    if trailing_newline {
        out.push('\n');
    }
    out
}

/// Verifica as refs de um handoff advisorymente (D227).
///
/// # Errors
/// Propaga erros de leitura do índice de âncoras e do sistema de arquivos.
pub fn verify_refs(
    store: &Store<'_>,
    anchors: &AnchorStore<'_>,
    fs: &dyn Fs,
    project_root: &Path,
    text: &str,
) -> Result<Vec<RefFinding>> {
    let records = anchors.list()?;
    let mut out = Vec::new();
    for line in text.lines() {
        let finding = if let Some(raw) = line.strip_prefix(ANCHOR_LINE_PREFIX) {
            verify_anchor_line(raw, &records, fs, project_root)?
        } else {
            verify_note_line(line, store)?
        };
        if let Some(finding) = finding {
            out.push(finding);
        }
    }
    Ok(out)
}

/// Verifica uma linha `anchor|path@hash`.
fn verify_anchor_line(
    raw: &str,
    records: &[AnchorRecord],
    fs: &dyn Fs,
    project_root: &Path,
) -> Result<Option<RefFinding>> {
    let Some((path, hash)) = parse_ref(raw) else {
        return Ok(None);
    };
    if !records.iter().any(|record| record.path == path) {
        return Ok(Some(Parts::dangling(RefKind::Anchor, path, hash).build()));
    }
    match hash_file(fs, &project_root.join(path))? {
        Some(current) if current != hash => Ok(Some(
            Parts::changed(RefKind::Anchor, path, hash, current).build(),
        )),
        Some(_) => Ok(None),
        None => Ok(Some(Parts::dangling(RefKind::Anchor, path, hash).build())),
    }
}

/// Verifica uma linha `id@hash|statement|tier`.
fn verify_note_line(line: &str, store: &Store<'_>) -> Result<Option<RefFinding>> {
    let first = line.split('|').next().unwrap_or("");
    let Some((id, hash)) = parse_ref(first) else {
        return Ok(None);
    };
    let Some(note) = store.read_optional(id)? else {
        return Ok(Some(Parts::dangling(RefKind::Note, id, hash).build()));
    };
    let status = note.frontmatter.status()?;
    if status == Status::Forgotten {
        return Ok(Some(
            Parts::new(RefKind::Note, RefStatus::Retired, id, hash)
                .detail("forgotten")
                .build(),
        ));
    }
    if status == Status::Superseded {
        let new_id = note
            .frontmatter
            .get("superseded_by")
            .and_then(Value::as_str)
            .unwrap_or("desconhecido");
        return Ok(Some(
            Parts::new(RefKind::Note, RefStatus::Superseded, id, hash)
                .detail(new_id)
                .build(),
        ));
    }
    let current = note
        .frontmatter
        .get("body_hash")
        .and_then(Value::as_str)
        .unwrap_or("");
    if current == hash {
        return Ok(None);
    }
    Ok(Some(
        Parts::new(RefKind::Note, RefStatus::Changed, id, hash)
            .new_hash(current)
            .build(),
    ))
}

/// Construtor com campos opcionais (evita função com muitos argumentos).
struct Parts<'a> {
    kind: RefKind,
    status: RefStatus,
    name: &'a str,
    old_hash: &'a str,
    new_hash: Option<String>,
    detail: Option<String>,
}

impl<'a> Parts<'a> {
    fn new(kind: RefKind, status: RefStatus, name: &'a str, old_hash: &'a str) -> Self {
        Self {
            kind,
            status,
            name,
            old_hash,
            new_hash: None,
            detail: None,
        }
    }

    fn dangling(kind: RefKind, name: &'a str, hash: &'a str) -> Self {
        Self::new(kind, RefStatus::Dangling, name, hash)
    }

    fn changed(kind: RefKind, name: &'a str, hash: &'a str, current: String) -> Self {
        let mut parts = Self::new(kind, RefStatus::Changed, name, hash);
        parts.new_hash = Some(current);
        parts
    }

    fn new_hash(mut self, current: &str) -> Self {
        self.new_hash = Some(current.to_string());
        self
    }

    fn detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    fn build(self) -> RefFinding {
        RefFinding {
            kind: self.kind,
            status: self.status,
            name: self.name.to_string(),
            old_hash: self.old_hash.to_string(),
            new_hash: self.new_hash,
            detail: self.detail,
        }
    }
}
