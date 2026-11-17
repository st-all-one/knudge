//! Propostas de manutenção — derivado `.idx/proposals.jsonl` (D230).
//!
//! O `prune` persiste aqui o plano de demolição para que o handoff mostre quantas propostas
//! ficaram pendentes. É índice, nunca verdade: não altera notas nem eventos, é reconstruível e
//! é purgado por [`crate::store::purge_derived`] (D84).

use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

use super::plan::{DemotionAction, DemotionReason};

/// Uma proposta persistida do `prune`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalEntry {
    /// Id da nota.
    pub id: String,
    /// Motivo da proposta.
    pub reason: DemotionReason,
    /// Ação sugerida (`forget` ou `review`).
    pub action: DemotionAction,
}

/// Store derivado de propostas (`.idx/proposals.jsonl`).
pub struct ProposalStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> ProposalStore<'a> {
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
        self.root.join(".idx").join("proposals.jsonl")
    }

    /// Lê as propostas, ordenadas por `id` (vazio se o arquivo não existir).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se o arquivo existir e não for legível.
    pub fn load(&self) -> Result<Vec<ProposalEntry>> {
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

    /// Conta propostas válidas sem alocar/ordenar a lista que `load` devolve (D230).
    ///
    /// # Errors
    /// Propaga erro de leitura ou UTF-8 inválido.
    pub fn count(&self) -> Result<usize> {
        let path = self.path();
        if !self.fs.exists(&path) {
            return Ok(0);
        }
        let bytes = self.fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        let mut count = 0_usize;
        for line in jsonl::lines(text) {
            if let Ok(value) = json::decode(line)
                && parse_entry(&value).is_some()
            {
                count = count.saturating_add(1);
            }
        }
        Ok(count)
    }

    /// Persiste as propostas — arquivo vazio limpa o plano anterior.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` em falha de escrita.
    pub fn persist(&self, entries: &[ProposalEntry]) -> Result<()> {
        let path = self.path();
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

fn to_value(entry: &ProposalEntry) -> Value {
    Value::map([
        ("id".to_string(), Value::Str(entry.id.clone())),
        (
            "reason".to_string(),
            Value::Str(entry.reason.as_str().to_string()),
        ),
        (
            "action".to_string(),
            Value::Str(entry.action.as_str().to_string()),
        ),
    ])
}

fn parse_entry(value: &Value) -> Option<ProposalEntry> {
    let map = value.as_map()?;
    let id = map.get("id")?.as_str()?.to_string();
    let reason = match map.get("reason")?.as_str()? {
        "expired" => DemotionReason::Expired,
        "anchor_decay" => DemotionReason::AnchorDecay,
        "contradicted" => DemotionReason::Contradicted,
        "defeated" => DemotionReason::Defeated,
        "drifted" => DemotionReason::Drifted,
        _ => return None,
    };
    let action = match map.get("action")?.as_str()? {
        "forget" => DemotionAction::Forget,
        "review" => DemotionAction::Review,
        _ => return None,
    };
    Some(ProposalEntry { id, reason, action })
}
