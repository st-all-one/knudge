//! Rascunho de nota: campos tipados que viram um frontmatter válido (D05/D16/D17).
//!
//! A escrita é **estrita na forma**: chaves desconhecidas são rejeitadas por
//! [`crate::schema::Frontmatter::set`] e opcionais vazios são **omitidos** (nunca `null`).

use indexmap::IndexMap;

use crate::graph;
use crate::schema::{
    Classification, EdgeKind, Frontmatter, NoteType, SCHEMA_VERSION, Scope, Status, Value, id,
};
use crate::store::Note;
use crate::time::Timestamp;
use crate::{Error, Result};

use super::{set_list, validate_anchors};

/// Especificação de uma nota nova.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Tipo fechado.
    pub note_type: NoteType,
    /// Afirmação (≤ 120 escalares).
    pub statement: String,
    /// Corpo (contexto).
    pub body: String,
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Proveniência.
    pub source: Option<String>,
    /// Arestas explícitas `(tipo, destino)`.
    pub edges: Vec<(EdgeKind, String)>,
    /// Âncoras (paths/globs).
    pub anchors: Vec<String>,
    /// Maturidade (default `tactical`).
    pub classification: Option<Classification>,
    /// Estado (default `active`).
    pub status: Option<Status>,
    /// Escopo (só `task`/`container` — D93).
    pub scope: Option<Scope>,
    /// Validators (só `task`).
    pub checks: Vec<String>,
    /// Evidências de fechamento (mapa).
    pub evidence: Vec<(String, Value)>,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            note_type: NoteType::Fact,
            statement: String::new(),
            body: String::new(),
            tags: Vec::new(),
            source: None,
            edges: Vec::new(),
            anchors: Vec::new(),
            classification: None,
            status: None,
            scope: None,
            checks: Vec::new(),
            evidence: Vec::new(),
        }
    }
}

impl Draft {
    /// Rascunho com tipo e afirmação.
    #[must_use]
    pub fn new(note_type: NoteType, statement: impl Into<String>) -> Self {
        Self {
            note_type,
            statement: statement.into(),
            ..Self::default()
        }
    }

    /// Define o corpo.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Lê um rascunho de um objeto JSON (K4/D110).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema`/`InvalidInput` para chave desconhecida, tipo inválido ou
    /// `statement` ausente.
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value
            .as_map()
            .ok_or_else(|| Error::schema("rascunho deve ser um objeto JSON"))?;
        for key in map.keys() {
            if !DRAFT_KEYS.contains(&key.as_str()) {
                return Err(Error::schema(format!(
                    "chave de rascunho desconhecida: {key}"
                )));
            }
        }
        let mut draft = Self::default();
        if let Some(note_type) = map.get("type").and_then(Value::as_str) {
            draft.note_type = note_type.parse()?;
        }
        if let Some(statement) = map.get("statement").and_then(Value::as_str) {
            draft.statement = statement.to_string();
        }
        if draft.statement.is_empty() {
            return Err(Error::invalid_input("rascunho sem `statement`"));
        }
        if let Some(body) = map.get("body").and_then(Value::as_str) {
            draft.body = body.to_string();
        }
        if let Some(source) = map.get("source").and_then(Value::as_str) {
            draft.source = Some(source.to_string());
        }
        if let Some(class) = map.get("classification").and_then(Value::as_str) {
            draft.classification = Some(class.parse()?);
        }
        if let Some(status) = map.get("status").and_then(Value::as_str) {
            draft.status = Some(status.parse()?);
        }
        draft.tags = string_list(map.get("tags"))?;
        draft.anchors = string_list(map.get("anchors"))?;
        Ok(draft)
    }

    /// Texto usado no dedup (afirmação + corpo).
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = self.statement.clone();
        if !self.body.is_empty() {
            text.push(' ');
            text.push_str(&self.body);
        }
        text
    }

    /// Converte em nota válida, omitindo opcionais vazios.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema`/`InvalidInput` para chave/tipo/aresta/escopo inválidos ou
    /// `statement` acima do limite.
    pub fn to_note(&self, now_ms: i64) -> Result<Note> {
        if self.statement.trim().is_empty() {
            return Err(Error::invalid_input(
                "statement vazio: informe a afirmação da nota",
            ));
        }
        let mut frontmatter = Frontmatter::new();
        frontmatter.set(
            "id",
            Value::Str(id::note_id(self.note_type, &self.statement)),
        )?;
        if !self.note_type.is_group() {
            frontmatter.set("type", Value::Str(self.note_type.as_str().to_string()))?;
        }
        frontmatter.set("statement", Value::Str(self.statement.clone()))?;
        frontmatter.set(
            "created_at",
            Value::Str(Timestamp::from_millis(now_ms).to_rfc3339()),
        )?;
        frontmatter.set("schema_version", Value::Int(i64::from(SCHEMA_VERSION)))?;
        set_list(&mut frontmatter, "tags", &self.tags)?;
        if let Some(source) = &self.source {
            frontmatter.set("source", Value::Str(source.clone()))?;
        }
        for (kind, to) in &self.edges {
            graph::link(&mut frontmatter, *kind, to)?;
        }
        validate_anchors(&self.anchors)?;
        set_list(&mut frontmatter, "anchors", &self.anchors)?;
        if let Some(class) = self.classification {
            frontmatter.set("classification", Value::Str(class.as_str().to_string()))?;
        }
        if let Some(status) = self.status {
            frontmatter.set("status", Value::Str(status.as_str().to_string()))?;
        }
        self.validate_scope()?;
        if let Some(scope) = self.scope {
            frontmatter.set("scope", Value::Str(scope.as_str().to_string()))?;
        }
        set_list(&mut frontmatter, "checks", &self.checks)?;
        if !self.evidence.is_empty() {
            let mut map = IndexMap::new();
            for (key, value) in &self.evidence {
                map.insert(key.clone(), value.clone());
            }
            frontmatter.set("evidence", Value::Map(map))?;
        }
        let mut note = Note::new(frontmatter, self.body.clone());
        note.refresh_body_hash()?;
        note.frontmatter.validate()?;
        Ok(note)
    }

    fn validate_scope(&self) -> Result<()> {
        if self.scope.is_some() && !self.note_type.is_scoped() {
            return Err(Error::schema(
                "scope só vale para item de trabalho/container (D93/D113)",
            ));
        }
        if self.note_type.requires_scope() && self.scope.is_none() {
            return Err(Error::schema(
                "`type` de trabalho/container exige scope (D93/D113)",
            ));
        }
        Ok(())
    }
}

/// Chaves aceitas num rascunho JSONL (K4/D110).
const DRAFT_KEYS: [&str; 8] = [
    "type",
    "statement",
    "body",
    "tags",
    "anchors",
    "source",
    "classification",
    "status",
];

fn string_list(value: Option<&Value>) -> Result<Vec<String>> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::List(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| Error::schema("lista deve conter só strings"))
            })
            .collect(),
        Some(_) => Err(Error::schema("valor deve ser lista de strings")),
    }
}
