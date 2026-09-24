//! `kd knowledge rank` — notas mais confiáveis, sem pergunta textual (ex-`ask --rank`, D107/D146).

use knudge_core::Result;
use knudge_core::lifecycle::DEFAULT_TASK_CONFIRMATION;
use knudge_core::retrieval::{DEFAULT_LIMIT, RankQuery, Universe, format_hit, rank};
use serde_json::json;

use crate::cli::KnowledgeRankArgs;
use crate::commands::corpus::CorpusScope;
use crate::output::Output;
use crate::session::Session;

/// `kd knowledge rank` — exige escopo ou `--universe` (D146 B).
///
/// # Errors
/// Propaga erros de índice/grafo; `invalid_input` (2) sem escopo.
pub fn run(session: &Session, args: &KnowledgeRankArgs) -> Result<Output> {
    let scope = CorpusScope {
        types: args.types.clone(),
        classes: args.classes.clone(),
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
        around: args.around.clone(),
        depth: args.depth,
        universe: args.universe,
    };
    scope.require("knowledge rank")?;
    let index = session.index()?;
    let graph = session.graph()?;
    let selection = scope.select(&index, &graph)?;
    let config = session.config();
    let limit = args
        .limit
        .unwrap_or_else(|| usize_from(config.get_int("recall.default_limit"), DEFAULT_LIMIT));
    let weight = config
        .get_float("recall.confirmation_from_tasks")
        .unwrap_or(DEFAULT_TASK_CONFIRMATION);
    let hits: Vec<_> = rank(
        &index,
        selection.filter(),
        &RankQuery {
            universe: if args.universe {
                Universe::All
            } else {
                Universe::Knowledge
            },
            now_ms: Some(session.now_ms()),
            limit: 0,
            task_weight: weight,
        },
    )
    .into_iter()
    .filter(|hit| selection.matches_id(&hit.id))
    .take(limit)
    .collect();
    let text = hits.iter().map(format_hit).collect::<Vec<_>>().join("\n");
    let data = json!({
        "ranked": hits.iter().map(|hit| json!({
            "id": hit.id,
            "statement": hit.statement,
            "confidence": hit.confidence,
            "why": hit.why.as_str(),
            "channels": {
                "lexical": hit.channels.lexical,
                "anchor": hit.channels.anchor,
                "semantic": hit.channels.semantic,
                "recent": hit.channels.recent,
                "stars": hit.channels.stars,
            },
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

fn usize_from(value: Option<i64>, fallback: usize) -> usize {
    value
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(fallback)
}
