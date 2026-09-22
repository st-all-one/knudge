//! Registro de evento e sua codificação JSON canônica.

use indexmap::IndexMap;

use crate::jsonl::json;
use crate::schema::{Value, hash};
use crate::{Error, Result};

use super::EVENT_PREFIX;

/// Evento auditável (`write`, `update`, `supersede`, `remove`, `learn`, …).
///
/// O `id` é derivado do conteúdo (`op`, `note_id`, `at`, `actor`, `data`), o que garante dedup
/// determinístico sob merge (D26/D28).
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Operação (`write`, `update`, `remove`, `supersede`, …).
    pub op: String,
    /// Nota afetada, quando houver.
    pub note_id: Option<String>,
    /// Instante em milissegundos desde a época.
    pub at: i64,
    /// Quem executou (`cli`, `mcp`, …).
    pub actor: Option<String>,
    /// Payload livre (ex.: `revision`, `reason`).
    pub data: IndexMap<String, Value>,
}

impl Event {
    /// Cria um evento sem nota/ator/dados.
    #[must_use]
    pub fn new(op: impl Into<String>, at: i64) -> Self {
        Self {
            op: op.into(),
            note_id: None,
            at,
            actor: None,
            data: IndexMap::new(),
        }
    }

    /// Associa a nota afetada.
    #[must_use]
    pub fn with_note_id(mut self, note_id: impl Into<String>) -> Self {
        self.note_id = Some(note_id.into());
        self
    }

    /// Define o ator.
    #[must_use]
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    /// Acrescenta um campo ao payload.
    #[must_use]
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// `id` derivado do conteúdo (D26/D95).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum valor do payload não for codificável em JSON.
    pub fn id(&self) -> Result<String> {
        let text = json::encode(&Value::Map(self.content_map()))?;
        Ok(format!(
            "{EVENT_PREFIX}_{}",
            hash::base36_8(hash::short_hash(text.as_bytes()))
        ))
    }

    /// Serializa como linha JSONL (com `\n`).
    ///
    /// # Errors
    /// Propaga erro de codificação.
    pub fn to_line(&self) -> Result<String> {
        let text = json::encode(&Value::Map(self.to_map()?))?;
        Ok(format!("{text}\n"))
    }

    /// Constrói a partir de um valor JSON (a chave `id` é ignorada na leitura).
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se faltar campo obrigatório ou houver tipo errado.
    pub fn from_value(value: &Value) -> Result<Self> {
        let Value::Map(map) = value else {
            return Err(Error::invalid_input("evento deve ser objeto JSON"));
        };
        let op = map
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::invalid_input("evento sem `op`"))?
            .to_string();
        let at = map
            .get("at")
            .and_then(Value::as_int)
            .ok_or_else(|| Error::invalid_input("evento sem `at`"))?;
        let note_id = map
            .get("note_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let actor = map.get("actor").and_then(Value::as_str).map(str::to_string);
        let data = match map.get("data") {
            None => IndexMap::new(),
            Some(Value::Map(data)) => data.clone(),
            Some(_) => return Err(Error::invalid_input("`data` deve ser objeto")),
        };
        Ok(Self {
            op,
            note_id,
            at,
            actor,
            data,
        })
    }

    /// Id armazenado no valor JSON, se presente.
    #[must_use]
    pub fn stored_id(value: &Value) -> Option<&str> {
        value.as_map()?.get("id")?.as_str()
    }

    fn content_map(&self) -> IndexMap<String, Value> {
        let mut map = IndexMap::new();
        map.insert("op".to_string(), Value::Str(self.op.clone()));
        if let Some(note_id) = &self.note_id {
            map.insert("note_id".to_string(), Value::Str(note_id.clone()));
        }
        map.insert("at".to_string(), Value::Int(self.at));
        if let Some(actor) = &self.actor {
            map.insert("actor".to_string(), Value::Str(actor.clone()));
        }
        if !self.data.is_empty() {
            map.insert("data".to_string(), Value::Map(self.data.clone()));
        }
        map
    }

    fn to_map(&self) -> Result<IndexMap<String, Value>> {
        let mut map = self.content_map();
        map.insert("id".to_string(), Value::Str(self.id()?));
        Ok(map)
    }
}
