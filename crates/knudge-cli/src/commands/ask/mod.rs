//! `kd ask` — toda pesquisa: rank, recall, get (`--id`) e expand (`--around`) (E12-T01).

use std::io::IsTerminal;

use knudge_core::graph::Graph;
use knudge_core::jsonl;
use knudge_core::lifecycle::UsageStore;
use knudge_core::retrieval::get;
use knudge_core::schema::{NoteType, Status, Value};
use knudge_core::store::Note;
use knudge_core::{Error, Result};
use serde_json::json;

use crate::cli::AskArgs;
use crate::commands::input;
use crate::output::Output;
use crate::session::Session;

mod query;
mod render;

use query::recall_query;

/// Uso resumido quando nenhum modo de busca é selecionado (token-optimized).
const ASK_USAGE: &str = "\
nenhum modo de busca: informe uma QUERY ou um modo.
  kd ask <QUERY> [--type T] [--class C] [--tag T] [--status S] [--scope ID]
                 [--anchor PATH] [--since TS] [--until TS] [--limit N]
                 [--brief] [--full-content] [--with-task]
  kd ask --id <ID>...                              # corpos por id
  kd ask --around <ID> [--via ARESTA] [--depth N]  # expande o grafo
  kd ask --anchor <PATH>                           # por âncora, sem query
veja: kd ask --help";

/// Executa `kd ask` no modo adequado (get > expand > recall).
///
/// # Errors
/// Propaga erros de índice/grafo e `strict` (D94). Sem modo selecionado (query e âncora
/// vazias), devolve o uso do comando em vez de sair silenciosamente.
pub fn run(session: &Session, args: &AskArgs) -> Result<Output> {
    if let Some(params) = &args.params {
        let text = if params == "-" {
            input::read_stdin()?
        } else {
            params.clone()
        };
        let parsed = parse_ask_params(&jsonl::decode(&text)?)?;
        return run(session, &parsed);
    }
    let resolved = resolve_query(args)?;
    let args = resolved.as_ref().unwrap_or(args);
    if !args.ids.is_empty() {
        return get_ids(session, args);
    }
    if let Some(around) = &args.around {
        return expand(session, args, around);
    }
    if args.query.is_empty() && args.anchor.is_empty() {
        return Err(Error::invalid_input(ASK_USAGE));
    }
    recall_query(session, args)
}

/// Lê a consulta de stdin quando o posicional é `-` ou vazio com pipe (D147).
fn resolve_query(args: &AskArgs) -> Result<Option<AskArgs>> {
    let explicit = args.query.len() == 1 && args.query.first().is_some_and(|q| q.as_str() == "-");
    let piped = args.query.is_empty()
        && args.ids.is_empty()
        && args.around.is_none()
        && !std::io::stdin().is_terminal();
    if !explicit && !piped {
        return Ok(None);
    }
    let text = input::read_stdin()?;
    let query = text.trim().to_string();
    if !explicit && query.is_empty() {
        return Ok(None);
    }
    let mut owned = args.clone();
    owned.query = vec![query];
    Ok(Some(owned))
}

/// Converte o objeto de `--params` em [`AskArgs`] (D147).
fn parse_ask_params(value: &Value) -> Result<AskArgs> {
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
    args.since = map.get("since").and_then(Value::as_str).map(str::to_string);
    args.until = map.get("until").and_then(Value::as_str).map(str::to_string);
    args.as_of = map.get("as_of").and_then(Value::as_str).map(str::to_string);
    if let Some(limit) = map.get("limit").and_then(Value::as_int) {
        args.limit = Some(
            usize::try_from(limit).map_err(|_| Error::invalid_input("`limit` fora do range"))?,
        );
    }
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

fn get_ids(session: &Session, args: &AskArgs) -> Result<Output> {
    let store = session.store();
    let out = get(&store, &args.ids)?;
    let mut warnings = out.warnings;
    let blocks: Vec<String> = out
        .notes
        .iter()
        .map(|note| {
            let id = note.frontmatter.id().unwrap_or_default();
            let statement = note.frontmatter.statement().unwrap_or_default();
            if note.body.is_empty() || args.brief {
                format!("{id}|{statement}")
            } else {
                format!("{id}|{statement}\n{}", note.body)
            }
        })
        .collect();
    let data = json!({
        "notes": out.notes.iter().map(note_json).collect::<Vec<_>>(),
    });
    let ids: Vec<String> = out
        .notes
        .iter()
        .filter_map(|note| note.frontmatter.id().ok().map(str::to_string))
        .collect();
    if let Some(warning) = record_usage(session, &ids) {
        warnings.push(warning);
    }
    Ok(Output::new(blocks.join("\n"), data).with_warnings(warnings))
}

fn expand(session: &Session, args: &AskArgs, around: &str) -> Result<Output> {
    if !session.store().exists(around) {
        return Err(Error::not_found(format!("nota ausente: {around}")));
    }
    let graph: Graph = session.graph()?;
    let kind = match &args.via {
        Some(value) => Some(value.parse()?),
        None => None,
    };
    let hits = graph.expand(around, kind, u32::from(args.depth));
    let text = hits
        .iter()
        .map(|hit| format!("{}|{}|{}", hit.id, hit.kind.as_str(), hit.depth))
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "around": around,
        "hits": hits.iter().map(|hit| json!({
            "id": hit.id,
            "kind": hit.kind.as_str(),
            "depth": hit.depth,
        })).collect::<Vec<_>>(),
    });
    let mut ids: Vec<String> = hits.iter().map(|hit| hit.id.clone()).collect();
    ids.push(around.to_string());
    let warnings = record_usage(session, &ids).into_iter().collect();
    Ok(Output::new(text, data).with_warnings(warnings))
}

fn note_json(note: &Note) -> serde_json::Value {
    json!({
        "id": note.frontmatter.id().unwrap_or_default(),
        "type": note.frontmatter.note_type().map(NoteType::as_str).unwrap_or_default(),
        "statement": note.frontmatter.statement().unwrap_or_default(),
        "status": note.frontmatter.status().map(Status::as_str).unwrap_or_default(),
        "body": note.body,
    })
}

/// Credita uso dos ids lidos (D154). No-op se `retention.renew_on_use` estiver desligado.
///
/// Retorna aviso (R33) quando o derivado não puder ser gravado; nunca derruba a leitura.
fn record_usage(session: &Session, ids: &[String]) -> Option<String> {
    if ids.is_empty() {
        return None;
    }
    if !session
        .config()
        .get_bool("retention.renew_on_use")
        .unwrap_or(false)
    {
        return None;
    }
    let store = UsageStore::new(session.fs_dyn(), session.knowledge_dir());
    match store.record(ids, session.now_ms()) {
        Ok(_ignored) => None,
        Err(error) => Some(format!("uso: {error}")),
    }
}
