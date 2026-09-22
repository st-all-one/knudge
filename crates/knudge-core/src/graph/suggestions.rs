//! Armazenamento derivado de sugestões de aresta (D50).
//!
//! Um arquivo JSONL em `.idx/suggestions.jsonl`, uma linha por `(id, kind, reason)` com uma
//! lista `targets`. É **derivado**: pode ser apagado/rebuildado e é purgado por
//! [`crate::store::purge_derived`] (o `id` da origem remove a linha; a lista `targets` é
//! podada quando um alvo é removido — D84).

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::{EdgeKind, Value};
use crate::{Error, Result};

use super::Suggestion;

/// Sugestões persistidas de uma nota de origem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionRecord {
    /// Id da nota que originou a sugestão.
    pub id: String,
    /// Tipo sugerido.
    pub kind: EdgeKind,
    /// Motivo legível.
    pub reason: String,
    /// Alvos sugeridos.
    pub targets: Vec<String>,
}

/// Store derivado de sugestões.
pub struct SuggestionStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> SuggestionStore<'a> {
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
        self.root.join(".idx").join("suggestions.jsonl")
    }

    /// Substitui as sugestões da nota `id` por `suggestions` (rebuild determinístico).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` em falha de leitura/escrita/parse.
    pub fn write(&self, id: &str, suggestions: &[Suggestion]) -> Result<()> {
        let mut records: Vec<SuggestionRecord> = self
            .list()?
            .into_iter()
            .filter(|record| record.id != id)
            .collect();
        for record in group(id, suggestions) {
            records.push(record);
        }
        records.sort_by(|left, right| {
            (&left.id, left.kind, &left.reason).cmp(&(&right.id, right.kind, &right.reason))
        });
        self.persist(&records)
    }

    /// Lê todas as sugestões persistidas, ordenadas.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se o arquivo existir e não for legível.
    pub fn list(&self) -> Result<Vec<SuggestionRecord>> {
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
        records.sort_by(|left, right| {
            (&left.id, left.kind, &left.reason).cmp(&(&right.id, right.kind, &right.reason))
        });
        Ok(records)
    }

    fn persist(&self, records: &[SuggestionRecord]) -> Result<()> {
        let path = self.path();
        if records.is_empty() {
            if self.fs.exists(&path) {
                self.fs.remove_file(&path)?;
            }
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            self.fs.create_dir_all(parent)?;
        }
        let mut out = String::new();
        for record in records {
            out.push_str(&json::encode(&to_value(record))?);
            out.push('\n');
        }
        self.fs.write_atomic(&path, out.as_bytes())
    }
}

fn group(id: &str, suggestions: &[Suggestion]) -> Vec<SuggestionRecord> {
    let mut groups: BTreeMap<(EdgeKind, String), Vec<String>> = BTreeMap::new();
    for suggestion in suggestions {
        groups
            .entry((suggestion.kind, suggestion.reason.clone()))
            .or_default()
            .push(suggestion.target.clone());
    }
    groups
        .into_iter()
        .map(|((kind, reason), mut targets)| {
            targets.sort();
            targets.dedup();
            SuggestionRecord {
                id: id.to_string(),
                kind,
                reason,
                targets,
            }
        })
        .collect()
}

fn to_value(record: &SuggestionRecord) -> Value {
    let targets = record
        .targets
        .iter()
        .map(|target| Value::Str(target.clone()))
        .collect();
    Value::map([
        ("id".to_string(), Value::Str(record.id.clone())),
        (
            "kind".to_string(),
            Value::Str(record.kind.as_str().to_string()),
        ),
        ("reason".to_string(), Value::Str(record.reason.clone())),
        ("targets".to_string(), Value::List(targets)),
    ])
}

fn parse_record(value: &Value) -> Option<SuggestionRecord> {
    let map = value.as_map()?;
    let id = map.get("id")?.as_str()?.to_string();
    let kind = map.get("kind")?.as_str()?.parse().ok()?;
    let reason = map
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let targets = match map.get("targets") {
        Some(Value::List(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    };
    Some(SuggestionRecord {
        id,
        kind,
        reason,
        targets,
    })
}
