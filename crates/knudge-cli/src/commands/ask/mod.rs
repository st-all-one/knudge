//! `kd ask` — toda pesquisa: rank, recall, get (`--id`) e expand (`--around`) (E12-T01).

use knudge_core::graph::Graph;
use knudge_core::retrieval::{get, tag_counts};
use knudge_core::schema::{NoteType, Status};
use knudge_core::store::Note;
use knudge_core::{Error, Result};
use serde_json::json;

use crate::cli::AskArgs;
use crate::output::Output;
use crate::session::Session;

mod query;

use query::{rank_mode, recall_query};

/// Uso resumido quando nenhum modo de busca é selecionado (token-optimized).
const ASK_USAGE: &str = "\
nenhum modo de busca: informe uma QUERY ou um modo.
  kd ask <QUERY> [--type T] [--class C] [--tag T] [--status S] [--scope ID]
                 [--anchor PATH] [--since TS] [--until TS] [--limit N] [--brief] [--with-body]
  kd ask --id <ID>...                              # corpos por id
  kd ask --around <ID> [--via ARESTA] [--depth N]  # expande o grafo
  kd ask --rank                                    # mais confiáveis (D107)
  kd ask --tags                                    # vocabulário de tags
  kd ask --anchor <PATH>                           # por âncora, sem query
veja: kd ask --help";

/// Executa `kd ask` no modo adequado (rank > tags > get > expand > recall).
///
/// # Errors
/// Propaga erros de índice/grafo e `strict` (D94). Sem modo selecionado (query e âncora
/// vazias), devolve o uso do comando em vez de sair silenciosamente.
pub fn run(session: &Session, args: &AskArgs) -> Result<Output> {
    if args.rank {
        return rank_mode(session, args);
    }
    if args.tag_vocab {
        return tag_vocab(session, args);
    }
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

/// `kd ask --tags` — vocabulário de tags (`tag|count`, `count` desc, `tag` asc — D107).
fn tag_vocab(session: &Session, args: &AskArgs) -> Result<Output> {
    let index = session.index()?;
    let mut counts = tag_counts(&index);
    if let Some(limit) = args.limit {
        counts.truncate(limit);
    }
    let text = counts
        .iter()
        .map(|(tag, count)| format!("{tag}|{count}"))
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "tags": counts.iter().map(|(tag, count)| json!({"tag": tag, "count": count})).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

fn get_ids(session: &Session, args: &AskArgs) -> Result<Output> {
    let store = session.store();
    let out = get(&store, &args.ids)?;
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
    Ok(Output::new(blocks.join("\n"), data).with_warnings(out.warnings))
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
    Ok(Output::new(text, data))
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
