//! Modos de consulta de `kd ask`: `recall` (texto) e `--rank` (sem query) (E06, K2/D107).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::embeddings::{EmbeddingIndex, rank_query};
use knudge_core::lifecycle::DEFAULT_TASK_CONFIRMATION;
use knudge_core::retrieval::{
    DEFAULT_ANCHOR_WEIGHT, DEFAULT_LEXICAL_WEIGHT, DEFAULT_LIMIT, DEFAULT_RRF_K,
    DEFAULT_SEMANTIC_WEIGHT, Filter, FusionWeights, Index, RecallQuery, Universe, active_ids,
    recall,
};
use knudge_core::schema::Status;
use serde_json::json;

use crate::cli::AskArgs;
use crate::commands::embedder;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::render::{HitFormat, hit_format, hit_json, load_bodies, preview_chars, render_hit};

/// Sentinela de busca vazia (D152): distingue "sem resultado" de erro sem parsing ambíguo.
const NO_RESULTS: &str = "[no_results]";

/// Executa o `recall` textual com filtros, canal vetorial e `strict` (D94/D102).
///
/// # Errors
/// Propaga erros de índice/grafo e `strict`.
pub(super) fn recall_query(session: &Session, args: &AskArgs) -> Result<Output> {
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let text = args.query.join(" ");
    let mut query = build_recall_query(session, args, index, &text)?;
    if !text.is_empty() {
        attach_semantic(session, &text, &mut query);
    }
    let out = recall(index, graph, &query)?;
    let mut warnings = out.warnings;
    let format = hit_format(args);
    let preview_chars = preview_chars(session);
    let bodies = if format == HitFormat::Brief {
        BTreeMap::new()
    } else {
        load_bodies(session, &out.hits)?
    };
    let body = if out.hits.is_empty() {
        NO_RESULTS.to_string()
    } else {
        out.hits
            .iter()
            .enumerate()
            .map(|(position, hit)| render_hit(hit, position, format, &bodies, preview_chars))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let body = if let Some(as_of) = &args.as_of {
        format!("as_of={as_of}\n{body}")
    } else {
        body
    };
    let data = json!({
        "query": text,
        "as_of": args.as_of,
        "hits": out.hits.iter().map(|hit| hit_json(hit, format, &bodies, args.as_of.as_deref(), &text)).collect::<Vec<_>>(),
    });
    let hit_ids: Vec<String> = out.hits.iter().map(|hit| hit.id.clone()).collect();
    if let Some(warning) = super::record_usage(session, &hit_ids) {
        warnings.push(warning);
    }
    Ok(Output::new(body, data).with_warnings(warnings))
}

/// Monta a `RecallQuery` a partir da config, dos filtros e do universo (D94/D102/D146).
fn build_recall_query(
    session: &Session,
    args: &AskArgs,
    index: &Index,
    text: &str,
) -> Result<RecallQuery> {
    let config = session.config();
    let mut query = RecallQuery::new(text.to_string());
    query.limit = args
        .limit
        .unwrap_or_else(|| usize_from(config.get_int("recall.default_limit"), DEFAULT_LIMIT));
    query.rrf_k = u32_from(config.get_int("recall.rrf_k"), DEFAULT_RRF_K);
    query.task_confirmation_weight = config
        .get_float("recall.confirmation_from_tasks")
        .unwrap_or(DEFAULT_TASK_CONFIRMATION);
    query.weights = FusionWeights {
        lexical: config
            .get_float("recall.lexical_weight")
            .unwrap_or(DEFAULT_LEXICAL_WEIGHT),
        anchor: config
            .get_float("recall.anchor_weight")
            .unwrap_or(DEFAULT_ANCHOR_WEIGHT),
        semantic: config
            .get_float("recall.semantic_weight")
            .unwrap_or(DEFAULT_SEMANTIC_WEIGHT),
    };
    query.filter = Filter {
        types: parse::types(&args.types)?,
        classifications: parse::classifications(&args.classes)?,
        statuses: resolve_statuses(args)?,
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
    };
    query.universe = if args.with_task {
        Universe::All
    } else {
        Universe::Knowledge
    };
    // `--anchor` alimenta o canal de âncoras (D81), não só o filtro: sem isso, um
    // `ask --anchor` sem query textual não teria candidato lexical e voltaria vazio.
    // Repetível e com vírgula (`--anchor a,b --anchor c`).
    query.working_paths.clone_from(&args.anchor);
    query.scope.clone_from(&args.scope);
    query.now_ms = Some(session.now_ms());
    query.strict = config.strict();
    if let Some(raw) = &args.as_of {
        let as_of_ms = parse::timestamp(raw)?;
        if as_of_ms > session.now_ms() {
            return Err(knudge_core::Error::invalid_input("`--as-of` no futuro"));
        }
        let (events, _warnings) = session.events().read_all()?;
        let docs: Vec<(String, i64)> = index
            .docs
            .iter()
            .map(|doc| (doc.meta.id.clone(), doc.meta.created_ms))
            .collect();
        query.as_of = Some(active_ids(&events, &docs, as_of_ms));
    }
    Ok(query)
}

/// Statuses efetivos: os pedidos ou o default que esconde soft-delete/supersede (D43).
fn resolve_statuses(args: &AskArgs) -> Result<Vec<Status>> {
    let statuses = parse::statuses(args.status.iter().cloned().collect::<Vec<_>>().as_slice())?;
    // `--status forgotten`/`--status superseded` inspecionam a linhagem (D43).
    if statuses.is_empty() && args.as_of.is_some() {
        // Com `as_of`, o estado reconstruído é soberano: não pré-excluir do índice atual.
        return Ok(Vec::new());
    }
    Ok(if statuses.is_empty() {
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
