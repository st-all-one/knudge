//! Codec do índice derivado: `NoteDoc` ↔ `Value` (E06-T01).

use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::retrieval::filter::Meta;
use crate::retrieval::index::{Field, FieldTf, NoteDoc};
use crate::schema::{Classification, NoteType, Scope, Status, Value};
use crate::{Error, Result};

pub(super) fn doc_to_value(doc: &NoteDoc) -> Value {
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
    if let Some(scope) = doc.meta.scope {
        map.insert("scope".to_string(), Value::Str(scope.as_str().to_string()));
    }
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
    map.insert("failures".to_string(), Value::Float(doc.meta.failures));
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

pub(super) fn doc_from_value(value: &Value) -> Result<NoteDoc> {
    let map = value
        .as_map()
        .ok_or_else(|| Error::invalid_input("linha de índice não é objeto"))?;
    // O grupo derivado é persistido como `type: epic` (D149), que `NoteType::ALL` não aceita
    // por ser omitido na nota canônica. Aceitar aqui mantém o índice legível (D15).
    let note_type = match str_field(map, "type")? {
        "epic" => NoteType::Epic,
        text => text.parse::<NoteType>()?,
    };
    let meta = Meta {
        id: str_field(map, "id")?.to_string(),
        note_type,
        scope: opt_scope(map)?,
        classification: str_field(map, "classification")?.parse::<Classification>()?,
        status: str_field(map, "status")?.parse::<Status>()?,
        tags: str_list(map, "tags"),
        anchors: str_list(map, "anchors"),
        created_ms: int_field(map, "created")?,
        confirmation: map
            .get("confirmation")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        failures: map.get("failures").and_then(Value::as_f64).unwrap_or(0.0),
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

/// Escopo opcional (índices antigos não o têm — derivado, D15).
fn opt_scope(map: &IndexMap<String, Value>) -> Result<Option<Scope>> {
    match map.get("scope").and_then(Value::as_str) {
        Some(text) => text.parse::<Scope>().map(Some),
        None => Ok(None),
    }
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
