//! Drift de âncoras — derivado, `.idx/drift.jsonl` (D203, absorção de E16/T05/D174).
//!
//! Drift = fração de âncoras **quebradas** de uma nota (`1 - AnchorValidity::fraction`). É
//! computado **off-path** (varredura única do projeto) e persistido como derivado (D84); a
//! confiança derivada desconta o score por [`crate::lifecycle::confidence::drift_factor`] (D203).
//! Arquivo ausente ⇒ `drift = 0` (degradação graciosa, R33).
//!
//! O uso é **índice**, nunca verdade: o drift não muda a nota nem o log de eventos. O arquivo é
//! purgado por [`crate::store::purge_derived`] (D84) e é reconstruível a partir de `notas/` + a
//! árvore do projeto.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::lifecycle::decay::AnchorValidity;
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

/// Drift de âncoras de uma nota.
#[derive(Debug, Clone, PartialEq)]
pub struct DriftEntry {
    /// Id da nota.
    pub id: String,
    /// Drift em `[0,1]` (0 = âncoras em dia; 1 = todas quebradas).
    pub drift: f64,
}

/// Índice imutável `id -> drift`, para consulta de confiança.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DriftIndex {
    drift: BTreeMap<String, f64>,
}

impl DriftIndex {
    /// Constrói o índice a partir das entradas (última entrada vence).
    #[must_use]
    pub fn new(entries: &[DriftEntry]) -> Self {
        let drift = entries
            .iter()
            .map(|entry| (entry.id.clone(), entry.drift))
            .collect();
        Self { drift }
    }

    /// Drift da nota (`0.0` se ausente).
    #[must_use]
    pub fn get(&self, id: &str) -> f64 {
        self.drift.get(id).copied().unwrap_or(0.0)
    }

    /// `true` se não há nenhuma entrada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.drift.is_empty()
    }
}

/// Converte a validade de âncoras em entradas de drift (puro).
///
/// Drift = `1 - AnchorValidity::fraction` (0 quando não há âncoras). Ordenado por `id`.
#[must_use]
pub fn entries_from_validity(validity: &BTreeMap<String, AnchorValidity>) -> Vec<DriftEntry> {
    validity
        .iter()
        .map(|(id, value)| DriftEntry {
            id: id.clone(),
            drift: (1.0 - value.fraction()).clamp(0.0, 1.0),
        })
        .collect()
}

/// Store derivado de drift (`.idx/drift.jsonl`).
pub struct DriftStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> DriftStore<'a> {
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
        self.root.join(".idx").join("drift.jsonl")
    }

    /// Lê as entradas de drift, ordenadas por `id` (vazio se o arquivo não existir).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se o arquivo existir e não for legível.
    pub fn load(&self) -> Result<Vec<DriftEntry>> {
        let path = self.path();
        if !self.fs.exists(&path) {
            return Ok(Vec::new());
        }
        let bytes = self.fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        let mut entries = Vec::new();
        for line in jsonl::lines(text) {
            if let Ok(value) = json::decode(line)
                && let Some(entry) = parse_entry(&value)
            {
                entries.push(entry);
            }
        }
        entries.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        Ok(entries)
    }

    /// Índice de consulta a partir do arquivo (vazio se ausente).
    ///
    /// # Errors
    /// Propaga erro de leitura.
    pub fn index(&self) -> Result<DriftIndex> {
        Ok(DriftIndex::new(&self.load()?))
    }

    /// Persiste as entradas (uma escrita por invocação).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` em falha de escrita.
    pub fn persist(&self, entries: &[DriftEntry]) -> Result<()> {
        let path = self.path();
        if entries.is_empty() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            self.fs.create_dir_all(parent)?;
        }
        let mut out = String::new();
        for entry in entries {
            out.push_str(&json::encode(&to_value(entry))?);
            out.push('\n');
        }
        self.fs.write_atomic(&path, out.as_bytes())
    }
}

fn to_value(entry: &DriftEntry) -> Value {
    Value::map([
        ("id".to_string(), Value::Str(entry.id.clone())),
        ("drift".to_string(), Value::Float(entry.drift)),
    ])
}

fn parse_entry(value: &Value) -> Option<DriftEntry> {
    let map = value.as_map()?;
    let id = map.get("id")?.as_str()?.to_string();
    let drift = map.get("drift").and_then(Value::as_f64).unwrap_or(0.0);
    Some(DriftEntry {
        id,
        drift: drift.clamp(0.0, 1.0),
    })
}
