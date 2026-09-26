//! `kd rewind` — estado/handoff ponto-no-tempo (E12-T01, D57/D88).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::handoff::{
    ContextStore, CorpusScope as HandoffScope, DEFAULT_BUDGET, RewindInput, RewindMode,
    RewindRequest, rewind,
};
use knudge_core::lifecycle::{DEFAULT_TASK_CONFIRMATION, ShelfLife, UsageStore, freshness_with};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::RewindArgs;
use crate::commands::corpus::CorpusScope;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::embedder;

/// Executa `kd rewind`.
///
/// # Errors
/// Propaga erros de índice/eventos e de `context_id` inválido.
pub fn run(session: &Session, args: &RewindArgs) -> Result<Output> {
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let corpus = corpus_of(args);
    let selection = corpus.select(index, graph)?;
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
    let notes = &loaded.notes;
    let policy = ShelfLife::from_config(session.config());
    let pending = embedder::pending(session, notes)?;
    let usage = UsageStore::new(session.fs_dyn(), session.knowledge_dir()).index()?;
    let fresh = freshness_with(notes, session.now_ms(), &policy, pending, &usage)?;
    let bodies = bodies_of(notes);
    let input = RewindInput {
        index,
        graph,
        events: &events,
        changed_paths: &changed,
        freshness: fresh,
        bodies: &bodies,
        scope: HandoffScope {
            filter: selection.filter().clone(),
            allowed: selection.allowed().cloned(),
        },
        task_confirmation_weight: session
            .config()
            .get_float("recall.confirmation_from_tasks")
            .unwrap_or(DEFAULT_TASK_CONFIRMATION),
        now_ms: session.now_ms(),
    };
    let out = rewind(&input, &request, &contexts)?;
    warnings.extend(out.warnings.iter().cloned());
    let ids: Vec<String> = out.items.iter().map(|item| item.id.clone()).collect();
    warnings.extend(record_usage(session, &policy, &ids));
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

/// Credita o uso dos itens devolvidos (D154), se `renew_on_use` estiver ligado.
///
/// Devolve aviso (R33) quando o derivado não puder ser gravado; nunca derruba a leitura.
fn record_usage(session: &Session, policy: &ShelfLife, ids: &[String]) -> Option<String> {
    if !policy.renew_on_use || ids.is_empty() {
        return None;
    }
    let store = UsageStore::new(session.fs_dyn(), session.knowledge_dir());
    match store.record(ids, session.now_ms()) {
        Ok(_ignored) => None,
        Err(error) => Some(format!("uso: {error}")),
    }
}

/// Mapa id → corpo das notas (para o `rewind` anexar `foundational`/`decision`, D162).
fn bodies_of(notes: &[Note]) -> BTreeMap<String, String> {
    notes
        .iter()
        .filter_map(|note| {
            note.frontmatter
                .id()
                .ok()
                .map(|id| (id.to_string(), note.body.clone()))
        })
        .collect()
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

/// Escopo de corpus do `rewind` (filtros + vizinhança) — D143.
fn corpus_of(args: &RewindArgs) -> CorpusScope {
    CorpusScope {
        types: args.types.clone(),
        classes: args.classes.clone(),
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
        around: args.around.clone(),
        depth: args.depth,
        universe: false,
    }
}
