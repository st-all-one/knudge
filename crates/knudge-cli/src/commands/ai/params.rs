//! `--params` do `kd ai`: objeto JSON → [`AiArgs`] (D222).
//!
//! O objeto aceita `query` (string ou lista), `area`, `source`, `limit`, `sort`, `agent_id` e
//! `agent_role`; com `--params` o objeto define a consulta inteira, como no `ask` (D147).

use knudge_core::jsonl;
use knudge_core::schema::Value;
use knudge_core::{Error, Result};

use crate::cli::{AiArea, AiArgs, AiSort};
use crate::commands::input;

/// Resolve `--params` (JSON ou `-` para stdin); com ele, o objeto define a consulta inteira.
pub(super) fn resolve_params(args: &AiArgs) -> Result<AiArgs> {
    let Some(params) = &args.params else {
        return Ok(args.clone());
    };
    let text = if params == "-" {
        input::read_stdin()?
    } else {
        params.clone()
    };
    parse_params(&jsonl::decode(&text)?)
}

/// Converte o objeto de `--params` em [`AiArgs`] (D222).
fn parse_params(value: &Value) -> Result<AiArgs> {
    let map = value
        .as_map()
        .ok_or_else(|| Error::schema("`--params` deve ser um objeto JSON"))?;
    let query = match map.get("query") {
        None => Vec::new(),
        Some(Value::List(_)) => string_list(map.get("query"))?,
        Some(Value::Str(query)) => vec![query.clone()],
        Some(_) => return Err(Error::schema("`query` deve ser string ou lista")),
    };
    let mut args = AiArgs {
        query,
        ..AiArgs::default()
    };
    if let Some(area) = map.get("area").and_then(Value::as_str) {
        args.area = AiArea::parse(area)?;
    }
    if let Some(sort) = map.get("sort").and_then(Value::as_str) {
        args.sort = AiSort::parse(sort)?;
    }
    args.source = map
        .get("source")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(limit) = map.get("limit").and_then(Value::as_int) {
        args.limit = Some(
            usize::try_from(limit).map_err(|_| Error::invalid_input("`limit` fora do range"))?,
        );
    }
    args.agent_id = string_list(map.get("agent_id"))?;
    args.agent_role = string_list(map.get("agent_role"))?;
    Ok(args)
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
