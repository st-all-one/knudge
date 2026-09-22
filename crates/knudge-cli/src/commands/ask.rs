//! `kd ask` — toda pesquisa: recall, get (`--id`) e expand (`--around`) (E12-T01).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::{
    DEFAULT_LIMIT, DEFAULT_RRF_K, Filter, RecallHit, RecallQuery, format_brief, format_hit, get,
    recall,
};
use knudge_core::schema::{NoteType, Status};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::AskArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd ask` no modo adequado (get > expand > recall).
///
/// # Errors
/// Propaga erros de índice/grafo e `strict` (D94).
pub fn run(session: &Session, args: &AskArgs) -> Result<Output> {
    if !args.ids.is_empty() {
        return get_ids(session, args);
    }
    if let Some(around) = &args.around {
        return expand(session, args, around);
    }
    recall_query(session, args)
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

fn recall_query(session: &Session, args: &AskArgs) -> Result<Output> {
    let index = session.index()?;
    let graph = session.graph()?;
    let config = session.config();
    let text = args.query.join(" ");
    let mut query = RecallQuery::new(text.clone());
    query.limit = args
        .limit
        .unwrap_or_else(|| usize_from(config.get_int("recall.default_limit"), DEFAULT_LIMIT));
    query.rrf_k = u32_from(config.get_int("recall.rrf_k"), DEFAULT_RRF_K);
    let statuses = parse::statuses(args.status.iter().cloned().collect::<Vec<_>>().as_slice())?;
    let statuses = if statuses.is_empty() {
        // Soft-delete/supersede ficam fora do `ask` por padrão (D43); peça
        // `--status forgotten`/`--status superseded` para inspecionar a linhagem.
        Status::ALL
            .into_iter()
            .filter(|status| !matches!(status, Status::Superseded | Status::Forgotten))
            .collect()
    } else {
        statuses
    };
    query.filter = Filter {
        types: parse::types(&args.types)?,
        classifications: parse::classifications(&args.classes)?,
        statuses,
        tags: args.tags.clone(),
        anchors: args.anchor.iter().cloned().collect(),
    };
    query.container.clone_from(&args.container);
    query.now_ms = Some(session.now_ms());
    query.strict = config.strict();
    let out = recall(&index, &graph, &query)?;
    let format = if args.brief {
        HitFormat::Brief
    } else {
        HitFormat::Full
    };
    let bodies = if args.with_body && format == HitFormat::Full {
        load_bodies(session, &out.hits)?
    } else {
        BTreeMap::new()
    };
    let body = out
        .hits
        .iter()
        .map(|hit| render_hit(hit, format, &bodies))
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "query": text,
        "hits": out.hits.iter().map(|hit| hit_json(hit, format, &bodies)).collect::<Vec<_>>(),
    });
    Ok(Output::new(body, data).with_warnings(out.warnings))
}

/// Formato de renderização de um hit no pipe/JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitFormat {
    /// `id|statement|score|why` (e corpo com `--with-body`).
    Full,
    /// `id|statement`.
    Brief,
}

/// Carrega os corpos dos hits (só no modo `--with-body`).
fn load_bodies(session: &Session, hits: &[RecallHit]) -> Result<BTreeMap<String, String>> {
    let ids: Vec<String> = hits.iter().map(|hit| hit.id.clone()).collect();
    let out = get(&session.store(), &ids)?;
    let mut bodies = BTreeMap::new();
    for note in out.notes {
        if let Ok(id) = note.frontmatter.id() {
            let _ignored = bodies.insert(id.to_string(), note.body);
        }
    }
    Ok(bodies)
}

/// Renderiza um hit no pipe: `--brief` → `id|statement`; senão 4 colunas + corpo opcional.
fn render_hit(hit: &RecallHit, format: HitFormat, bodies: &BTreeMap<String, String>) -> String {
    if format == HitFormat::Brief {
        return format_brief(hit);
    }
    let line = format_hit(hit);
    match bodies.get(hit.id.as_str()) {
        Some(text) if !text.is_empty() => format!("{line}\n{text}"),
        _ => line,
    }
}

fn hit_json(
    hit: &RecallHit,
    format: HitFormat,
    bodies: &BTreeMap<String, String>,
) -> serde_json::Value {
    let mut value = if format == HitFormat::Brief {
        json!({ "id": hit.id, "statement": hit.statement })
    } else {
        json!({
            "id": hit.id,
            "statement": hit.statement,
            "score": hit.score,
            "confidence": hit.confidence,
            "why": hit.why.as_str(),
        })
    };
    if let Some(text) = bodies.get(hit.id.as_str())
        && let Some(object) = value.as_object_mut()
    {
        let _ignored = object.insert("body".to_string(), json!(text));
    }
    value
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

fn usize_from(value: Option<i64>, fallback: usize) -> usize {
    value
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(fallback)
}

fn u32_from(value: Option<i64>, fallback: u32) -> u32 {
    value
        .and_then(|raw| u32::try_from(raw).ok())
        .unwrap_or(fallback)
}
