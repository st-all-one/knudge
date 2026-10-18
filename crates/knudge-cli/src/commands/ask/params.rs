//! `--params` do `kd ask`: objeto JSON → [`AskArgs`] (D147).
//!
//! O contrato de máquina aceita **os mesmos** filtros da linha de comando (`--agent`/`--activity`
//! inclusive, D218); listas de seleção são sempre listas JSON (`"tag": ["a","b"]`), excepto
//! `query`, que aceita string ou lista.

use knudge_core::schema::Value;
use knudge_core::{Error, Result};

use crate::cli::AskArgs;

/// Default de `--top-k` no modo `--suggest` (espelha o `default_value_t` do clap).
const DEFAULT_TOP_K: usize = 5;

/// Converte o objeto de `--params` em [`AskArgs`] (D147).
///
/// # Errors
/// `ErrorKind::Schema`/`InvalidInput` se um campo tiver o tipo ou o range errados.
pub(super) fn parse_ask_params(value: &Value) -> Result<AskArgs> {
    let map = value
        .as_map()
        .ok_or_else(|| Error::schema("`--params` deve ser um objeto JSON"))?;
    let mut args = AskArgs {
        query: match map.get("query") {
            None => Vec::new(),
            Some(Value::List(_)) => string_list(map.get("query"))?,
            Some(Value::Str(query)) => vec![query.clone()],
            Some(_) => return Err(Error::schema("`query` deve ser string ou lista")),
        },
        ..AskArgs::default()
    };
    args.ids = string_list(map.get("id"))?;
    args.around = map
        .get("around")
        .and_then(Value::as_str)
        .map(str::to_string);
    args.via = map.get("via").and_then(Value::as_str).map(str::to_string);
    if let Some(depth) = map.get("depth").and_then(Value::as_int) {
        args.depth =
            u8::try_from(depth).map_err(|_| Error::invalid_input("`depth` fora do range"))?;
    }
    args.brief = map.get("brief").and_then(Value::as_bool).unwrap_or(false);
    args.with_task = map
        .get("with_task")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    args.full_content = map
        .get("full_content")
        .and_then(Value::as_bool)
        .or_else(|| map.get("with_body").and_then(Value::as_bool))
        .unwrap_or(false);
    args.types = string_list(map.get("type"))?;
    args.classes = string_list(map.get("class"))?;
    args.tags = string_list(map.get("tag"))?;
    args.status = map
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_string);
    args.scope = map.get("scope").and_then(Value::as_str).map(str::to_string);
    args.anchor = string_list(map.get("anchor"))?;
    args.agent = string_list(map.get("agent"))?;
    args.activity = string_list(map.get("activity"))?;
    args.since = map.get("since").and_then(Value::as_str).map(str::to_string);
    args.until = map.get("until").and_then(Value::as_str).map(str::to_string);
    args.as_of = map.get("as_of").and_then(Value::as_str).map(str::to_string);
    if let Some(limit) = map.get("limit").and_then(Value::as_int) {
        args.limit = Some(
            usize::try_from(limit).map_err(|_| Error::invalid_input("`limit` fora do range"))?,
        );
    }
    parse_modes(value, &mut args)?;
    Ok(args)
}

/// Lê as flags de modo (`--rank`/`--tags`/`--suggest`/`--universe`/`--top-k`/`--relation`).
fn parse_modes(value: &Value, args: &mut AskArgs) -> Result<()> {
    let map = value
        .as_map()
        .ok_or_else(|| Error::schema("`--params` deve ser um objeto JSON"))?;
    args.rank = map.get("rank").and_then(Value::as_bool).unwrap_or(false);
    args.tags_vocab = map
        .get("tags_vocab")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    args.suggest = map.get("suggest").and_then(Value::as_bool).unwrap_or(false);
    args.universe = map
        .get("universe")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    args.top_k = match map.get("top_k").and_then(Value::as_int) {
        Some(top_k) => {
            usize::try_from(top_k).map_err(|_| Error::invalid_input("`top_k` fora do range"))?
        }
        None => DEFAULT_TOP_K,
    };
    args.relation = map
        .get("relation")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok(())
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
