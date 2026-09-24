//! `kd knowledge suggest` — sugestões semânticas de aresta/contradição (D158).
//!
//! Read-only: nada vira aresta nem é gravado. Usa o índice vetorial (derivado, opcional) e o
//! grafo para classificar pares em `duplicate`/`contradiction`/`link` (D49/D50).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::embeddings::{
    EmbeddingIndex, Relation, SemanticSuggestion, SuggestionPolicy, semantic_suggestions,
};
use knudge_core::retrieval::Index;
use serde_json::json;

use crate::cli::KnowledgeSuggestArgs;
use crate::commands::embedder;
use crate::output::Output;
use crate::session::Session;

/// Sentinela de busca vazia (D152).
const NO_RESULTS: &str = "[no_results]";

/// Executa `kd knowledge suggest`.
///
/// # Errors
/// Propaga erros de leitura do índice/grafo; índice vetorial ausente degrada para vazio.
pub fn run(session: &Session, args: &KnowledgeSuggestArgs) -> Result<Output> {
    let mut warnings = Vec::new();
    if !session
        .config()
        .get_bool("suggestions.enabled")
        .unwrap_or(true)
    {
        return Ok(Output::new(
            NO_RESULTS.to_string(),
            json!({ "suggestions": [] }),
        ));
    }
    let graph = session.graph()?;
    let index = session.index()?;
    let Some(embedding) = load_embedding(session, &mut warnings)? else {
        return Ok(
            Output::new(NO_RESULTS.to_string(), json!({ "suggestions": [] }))
                .with_warnings(warnings),
        );
    };
    let anchors = anchors_of(&index);
    let policy = policy(session);
    let filter = relation_filter(args);
    let limit = args
        .limit
        .or_else(|| {
            session
                .config()
                .get_int("recall.default_limit")
                .and_then(|raw| usize::try_from(raw).ok())
        })
        .unwrap_or(5);
    let mut found: Vec<SemanticSuggestion> =
        semantic_suggestions(&embedding, &graph, &anchors, &policy, args.top_k);
    if let Some(relation) = filter {
        found.retain(|suggestion| suggestion.relation == relation);
    }
    found.truncate(limit);
    Ok(render(&found).with_warnings(warnings))
}

/// Renderiza a saída do `suggest` (texto + JSON).
fn render(found: &[SemanticSuggestion]) -> Output {
    let text = if found.is_empty() {
        NO_RESULTS.to_string()
    } else {
        found
            .iter()
            .map(|suggestion| {
                format!(
                    "{}|{}|{}|{:.2}",
                    suggestion.relation.as_str(),
                    suggestion.from,
                    suggestion.to,
                    suggestion.score
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data = json!({
        "suggestions": found.iter().map(|suggestion| json!({
            "relation": suggestion.relation.as_str(),
            "from": suggestion.from,
            "to": suggestion.to,
            "score": suggestion.score,
        })).collect::<Vec<_>>(),
    });
    Output::new(text, data)
}

fn policy(session: &Session) -> SuggestionPolicy {
    let config = session.config();
    SuggestionPolicy {
        duplicate: config.get_float("dedup.merge_below").unwrap_or(0.92),
        low: config
            .get_float("suggestions.contradiction_low")
            .unwrap_or(0.4),
        high: config
            .get_float("suggestions.contradiction_high")
            .unwrap_or(0.75),
    }
}

fn relation_filter(args: &KnowledgeSuggestArgs) -> Option<Relation> {
    match args.relation.as_deref() {
        Some("duplicate") => Some(Relation::Duplicate),
        Some("contradiction") => Some(Relation::Contradiction),
        Some("link") => Some(Relation::Link),
        _ => None,
    }
}

fn anchors_of(index: &Index) -> BTreeMap<String, Vec<String>> {
    index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.clone(), doc.meta.anchors.clone()))
        .collect()
}

/// Carrega o índice vetorial, se disponível (degradação graciosa, R33).
fn load_embedding(session: &Session, warnings: &mut Vec<String>) -> Result<Option<EmbeddingIndex>> {
    let Some(embedder) = embedder::build(session)? else {
        return Ok(None);
    };
    EmbeddingIndex::load(
        session.fs_dyn(),
        &session.knowledge_dir(),
        embedder.meta(),
        warnings,
    )
}
