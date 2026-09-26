//! Registro derivado de hashes de âncora em `.idx/anchors.jsonl` (E09-T06).

use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

/// Nome do arquivo derivado dentro de `.idx/`.
pub const ANCHOR_FILE: &str = "anchors.jsonl";

/// Papel da âncora no verify-on-hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnchorRole {
    /// Citada no corpo: mudança de conteúdo invalida a nota.
    Cited,
    /// Só contexto: mudança de conteúdo não invalida.
    Context,
}

impl AnchorRole {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cited => "cited",
            Self::Context => "context",
        }
    }

    pub(super) fn parse(text: &str) -> Self {
        if text == "cited" {
            Self::Cited
        } else {
            Self::Context
        }
    }
}

/// Motivo de staleness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleReason {
    /// O arquivo ancorado não existe mais.
    Missing,
    /// O conteúdo mudou desde o último refresh.
    ContentChanged,
}

impl StaleReason {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::ContentChanged => "content_changed",
        }
    }
}

/// Registro derivado de uma âncora.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorRecord {
    /// Id da nota.
    pub id: String,
    /// Caminho (relativo ao projeto).
    pub path: String,
    /// `hex8` do conteúdo no último refresh.
    pub content_hash: String,
    /// Papel.
    pub role: AnchorRole,
}

/// Âncora considerada stale na verificação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleAnchor {
    /// Id da nota.
    pub id: String,
    /// Caminho.
    pub path: String,
    /// Papel.
    pub role: AnchorRole,
    /// Motivo.
    pub reason: StaleReason,
}

impl StaleAnchor {
    /// `true` se o staleness **invalida** a nota (papel `cited` — D86).
    #[must_use]
    pub const fn invalidates(&self) -> bool {
        matches!(self.role, AnchorRole::Cited)
    }
}

/// Store derivado dos hashes de âncora.
pub struct AnchorStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> AnchorStore<'a> {
    /// Cria o store com raiz em `.knudge/`.
    #[must_use]
    pub fn new(fs: &'a dyn Fs, root: impl Into<PathBuf>) -> Self {
        Self {
            fs,
            root: root.into(),
        }
    }

    /// Caminho do arquivo derivado.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.root.join(".idx").join(ANCHOR_FILE)
    }

    /// Lê todos os registros, ordenados.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se o arquivo existir e não for legível.
    pub fn list(&self) -> Result<Vec<AnchorRecord>> {
        let path = self.path();
        if !self.fs.exists(&path) {
            return Ok(Vec::new());
        }
        let bytes = self.fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        let mut records = Vec::new();
        for line in jsonl::lines(text) {
            if let Ok(value) = json::decode(line)
                && let Some(record) = parse_record(&value)
            {
                records.push(record);
            }
        }
        records
            .sort_unstable_by(|left, right| (&left.id, &left.path).cmp(&(&right.id, &right.path)));
        Ok(records)
    }

    /// Substitui os registros de `id` pelos fornecidos. Devolve `true` se algo mudou.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` em falha de escrita.
    pub fn replace(&self, id: &str, records: &[AnchorRecord]) -> Result<bool> {
        let existing = self.list()?;
        let mut all: Vec<AnchorRecord> = existing
            .iter()
            .filter(|record| record.id != id)
            .cloned()
            .collect();
        all.extend(records.iter().cloned());
        all.sort_unstable_by(|left, right| (&left.id, &left.path).cmp(&(&right.id, &right.path)));
        if all == existing {
            return Ok(false);
        }
        let path = self.path();
        if all.is_empty() {
            if self.fs.exists(&path) {
                self.fs.remove_file(&path)?;
            }
            return Ok(true);
        }
        if let Some(parent) = path.parent() {
            self.fs.create_dir_all(parent)?;
        }
        let mut out = String::new();
        for record in &all {
            out.push_str(&json::encode(&to_value(record))?);
            out.push('\n');
        }
        self.fs.write_atomic(&path, out.as_bytes())?;
        Ok(true)
    }
}

fn to_value(record: &AnchorRecord) -> Value {
    Value::map([
        ("id".to_string(), Value::Str(record.id.clone())),
        ("path".to_string(), Value::Str(record.path.clone())),
        ("hash".to_string(), Value::Str(record.content_hash.clone())),
        (
            "role".to_string(),
            Value::Str(record.role.as_str().to_string()),
        ),
    ])
}

fn parse_record(value: &Value) -> Option<AnchorRecord> {
    let map = value.as_map()?;
    Some(AnchorRecord {
        id: map.get("id")?.as_str()?.to_string(),
        path: map.get("path")?.as_str()?.to_string(),
        content_hash: map
            .get("hash")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        role: AnchorRole::parse(map.get("role").and_then(Value::as_str).unwrap_or("context")),
    })
}
