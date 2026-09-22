//! Índice derivado do retrieval (E06-T01).
//!
//! O índice é **derivado** de `notas/`: reconstruível byte a byte e descartável (D15/D27). Vive
//! em `.idx/retrieval.jsonl` (uma linha JSON por nota) e a troca é atômica via `tmp + rename`.
//! As estatísticas (df/avgdl) são **recomputadas ao carregar**, então nunca divergem do corpo.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::retrieval::filter::Meta;
use crate::retrieval::token::tokenize;
use crate::schema::{Classification, NoteType, Status, Value};
use crate::store::{Note, Store};
use crate::{Error, Result};

/// Nome do arquivo do índice dentro de `.idx/`.
pub const INDEX_FILE: &str = "retrieval.jsonl";

/// Acima deste tamanho (bytes) o índice emite aviso (E06-T07).
pub const INDEX_WARN_BYTES: u64 = 8_388_608;

/// Campo indexado (IDF é calculado por campo — D37).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    /// `statement` — o campo que domina.
    Statement,
    /// Corpo.
    Body,
    /// Tags.
    Tags,
}

impl Field {
    /// Todos os campos, em ordem canônica.
    pub const ALL: [Self; 3] = [Self::Statement, Self::Body, Self::Tags];

    /// Rótulo de persistência.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Statement => "statement",
            Self::Body => "body",
            Self::Tags => "tags",
        }
    }

    /// Peso do campo na soma BM25 (`statement` domina — D37).
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Self::Statement => 3.0,
            Self::Body => 1.0,
            Self::Tags => 2.0,
        }
    }

    /// Interpreta o rótulo de persistência.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.as_str() == text)
    }
}

/// Frequências de termos de um campo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldTf {
    /// termo → frequência.
    pub tf: BTreeMap<String, u32>,
    /// Número de tokens do campo.
    pub len: u32,
}

/// Documento indexado.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteDoc {
    /// Metadados (filtros, `why`, boost).
    pub meta: Meta,
    /// `statement` (para a saída do `recall`).
    pub statement: String,
    /// Campos e suas frequências.
    pub fields: BTreeMap<Field, FieldTf>,
}

impl NoteDoc {
    /// Indexa uma nota válida.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se o frontmatter estiver malformado.
    pub fn from_note(note: &Note) -> Result<Self> {
        let meta = Meta::from_frontmatter(&note.frontmatter)?;
        let statement = note.frontmatter.statement()?.to_string();
        let tags = meta.tags.join(" ");
        let mut fields = BTreeMap::new();
        fields.insert(Field::Statement, field_tf(&statement));
        fields.insert(Field::Body, field_tf(&note.body));
        fields.insert(Field::Tags, field_tf(&tags));
        Ok(Self {
            meta,
            statement,
            fields,
        })
    }

    /// Frequência do termo no campo (0 se ausente).
    #[must_use]
    pub fn tf(&self, field: Field, term: &str) -> u32 {
        self.fields
            .get(&field)
            .and_then(|entry| entry.tf.get(term))
            .copied()
            .unwrap_or(0)
    }

    /// Número de tokens do campo.
    #[must_use]
    pub fn len(&self, field: Field) -> u32 {
        self.fields.get(&field).map_or(0, |entry| entry.len)
    }
}

/// Estatísticas globais do corpus (recomputadas ao carregar).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    /// Número de documentos.
    pub n: u32,
    /// Comprimento médio por campo.
    pub avg_len: BTreeMap<Field, f64>,
    /// Document frequency por campo e termo.
    pub df: BTreeMap<Field, BTreeMap<String, u32>>,
}

/// Índice de retrieval.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Index {
    /// Documentos, ordenados por id.
    pub docs: Vec<NoteDoc>,
    /// Estatísticas globais.
    pub stats: Stats,
}

impl Index {
    /// Constrói o índice a partir de notas já parseadas.
    ///
    /// # Errors
    /// Propaga erros de extração de metadados.
    pub fn build(notes: &[Note]) -> Result<Self> {
        let mut docs = Vec::new();
        let _reserved = docs.try_reserve(notes.len());
        for note in notes {
            docs.push(NoteDoc::from_note(note)?);
        }
        docs.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));
        let stats = compute_stats(&docs);
        Ok(Self { docs, stats })
    }

    /// Constrói o índice lendo todas as notas do store.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse.
    pub fn from_store(store: &Store<'_>) -> Result<Self> {
        let mut notes = Vec::new();
        for id in store.list_ids()? {
            notes.push(store.read(&id)?);
        }
        Self::build(&notes)
    }

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
        let limit = usize::try_from(INDEX_WARN_BYTES).unwrap_or(usize::MAX);
        if data.len() > limit {
            warnings.push(format!(
                "índice de {} bytes acima do teto de {}; considere rebuild",
                data.len(),
                INDEX_WARN_BYTES
            ));
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
        let limit = usize::try_from(INDEX_WARN_BYTES).unwrap_or(usize::MAX);
        if bytes.len() > limit {
            warnings.push(format!(
                "índice de {} bytes acima do teto de {}; considere rebuild",
                bytes.len(),
                INDEX_WARN_BYTES
            ));
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

fn field_tf(text: &str) -> FieldTf {
    let mut tf: BTreeMap<String, u32> = BTreeMap::new();
    let mut len = 0_u32;
    for token in tokenize(text) {
        len = len.saturating_add(1);
        let entry = tf.entry(token.into_owned()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    FieldTf { tf, len }
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "médias do corpus em f64; somas e `n` não negativos"
)]
fn compute_stats(docs: &[NoteDoc]) -> Stats {
    let n = u32::try_from(docs.len()).unwrap_or(u32::MAX);
    let mut total: BTreeMap<Field, f64> = BTreeMap::new();
    let mut df: BTreeMap<Field, BTreeMap<String, u32>> = BTreeMap::new();
    for doc in docs {
        for field in Field::ALL {
            *total.entry(field).or_insert(0.0) += f64::from(doc.len(field));
            if let Some(entry) = doc.fields.get(&field) {
                for term in entry.tf.keys() {
                    let count = df.entry(field).or_default().entry(term.clone()).or_insert(0);
                    *count = count.saturating_add(1);
                }
            }
        }
    }
    let mut avg_len = BTreeMap::new();
    for field in Field::ALL {
        let sum = total.get(&field).copied().unwrap_or(0.0);
        let avg = if n == 0 { 0.0 } else { sum / f64::from(n) };
        avg_len.insert(field, avg);
    }
    Stats { n, avg_len, df }
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
        confirmation: map.get("confirmation").and_then(Value::as_f64).unwrap_or(0.0),
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
