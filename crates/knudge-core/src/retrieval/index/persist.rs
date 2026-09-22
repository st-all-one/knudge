//! Persistência do índice derivado: serialização canônica e carga tolerante (E06-T01).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::retrieval::filter::Meta;
use crate::schema::{Classification, NoteType, Status, Value};
use crate::store::Store;
use crate::{Error, Result};

use super::{Field, FieldTf, INDEX_FILE, INDEX_WARN_BYTES, Index, NoteDoc, compute_stats};

impl Index {
    /// Caminho do arquivo do índice.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(".idx").join(INDEX_FILE)
    }

    /// Serializa o índice em JSONL canônico, uma linha por documento.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum float não for finito.
    pub fn serialize(&self) -> Result<String> {
        let mut out = String::new();
        for doc in &self.docs {
            out.push_str(&json::encode(&doc_to_value(doc))?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Lê o índice de UM texto JSONL e recomputa as estatísticas.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput`/`Schema` para linha malformada.
    pub fn parse(text: &str) -> Result<Self> {
        let mut docs = Vec::new();
        for line in jsonl::lines(text) {
            docs.push(doc_from_value(&json::decode(line)?)?);
        }
        docs.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));
        let stats = compute_stats(&docs);
        Ok(Self { docs, stats })
    }

    /// Grava o índice atomicamente (`tmp + rename`), avisando se ultrapassar o teto.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, fs: &dyn Fs, root: &Path, warnings: &mut Vec<String>) -> Result<()> {
        let path = Self::path(root);
        if let Some(parent) = path.parent() {
            fs.create_dir_all(parent)?;
        }
        let data = self.serialize()?;
        if let Some(warning) = size_warning(data.len()) {
            warnings.push(warning);
        }
        fs.write_atomic(&path, data.as_bytes())
    }

    /// Carrega o índice, se existir e for legível.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se a leitura/parse falhar.
    pub fn load(fs: &dyn Fs, root: &Path, warnings: &mut Vec<String>) -> Result<Option<Self>> {
        let path = Self::path(root);
        if !fs.exists(&path) {
            return Ok(None);
        }
        let bytes = fs.read(&path)?;
        if let Some(warning) = size_warning(bytes.len()) {
            warnings.push(warning);
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        if text.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(Self::parse(text)?))
    }

    /// Abre o índice, reconstruindo e gravando se estiver ausente.
    ///
    /// # Errors
    /// Propaga erros de leitura/parse/construção/escrita.
    pub fn open(fs: &dyn Fs, root: &Path, store: &Store<'_>) -> Result<(Self, Vec<String>)> {
        let mut warnings = Vec::new();
        if let Some(index) = Self::load(fs, root, &mut warnings)? {
            return Ok((index, warnings));
        }
        let index = Self::from_store(store)?;
        index.save(fs, root, &mut warnings)?;
        warnings.push("índice ausente: reconstruído a partir de notas/".to_string());
        Ok((index, warnings))
    }
}
fn doc_to_value(doc: &NoteDoc) -> Value {
    let mut map = IndexMap::new();
    map.insert("id".to_string(), Value::Str(doc.meta.id.clone()));
    map.insert(
        "type".to_string(),
        Value::Str(doc.meta.note_type.as_str().to_string()),
    );
    map.insert(
        "classification".to_string(),
        Value::Str(doc.meta.classification.as_str().to_string()),
    );
    map.insert(
        "status".to_string(),
        Value::Str(doc.meta.status.as_str().to_string()),
    );
    map.insert("tags".to_string(), string_list(&doc.meta.tags));
    map.insert("anchors".to_string(), string_list(&doc.meta.anchors));
    map.insert("created".to_string(), Value::Int(doc.meta.created_ms));
    map.insert(
        "confirmation".to_string(),
        Value::Float(doc.meta.confirmation),
    );
    map.insert("statement".to_string(), Value::Str(doc.statement.clone()));
    let mut fields = IndexMap::new();
    for field in Field::ALL {
        if let Some(entry) = doc.fields.get(&field) {
            let mut tf = IndexMap::new();
            for (term, count) in &entry.tf {
                tf.insert(term.clone(), Value::Int(i64::from(*count)));
            }
            let mut value = IndexMap::new();
            value.insert("len".to_string(), Value::Int(i64::from(entry.len)));
            value.insert("tf".to_string(), Value::Map(tf));
            fields.insert(field.as_str().to_string(), Value::Map(value));
        }
    }
    map.insert("fields".to_string(), Value::Map(fields));
    Value::Map(map)
}

fn doc_from_value(value: &Value) -> Result<NoteDoc> {
    let map = value
        .as_map()
        .ok_or_else(|| Error::invalid_input("linha de índice não é objeto"))?;
    let meta = Meta {
        id: str_field(map, "id")?.to_string(),
        note_type: str_field(map, "type")?.parse::<NoteType>()?,
        classification: str_field(map, "classification")?.parse::<Classification>()?,
        status: str_field(map, "status")?.parse::<Status>()?,
        tags: str_list(map, "tags"),
        anchors: str_list(map, "anchors"),
        created_ms: int_field(map, "created")?,
        confirmation: map
            .get("confirmation")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    };
    let mut fields = BTreeMap::new();
    if let Some(Value::Map(fields_map)) = map.get("fields") {
        for (key, item) in fields_map {
            let Some(field) = Field::parse(key) else {
                continue;
            };
            let entry = item
                .as_map()
                .ok_or_else(|| Error::schema("campo do índice não é objeto"))?;
            let len = u32::try_from(int_field(entry, "len")?).unwrap_or(u32::MAX);
            let mut tf = BTreeMap::new();
            if let Some(Value::Map(terms)) = entry.get("tf") {
                for (term, count) in terms {
                    let count = count.as_int().unwrap_or(0);
                    tf.insert(term.clone(), u32::try_from(count).unwrap_or(u32::MAX));
                }
            }
            fields.insert(field, FieldTf { tf, len });
        }
    }
    Ok(NoteDoc {
        meta,
        statement: str_field(map, "statement")?.to_string(),
        fields,
    })
}

fn str_field<'a>(map: &'a IndexMap<String, Value>, key: &str) -> Result<&'a str> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::schema(format!("campo `{key}` ausente no índice")))
}

fn int_field(map: &IndexMap<String, Value>, key: &str) -> Result<i64> {
    map.get(key)
        .and_then(Value::as_int)
        .ok_or_else(|| Error::schema(format!("campo `{key}` ausente no índice")))
}

fn str_list(map: &IndexMap<String, Value>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(Value::List(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn string_list(items: &[String]) -> Value {
    Value::List(items.iter().map(|item| Value::Str(item.clone())).collect())
}

/// Aviso quando o índice passa do teto de [`INDEX_WARN_BYTES`] (E06-T07).
#[must_use]
pub fn size_warning(len: usize) -> Option<String> {
    let limit = usize::try_from(INDEX_WARN_BYTES).unwrap_or(usize::MAX);
    (len > limit).then(|| {
        format!("índice de {len} bytes acima do teto de {INDEX_WARN_BYTES}; considere rebuild")
    })
}
