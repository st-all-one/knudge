//! Parser puro de uma operação de tarefa (D141).

use indexmap::IndexMap;

use crate::schema::{EdgeKind, Scope, Value};
use crate::task::TaskSpec;
use crate::{Error, Result};

/// Uma operação do lote (uma linha JSONL / o objeto de `--params`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOp {
    /// Referência local opcional (para `parent`/arestas entre linhas).
    pub key: Option<String>,
    /// `id` presente = atualiza; ausente = cria.
    pub id: Option<String>,
    /// Campos da tarefa (o `statement` fica vazio quando o update não o informa).
    pub spec: TaskSpec,
    /// Arestas explícitas (exclui `results_in`, que vem de `parent`).
    pub edges: Vec<(EdgeKind, String)>,
}

impl TaskOp {
    /// Parseia uma operação de um objeto JSON.
    ///
    /// # Errors
    /// `ErrorKind::Schema`/`InvalidInput` para chave desconhecida, campo inválido ou criação
    /// sem `statement`/`scope`.
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value
            .as_map()
            .ok_or_else(|| Error::schema("operação deve ser um objeto JSON"))?;
        for key in map.keys() {
            if !is_known_key(key) {
                return Err(Error::schema(format!(
                    "chave de tarefa desconhecida: {key}"
                )));
            }
        }
        let id = map.get("id").and_then(Value::as_str).map(str::to_string);
        let statement = map.get("statement").and_then(Value::as_str);
        if statement.is_none() && id.is_none() {
            return Err(Error::invalid_input("operação de criação sem `statement`"));
        }
        let scope = match map.get("scope").and_then(Value::as_str) {
            Some(scope) => scope.parse()?,
            None if id.is_some() => Scope::Task,
            None => return Err(Error::invalid_input("operação de criação sem `scope`")),
        };
        let spec = parse_spec(map, scope, statement.unwrap_or_default())?;
        let edges = parse_edges(map)?;
        Ok(Self {
            key: map.get("key").and_then(Value::as_str).map(str::to_string),
            id,
            spec,
            edges,
        })
    }
}

/// Parseia os campos da tarefa (menos `key`/`id`/arestas).
fn parse_spec(map: &IndexMap<String, Value>, scope: Scope, statement: &str) -> Result<TaskSpec> {
    let mut spec = TaskSpec::new(scope, statement);
    spec.body = map
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if let Some(kind) = map.get("kind").and_then(Value::as_str) {
        spec.kind = Some(kind.parse()?);
    }
    spec.parent = map
        .get("parent")
        .and_then(Value::as_str)
        .map(str::to_string);
    spec.checks = string_list(map.get("checks"))?;
    spec.anchors = string_list(map.get("anchors"))?;
    spec.tags = string_list(map.get("tags"))?;
    if let Some(blocks) = map.get("blocks").and_then(Value::as_int) {
        spec.blocks = Some(
            u32::try_from(blocks).map_err(|_| Error::invalid_input("`blocks` fora do range"))?,
        );
    }
    if let Some(class) = map.get("classification").and_then(Value::as_str) {
        spec.classification = Some(class.parse()?);
    }
    if let Some(status) = map.get("status").and_then(Value::as_str) {
        spec.status = Some(status.parse()?);
    }
    Ok(spec)
}

/// Parseia as arestas explícitas (exclui `results_in`, que vem de `parent`).
fn parse_edges(map: &IndexMap<String, Value>) -> Result<Vec<(EdgeKind, String)>> {
    let mut edges = Vec::new();
    for kind in EdgeKind::ALL {
        if kind == EdgeKind::ResultsIn {
            continue;
        }
        for to in string_list(map.get(kind.key()))? {
            edges.push((kind, to));
        }
    }
    Ok(edges)
}

/// `true` se a chave é de campo conhecido ou de aresta explícita.
fn is_known_key(key: &str) -> bool {
    matches!(
        key,
        "key"
            | "id"
            | "statement"
            | "body"
            | "scope"
            | "kind"
            | "parent"
            | "checks"
            | "anchors"
            | "tags"
            | "classification"
            | "status"
            | "blocks"
    ) || EdgeKind::ALL
        .iter()
        .any(|kind| *kind != EdgeKind::ResultsIn && kind.key() == key)
}

/// Lista de strings de um campo opcional.
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
