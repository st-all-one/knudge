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

use super::set_list;

/// Especificação de uma nota nova.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Tipo fechado.
    pub note_type: NoteType,
    /// Afirmação (≤ 120 escalares).
    pub statement: String,
    /// Corpo (contexto).
    pub body: String,
    /// Confiança `0..=1`.
    pub confidence: f64,
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Proveniência.
    pub source: Option<String>,
    /// Expiração (ms desde a época).
    pub expires_at: Option<i64>,
    /// Agendamento `not_before` (ms desde a época) — separado da expiração (D56).
    pub not_before: Option<i64>,
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
            confidence: 0.7,
            tags: Vec::new(),
            source: None,
            expires_at: None,
            not_before: None,
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
        let mut frontmatter = Frontmatter::new();
        frontmatter.set(
            "id",
            Value::Str(id::note_id(self.note_type, &self.statement)),
        )?;
        frontmatter.set("type", Value::Str(self.note_type.as_str().to_string()))?;
        frontmatter.set("statement", Value::Str(self.statement.clone()))?;
        frontmatter.set(
            "created_at",
            Value::Str(Timestamp::from_millis(now_ms).to_rfc3339()),
        )?;
        frontmatter.set("confidence", Value::Float(self.confidence))?;
        frontmatter.set("schema_version", Value::Int(i64::from(SCHEMA_VERSION)))?;
        set_list(&mut frontmatter, "tags", &self.tags)?;
        if let Some(source) = &self.source {
            frontmatter.set("source", Value::Str(source.clone()))?;
        }
        if let Some(expires) = self.expires_at {
            frontmatter.set(
                "expires_at",
                Value::Str(Timestamp::from_millis(expires).to_rfc3339()),
            )?;
        }
        if let Some(not_before) = self.not_before {
            frontmatter.set(
                "not_before",
                Value::Str(Timestamp::from_millis(not_before).to_rfc3339()),
            )?;
        }
        for (kind, to) in &self.edges {
            graph::link(&mut frontmatter, *kind, to)?;
        }
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
        let scoped = matches!(self.note_type, NoteType::Task | NoteType::Container);
        match (self.scope.is_some(), scoped) {
            (true, true) | (false, false) => Ok(()),
            (true, false) => Err(Error::schema(
                "scope só vale para `type` task/container (D93)",
            )),
            (false, true) => Err(Error::schema("`type` task/container exige scope (D93)")),
        }
    }
}
