//! Uso (citações) por nota — derivado, `.idx/usage.jsonl` (D154).
//!
//! O uso é **índice**, nunca verdade: registrar quantas vezes uma nota foi devolvida pelo
//! `ask`/`rewind` não muda a nota nem o log de eventos (auditoria). O arquivo é purgado por
//! [`crate::store::purge_derived`] (D84) e pode ser reconstruído apenas parcialmente — a
//! história de uso não está em `notas/` (borda documentada).
//!
//! A renovação de shelf-life usa o **último uso** (`last_seen_ms`) e só **estende** retenção,
//! nunca encurta (invariante §8 do plano).

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

/// Uso acumulado de uma nota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    /// Id da nota.
    pub id: String,
    /// Número de citações creditadas.
    pub hits: u32,
    /// Instante (ms) do último uso creditado.
    pub last_seen_ms: i64,
}

/// Índice imutável `id -> last_seen_ms`, para consulta de expiração.
#[derive(Debug, Clone, Default)]
pub struct UsageIndex {
    last_seen: BTreeMap<String, i64>,
}

impl UsageIndex {
    /// Constrói o índice a partir das entradas de uso.
    #[must_use]
    pub fn new(entries: &[Usage]) -> Self {
        let mut last_seen = BTreeMap::new();
        for entry in entries {
            let slot = last_seen
                .entry(entry.id.clone())
                .or_insert(entry.last_seen_ms);
            *slot = (*slot).max(entry.last_seen_ms);
        }
        Self { last_seen }
    }

    /// Último uso creditado da nota, se houver.
    #[must_use]
    pub fn last_seen(&self, id: &str) -> Option<i64> {
        self.last_seen.get(id).copied()
    }

    /// `true` se não há nenhuma entrada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.last_seen.is_empty()
    }
}

/// Mescla citações num lote (puro).
///
/// `cited` é deduplicado; uma citação em `now_ms` quando o último uso já é `now_ms` **não**
/// incrementa de novo (idempotência por `(id, now)`). O resultado é ordenado por `id`.
#[must_use]
pub fn record_usage(existing: &[Usage], cited: &[String], now_ms: i64) -> Vec<Usage> {
    let mut map: BTreeMap<String, Usage> = BTreeMap::new();
    for entry in existing {
        let slot = map.entry(entry.id.clone()).or_insert_with(|| Usage {
            id: entry.id.clone(),
            hits: 0,
            last_seen_ms: entry.last_seen_ms,
        });
        slot.hits = slot.hits.max(entry.hits);
        slot.last_seen_ms = slot.last_seen_ms.max(entry.last_seen_ms);
    }
    for id in cited {
        let slot = map.entry(id.clone()).or_insert_with(|| Usage {
            id: id.clone(),
            hits: 0,
            last_seen_ms: now_ms,
        });
        if slot.last_seen_ms != now_ms {
            slot.hits = slot.hits.saturating_add(1);
        }
        slot.last_seen_ms = slot.last_seen_ms.max(now_ms);
    }
    map.into_values().collect()
}

/// Store derivado de uso (`.idx/usage.jsonl`).
pub struct UsageStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> UsageStore<'a> {
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
        self.root.join(".idx").join("usage.jsonl")
    }

    /// Lê as entradas de uso, ordenadas por `id`.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se o arquivo existir e não for legível.
    pub fn load(&self) -> Result<Vec<Usage>> {
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
        entries.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(entries)
    }

    /// Índice de consulta a partir do arquivo (vazio se ausente).
    ///
    /// # Errors
    /// Propaga erro de leitura.
    pub fn index(&self) -> Result<UsageIndex> {
        Ok(UsageIndex::new(&self.load()?))
    }

    /// Credita `cited` em `now_ms` e persiste (coalescido; uma escrita por invocação).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` em falha de leitura/escrita.
    pub fn record(&self, cited: &[String], now_ms: i64) -> Result<Vec<Usage>> {
        let existing = self.load()?;
        let merged = record_usage(&existing, cited, now_ms);
        self.persist(&merged)?;
        Ok(merged)
    }

    fn persist(&self, entries: &[Usage]) -> Result<()> {
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

fn to_value(entry: &Usage) -> Value {
    Value::map([
        ("id".to_string(), Value::Str(entry.id.clone())),
        ("hits".to_string(), Value::Int(i64::from(entry.hits))),
        ("last_seen_ms".to_string(), Value::Int(entry.last_seen_ms)),
    ])
}

fn parse_entry(value: &Value) -> Option<Usage> {
    let map = value.as_map()?;
    let id = map.get("id")?.as_str()?.to_string();
    let hits = map
        .get("hits")
        .and_then(Value::as_int)
        .unwrap_or(0)
        .clamp(0, i64::from(u32::MAX));
    let last_seen_ms = map.get("last_seen_ms").and_then(Value::as_int).unwrap_or(0);
    let hits = u32::try_from(hits).unwrap_or(0);
    Some(Usage {
        id,
        hits,
        last_seen_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::fakes::MemFs;

    fn entry(id: &str, hits: u32, last: i64) -> Usage {
        Usage {
            id: id.to_string(),
            hits,
            last_seen_ms: last,
        }
    }

    #[test]
    fn record_increments_and_extends() {
        let out = record_usage(&[entry("a", 1, 10)], &["a".to_string()], 20);
        assert_eq!(out, vec![entry("a", 2, 20)]);
    }

    #[test]
    fn record_is_idempotent_for_same_instant() {
        let first = record_usage(&[], &["a".to_string()], 20);
        let second = record_usage(&first, &["a".to_string()], 20);
        assert_eq!(first, second);
    }

    #[test]
    fn record_never_shortens_last_seen() {
        let out = record_usage(&[entry("a", 3, 100)], &["a".to_string()], 20);
        assert_eq!(out.first().map(|u| u.last_seen_ms), Some(100));
    }

    #[test]
    fn record_is_order_independent() {
        let a = record_usage(&[entry("a", 1, 5)], &["b".to_string()], 9);
        let b = record_usage(&[entry("a", 1, 5)], &["b".to_string()], 9);
        assert_eq!(a, b);
    }

    #[test]
    fn store_round_trips_and_indexes() -> Result<()> {
        let fs = MemFs::default();
        let store = UsageStore::new(&fs, "/k");
        store.record(&["a".to_string(), "b".to_string()], 42)?;
        store.record(&["a".to_string()], 43)?;
        let index = store.index()?;
        assert_eq!(index.last_seen("a"), Some(43));
        assert_eq!(index.last_seen("b"), Some(42));
        assert_eq!(index.last_seen("c"), None);
        Ok(())
    }
}
