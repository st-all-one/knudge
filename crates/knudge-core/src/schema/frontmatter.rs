//! Frontmatter canônico (E02-T01/T07).
//!
//! A ordem das chaves é **contrato** (D04/D13): [`Frontmatter::to_value`] sempre emite na
//! [`CANONICAL_KEYS`](super::keys::CANONICAL_KEYS), independentemente da ordem de inserção.
//! Campos opcionais são **omitidos**, nunca `null`/vazio (D05). Chave desconhecida é rejeitada
//! no write e **ignorada com warning** no read (D16).

use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::schema::{
    Classification, EDGE_KEYS, Edge, EdgeKind, NoteType, Scope, Status, Value, id, text,
};
use crate::time::Timestamp;
use crate::{Error, Result, toon};

use super::keys::{CANONICAL_KEYS, REQUIRED_KEYS};

/// Frontmatter canônico, com campos opcionais omitidos.
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
    fields: IndexMap<String, Value>,
}

impl Frontmatter {
    /// Cria um frontmatter vazio.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fields: IndexMap::new(),
        }
    }

    /// Insere um campo conhecido (write estrito: chave desconhecida é erro).
    pub fn set(&mut self, key: &str, value: Value) -> Result<()> {
        if !CANONICAL_KEYS.contains(&key) {
            return Err(Error::schema(format!("chave desconhecida: {key:?}")));
        }
        self.fields.insert(key.to_string(), value);
        Ok(())
    }

    /// Lê um campo.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Remove um campo.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.fields.shift_remove(key)
    }

    /// Chaves presentes, na ordem canônica.
    #[must_use]
    pub fn keys(&self) -> Vec<&str> {
        CANONICAL_KEYS
            .iter()
            .copied()
            .filter(|key| self.fields.contains_key(*key))
            .collect()
    }

    /// Converte para [`Value::Map`] na ordem canônica.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut map = IndexMap::new();
        for key in CANONICAL_KEYS {
            if let Some(value) = self.fields.get(key) {
                map.insert(key.to_string(), value.clone());
            }
        }
        Value::Map(map)
    }

    /// Lê de um [`Value::Map`]; chaves desconhecidas viram warnings (D16).
    pub fn from_value(value: &Value) -> Result<(Self, Vec<String>)> {
        let Value::Map(map) = value else {
            return Err(Error::schema("frontmatter deve ser um mapa"));
        };
        let mut fields = IndexMap::new();
        let mut warnings = Vec::new();
        for (key, item) in map {
            if CANONICAL_KEYS.contains(&key.as_str()) {
                fields.insert(key.clone(), item.clone());
            } else {
                warnings.push(format!("chave desconhecida ignorada: {key:?}"));
            }
        }
        Ok((Self { fields }, warnings))
    }

    /// Parseia um documento TOON e aplica [`Frontmatter::from_value`].
    pub fn parse(src: &str) -> Result<(Self, Vec<String>)> {
        let value = toon::parse(src)?;
        Self::from_value(&value)
    }

    /// Valida presença e tipo dos campos obrigatórios.
    pub fn validate(&self) -> Result<()> {
        for key in REQUIRED_KEYS {
            if !self.fields.contains_key(key) {
                return Err(Error::schema(format!("campo obrigatório ausente: {key}")));
            }
        }
        self.note_type()?;
        let confidence = self.confidence()?;
        if !(0.0..=1.0).contains(&confidence) {
            return Err(Error::schema(format!(
                "confidence fora de 0..=1: {confidence}"
            )));
        }
        text::validate_statement(self.statement()?)?;
        self.schema_version()?;
        if !id::is_valid_note_id(self.id()?) {
            return Err(Error::schema(format!("id inválido: {:?}", self.id()?)));
        }
        self.validate_edges()?;
        Ok(())
    }

    fn validate_edges(&self) -> Result<()> {
        for key in EDGE_KEYS {
            for target in self.string_list(key)? {
                if !id::is_valid_note_id(target) {
                    return Err(Error::schema(format!(
                        "id de aresta inválido em {key}: {target:?}"
                    )));
                }
            }
        }
        match self.fields.get("superseded_by") {
            None => Ok(()),
            Some(Value::Str(target)) if id::is_valid_note_id(target) => Ok(()),
            Some(_) => Err(Error::schema("superseded_by deve ser um id válido")),
        }
    }

    /// Campo `id`.
    pub fn id(&self) -> Result<&str> {
        self.required_str("id")
    }

    /// Campo `type`.
    pub fn note_type(&self) -> Result<NoteType> {
        NoteType::from_str(self.required_str("type")?)
    }

    /// Campo `statement`.
    pub fn statement(&self) -> Result<&str> {
        self.required_str("statement")
    }

    /// Campo `confidence`.
    pub fn confidence(&self) -> Result<f64> {
        self.fields
            .get("confidence")
            .and_then(Value::as_f64)
            .ok_or_else(|| Error::schema("confidence ausente ou inválido"))
    }

    /// Campo `schema_version`.
    pub fn schema_version(&self) -> Result<u32> {
        let version = self
            .fields
            .get("schema_version")
            .and_then(Value::as_int)
            .ok_or_else(|| Error::schema("schema_version ausente"))?;
        u32::try_from(version).map_err(|_| Error::schema("schema_version inválido"))
    }

    /// Campo opcional `classification` (default `tactical`).
    pub fn classification(&self) -> Result<Classification> {
        self.optional_enum("classification")
    }

    /// Campo opcional `status` (default `active`).
    pub fn status(&self) -> Result<Status> {
        self.optional_enum("status")
    }

    /// Campo opcional `scope`.
    pub fn scope(&self) -> Result<Option<Scope>> {
        match self.fields.get("scope") {
            None => Ok(None),
            Some(Value::Str(text)) => Ok(Some(Scope::from_str(text)?)),
            Some(_) => Err(Error::schema("scope deve ser string")),
        }
    }

    /// Campo opcional `expires_at` (RFC3339 → ms).
    pub fn expires_at(&self) -> Result<Option<i64>> {
        self.optional_timestamp("expires_at")
    }

    /// Campo opcional `not_before` (RFC3339 → ms) — agendamento, separado da expiração (D56).
    pub fn not_before(&self) -> Result<Option<i64>> {
        self.optional_timestamp("not_before")
    }

    fn optional_timestamp(&self, key: &str) -> Result<Option<i64>> {
        match self.fields.get(key) {
            None => Ok(None),
            Some(Value::Int(ms)) => Ok(Some(*ms)),
            Some(Value::Str(text)) => text
                .parse::<Timestamp>()
                .map(|ts| Some(ts.as_millis()))
                .map_err(|_| Error::schema(format!("{key} deve ser timestamp RFC3339"))),
            Some(_) => Err(Error::schema(format!("{key} deve ser string"))),
        }
    }

    /// Lista de strings de um campo opcional (ids de aresta).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se o campo não for lista de strings.
    pub fn string_list(&self, key: &str) -> Result<Vec<&str>> {
        match self.fields.get(key) {
            None => Ok(Vec::new()),
            Some(Value::List(items)) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .ok_or_else(|| Error::schema(format!("{key} deve conter apenas strings")))
                })
                .collect(),
            Some(_) => Err(Error::schema(format!("{key} deve ser uma lista"))),
        }
    }

    /// Arestas explícitas declaradas nesta nota (ordem canônica dos tipos).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se `id` ou algum destino for inválido.
    pub fn edges(&self) -> Result<Vec<Edge>> {
        let from = self.id()?.to_string();
        let mut edges = Vec::new();
        for kind in EdgeKind::ALL {
            for to in self.string_list(kind.key())? {
                edges.push(Edge::new(from.clone(), kind, to.to_string()));
            }
        }
        Ok(edges)
    }

    fn optional_enum<T: FromStr<Err = Error> + Default>(&self, key: &str) -> Result<T> {
        match self.fields.get(key) {
            None => Ok(T::default()),
            Some(Value::Str(text)) => text.parse::<T>(),
            Some(_) => Err(Error::schema(format!("{key} deve ser string"))),
        }
    }

    fn required_str(&self, key: &str) -> Result<&str> {
        self.fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::schema(format!("campo obrigatório ausente ou inválido: {key}")))
    }
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&toon::emit(&self.to_value()))
    }
}
