//! `kd rewind` — estado/handoff ponto-no-tempo (E12-T01, D57/D88).

use knudge_core::Result;
use knudge_core::handoff::{
    ContextStore, DEFAULT_BUDGET, RewindInput, RewindMode, RewindRequest, rewind,
};
use serde_json::json;

use crate::cli::RewindArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd rewind`.
///
/// # Errors
/// Propaga erros de índice/eventos e de `context_id` inválido.
pub fn run(session: &Session, args: &RewindArgs) -> Result<Output> {
    let index = session.index()?;
    let graph = session.graph()?;
    let (events, mut warnings) = session.events().read_all()?;
    let changed = session.changed_paths()?;
    let request = RewindRequest {
        mode: mode_of(args),
        budget: args.budget.unwrap_or(DEFAULT_BUDGET),
        since: parse::timestamp_opt(args.since.as_ref())?,
        until: parse::timestamp_opt(args.until.as_ref())?,
        resume: args.resume.clone(),
    };
    let contexts = ContextStore::new(session.fs_dyn(), session.knowledge_dir());
    let input = RewindInput {
        index: &index,
        graph: &graph,
        events: &events,
        changed_paths: &changed,
        embeddings_pending: 0,
        now_ms: session.now_ms(),
    };
    let out = rewind(&input, &request, &contexts)?;
    warnings.extend(out.warnings.iter().cloned());
    let data = json!({
        "context_id": out.context_id,
        "text": out.text,
        "items": out.items.iter().map(|item| json!({
            "id": item.id,
            "statement": item.statement,
            "score": item.score,
            "tier": item.tier.as_str(),
        })).collect::<Vec<_>>(),
        "truncated": out.truncated,
        "dropped": out.dropped,
        "embeddings_pending": out.embeddings_pending,
    });
    Ok(Output::new(out.text, data).with_warnings(warnings))
}

fn mode_of(args: &RewindArgs) -> RewindMode {
    if let Some(scope) = &args.scope {
        RewindMode::Scope(scope.clone())
    } else if !args.files.is_empty() {
        RewindMode::Files(args.files.clone())
    } else {
        RewindMode::Manifest
    }
}
