//! Patch de `update`: campos mutáveis e leitura JSON (`--update --params`, D147).

use crate::schema::{Classification, NoteType, Scope, Status, Value};
use crate::store::Note;
use crate::write::status::validate_transition;
use crate::write::{set_list, validate_anchors};
use crate::{Error, Result};

use super::validate_scope;

/// Campos mutáveis por [`super::update`] (ausente = não mexe).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Patch {
    /// Novo tipo (mudança dispara supersede).
    pub note_type: Option<NoteType>,
    /// Nova afirmação (mudança dispara supersede).
    pub statement: Option<String>,
    /// Novo corpo.
    pub body: Option<String>,
    /// Novas tags (vazio limpa).
    pub tags: Option<Vec<String>>,
    /// Nova classificação.
    pub classification: Option<Classification>,
    /// Novo status (validado por [`validate_transition`]).
    pub status: Option<Status>,
    /// Novo escopo (só `task`/`container`).
    pub scope: Option<Scope>,
    /// Novas âncoras (vazio limpa).
    pub anchors: Option<Vec<String>>,
}

impl Patch {
    /// Aplica o patch a uma nota (sem gravar).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema`/`InvalidInput` para valor ou transição inválidos.
    pub fn apply(&self, note: &mut Note) -> Result<()> {
        if let Some(note_type) = self.note_type {
            note.frontmatter
                .set("type", Value::Str(note_type.as_str().to_string()))?;
        }
        if let Some(statement) = &self.statement {
            if statement.trim().is_empty() {
                return Err(Error::invalid_input(
                    "statement vazio: informe a afirmação da nota",
                ));
            }
            note.frontmatter
                .set("statement", Value::Str(statement.clone()))?;
        }
        if let Some(body) = &self.body {
            note.body.clone_from(body);
        }
        if let Some(tags) = &self.tags {
            set_list(&mut note.frontmatter, "tags", tags)?;
        }
        if let Some(class) = self.classification {
            note.frontmatter
                .set("classification", Value::Str(class.as_str().to_string()))?;
        }
        if let Some(status) = self.status {
            validate_transition(note.frontmatter.status()?, status)?;
            note.frontmatter
                .set("status", Value::Str(status.as_str().to_string()))?;
        }
        if let Some(scope) = self.scope {
            validate_scope(note.frontmatter.note_type()?, Some(scope))?;
            note.frontmatter
                .set("scope", Value::Str(scope.as_str().to_string()))?;
        }
        if let Some(anchors) = &self.anchors {
            validate_anchors(anchors)?;
            set_list(&mut note.frontmatter, "anchors", anchors)?;
        }
        Ok(())
    }

    /// Lê um patch de um objeto JSON (`--update --params`, D147).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema`/`InvalidInput` para chave desconhecida ou valor inválido.
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value
            .as_map()
            .ok_or_else(|| Error::schema("patch deve ser um objeto JSON"))?;
        for key in map.keys() {
            if !PATCH_KEYS.contains(&key.as_str()) {
                return Err(Error::schema(format!("chave de patch desconhecida: {key}")));
            }
        }
        let mut patch = Self::default();
        if let Some(note_type) = map.get("type").and_then(Value::as_str) {
            patch.note_type = Some(note_type.parse()?);
        }
        if let Some(statement) = map.get("statement").and_then(Value::as_str) {
            patch.statement = Some(statement.to_string());
        }
        if let Some(body) = map.get("body").and_then(Value::as_str) {
            patch.body = Some(body.to_string());
        }
        if let Some(tags) = map.get("tags") {
            patch.tags = Some(string_list(tags)?);
        }
        if let Some(anchors) = map.get("anchors") {
            patch.anchors = Some(string_list(anchors)?);
        }
        if let Some(class) = map.get("classification").and_then(Value::as_str) {
            patch.classification = Some(class.parse()?);
        }
        if let Some(status) = map.get("status").and_then(Value::as_str) {
            patch.status = Some(status.parse()?);
        }
        if let Some(scope) = map.get("scope").and_then(Value::as_str) {
            patch.scope = Some(scope.parse()?);
        }
        Ok(patch)
    }
}

/// Chaves aceitas num patch JSON (`--update --params`, D147).
const PATCH_KEYS: [&str; 8] = [
    "type",
    "statement",
    "body",
    "tags",
    "anchors",
    "classification",
    "status",
    "scope",
];

fn string_list(value: &Value) -> Result<Vec<String>> {
    match value {
        Value::List(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| Error::schema("lista deve conter só strings"))
            })
            .collect(),
        _ => Err(Error::schema("valor deve ser lista de strings")),
    }
}
