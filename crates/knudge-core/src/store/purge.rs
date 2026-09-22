//! Purga do derivado em toda remoção (D84).
//!
//! Função única usada por supersede, merge, `compact`, TTL e dedup: remove o id das estruturas
//! derivadas em `.idx/` (JSONL e `index.json`). O vetor/índice é descartável; `doctor` (E09)
//! detecta divergência canônico↔derivado e orienta o rebuild.

use std::path::Path;

use indexmap::IndexMap;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

/// Remove `id` de toda estrutura derivada sob `<root>/.idx/`.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`InvalidInput` se leitura/escrita/parse falharem.
pub fn purge_derived(fs: &dyn Fs, root: &Path, id: &str) -> Result<()> {
    let idx = root.join(".idx");
    if !fs.exists(&idx) {
        return Ok(());
    }
    for path in fs.list_dir(&idx)? {
        match path.extension().and_then(|e| e.to_str()) {
            Some("jsonl") => purge_jsonl(fs, &path, id)?,
            Some("json") => purge_index(fs, &path, id)?,
            _ => {}
        }
    }
    Ok(())
}

fn purge_jsonl(fs: &dyn Fs, path: &Path, id: &str) -> Result<()> {
    let bytes = fs.read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
    let mut out = String::new();
    let mut changed = false;
    for line in jsonl::lines(text) {
        let Ok(mut value) = json::decode(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        let remove_record = value.as_map().is_some_and(|map| record_matches(map, id));
        if remove_record {
            changed = true;
            continue;
        }
        let before = value.clone();
        remove_id(&mut value, id);
        if value == before {
            out.push_str(line);
        } else {
            changed = true;
            out.push_str(&json::encode(&value)?);
        }
        out.push('\n');
    }
    if changed {
        fs.write_atomic(path, out.as_bytes())?;
    }
    Ok(())
}

fn purge_index(fs: &dyn Fs, path: &Path, id: &str) -> Result<()> {
    let bytes = fs.read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
    if text.trim().is_empty() {
        return Ok(());
    }
    let mut value = json::decode(text)?;
    let before = value.clone();
    remove_id(&mut value, id);
    if value != before {
        fs.write_atomic(path, json::encode(&value)?.as_bytes())?;
    }
    Ok(())
}

fn record_matches(map: &IndexMap<String, Value>, id: &str) -> bool {
    ["id", "note_id"]
        .iter()
        .any(|key| map.get(*key).and_then(Value::as_str) == Some(id))
}

fn remove_id(value: &mut Value, id: &str) {
    match value {
        Value::Map(map) => {
            map.shift_remove(id);
            for item in map.values_mut() {
                remove_id(item, id);
            }
        }
        Value::List(items) => {
            items.retain(|item| item.as_str() != Some(id));
            for item in &mut *items {
                remove_id(item, id);
            }
        }
        _ => {}
    }
}
