//! Modos de consulta de `kd ask`: `recall` (texto) e `--rank` (sem query) (E06, K2/D107).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::embeddings::{EmbeddingIndex, rank_query};
use knudge_core::lifecycle::DEFAULT_TASK_CONFIRMATION;
use knudge_core::retrieval::{
    DEFAULT_LIMIT, DEFAULT_RRF_K, Filter, RankQuery, RecallHit, RecallQuery, Universe,
    format_brief, format_hit, get, rank, recall,
};
use knudge_core::schema::Status;
use serde_json::json;

use crate::cli::AskArgs;
use crate::commands::embedder;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

/// `kd ask --rank` — notas mais confiáveis, sem pergunta textual (K2/D107).
///
/// # Errors
/// Propaga erros de índice e validação de filtros.
pub(super) fn rank_mode(session: &Session, args: &AskArgs) -> Result<Output> {
    let index = session.index()?;
    let config = session.config();
    let filter = Filter {
        types: parse::types(&args.types)?,
        classifications: parse::classifications(&args.classes)?,
        statuses: resolve_statuses(args)?,
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
    };
    let limit = args
        .limit
        .unwrap_or_else(|| usize_from(config.get_int("recall.default_limit"), DEFAULT_LIMIT));
    let weight = config
        .get_float("recall.confirmation_from_tasks")
        .unwrap_or(DEFAULT_TASK_CONFIRMATION);
    let hits = rank(
        &index,
        &filter,
        &RankQuery {
            universe: Universe::Knowledge,
            now_ms: Some(session.now_ms()),
            limit,
            task_weight: weight,
        },
    );
    let text = hits.iter().map(format_hit).collect::<Vec<_>>().join("\n");
    let data = json!({
        "ranked": hits.iter().map(|hit| json!({
            "id": hit.id,
            "statement": hit.statement,
            "confidence": hit.confidence,
            "why": hit.why.as_str(),
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

/// Executa o `recall` textual com filtros, canal vetorial e `strict` (D94/D102).
///
/// # Errors
/// Propaga erros de índice/grafo e `strict`.
pub(super) fn recall_query(session: &Session, args: &AskArgs) -> Result<Output> {
    let index = session.index()?;
    let graph = session.graph()?;
    let config = session.config();
    let text = args.query.join(" ");
    let mut query = RecallQuery::new(text.clone());
    query.limit = args
        .limit
        .unwrap_or_else(|| usize_from(config.get_int("recall.default_limit"), DEFAULT_LIMIT));
    query.rrf_k = u32_from(config.get_int("recall.rrf_k"), DEFAULT_RRF_K);
    query.task_confirmation_weight = config
        .get_float("recall.confirmation_from_tasks")
        .unwrap_or(DEFAULT_TASK_CONFIRMATION);
    query.filter = Filter {
        types: parse::types(&args.types)?,
        classifications: parse::classifications(&args.classes)?,
        statuses: resolve_statuses(args)?,
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
    };
    // `--anchor` alimenta o canal de âncoras (D81), não só o filtro: sem isso, um
    // `ask --anchor` sem query textual não teria candidato lexical e voltaria vazio.
    // Repetível e com vírgula (`--anchor a,b --anchor c`).
    query.working_paths.clone_from(&args.anchor);
    query.container.clone_from(&args.container);
    query.now_ms = Some(session.now_ms());
    query.strict = config.strict();
    if !text.is_empty() {
        attach_semantic(session, &text, &mut query);
    }
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

/// Statuses efetivos: os pedidos ou o default que esconde soft-delete/supersede (D43).
fn resolve_statuses(args: &AskArgs) -> Result<Vec<Status>> {
    let statuses = parse::statuses(args.status.iter().cloned().collect::<Vec<_>>().as_slice())?;
    Ok(if statuses.is_empty() {
        // `--status forgotten`/`--status superseded` inspecionam a linhagem (D43).
        Status::ALL
            .into_iter()
            .filter(|status| !matches!(status, Status::Superseded | Status::Forgotten))
            .collect()
    } else {
        statuses
    })
}

/// Liga o canal vetorial quando `recall.semantic` está ligado e há índice (D102).
///
/// Degradação graciosa (R33): falha do provedor vira `channel_warnings` e o `recall` cai no
/// lexical; `strict` promove o aviso a erro. Índice ausente/vazio ⇒ canal desligado e **nenhum**
/// HTTP é feito.
fn attach_semantic(session: &Session, text: &str, query: &mut RecallQuery) {
    if !session.config().get_bool("recall.semantic").unwrap_or(true) {
        return;
    }
    match semantic_ids(session, text, &mut query.channel_warnings) {
        Ok(Some(ids)) => query.vector = Some(ids),
        Ok(None) => {}
        Err(error) => query.channel_warnings.push(format!("vetorial: {error}")),
    }
}

/// Ids do índice vetorial mais próximos de `text` (ou `None` sem índice/embedder).
fn semantic_ids(
    session: &Session,
    text: &str,
    warnings: &mut Vec<String>,
) -> Result<Option<Vec<String>>> {
    let Some(embedder) = embedder::build(session)? else {
        return Ok(None);
    };
    let Some(index) = EmbeddingIndex::load(
        session.fs_dyn(),
        &session.knowledge_dir(),
        embedder.meta(),
        warnings,
    )?
    else {
        return Ok(None);
    };
    if index.is_empty() {
        return Ok(None);
    }
    let Some(vector) = embedder.embed(&[text.to_string()])?.pop() else {
        return Ok(None);
    };
    let top_k = session
        .config()
        .get_int("recall.semantic_top_k")
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(50);
    Ok(Some(rank_query(&index, &vector, top_k, 0.0)))
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

pub(super) fn usize_from(value: Option<i64>, fallback: usize) -> usize {
    value
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(fallback)
}

fn u32_from(value: Option<i64>, fallback: u32) -> u32 {
    value
        .and_then(|raw| u32::try_from(raw).ok())
        .unwrap_or(fallback)
}
