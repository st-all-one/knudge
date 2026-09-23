//! Subcomandos de manutenção: compact, eval, index, learn e prune (E12-T01).

use std::collections::BTreeMap;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::embeddings::{DrainInput, drain};
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{
    AnchorValidity, DecayPolicy, DemotionInput, ShelfLife, compute_anchor_validity,
    demotion_candidates,
};
use knudge_core::maintenance::{LearnInput, learn, propose_compact};
use knudge_core::store::Note;
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::super::embedder;
use super::super::hooks::{self, HookEvent};

/// `kd maintenance compact` — propõe merge/supersede (nunca aplica em silêncio).
///
/// # Errors
/// Propaga erros de leitura do store/índice.
pub fn compact(session: &Session, scope: Option<&str>) -> Result<Output> {
    let hook = hooks::run(session, HookEvent::PreCompact, &json!({ "scope": scope }))?;
    if hook.blocked {
        return Err(Error::invalid_input("hook `pre-compact` bloqueou"));
    }
    let store = session.store();
    let index = session.index()?;
    let graph = session.graph()?;
    let thresholds = session.thresholds()?;
    let proposals: Vec<_> = propose_compact(&store, &index, &thresholds)
        .into_iter()
        .filter(|proposal| match scope {
            Some(container) => belongs_to(&graph, &proposal.keep, container),
            None => true,
        })
        .collect();
    let text = proposals
        .iter()
        .map(|proposal| {
            format!(
                "{}|{}|{}|{:.2}",
                proposal.strategy.as_str(),
                proposal.keep,
                proposal.ids.join(","),
                proposal.score
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "proposals": proposals.iter().map(|proposal| json!({
            "strategy": proposal.strategy.as_str(),
            "keep": proposal.keep,
            "ids": proposal.ids,
            "why": proposal.why,
            "score": proposal.score,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

/// `kd maintenance learn` — propostas determinísticas (write-gap, dedup, link).
///
/// # Errors
/// Propaga erros de leitura de índice/eventos.
pub fn learn_cmd(session: &Session, scope: Option<&str>) -> Result<Output> {
    let index = session.index()?;
    let graph = session.graph()?;
    let (events, warnings) = session.events().read_all()?;
    let changed = session.changed_paths()?;
    let thresholds = session.thresholds()?;
    let input = LearnInput {
        index: &index,
        graph: &graph,
        events: &events,
        changed_paths: &changed,
        scope,
        thresholds: &thresholds,
    };
    let proposals = learn(&input);
    let text = proposals
        .iter()
        .map(|proposal| {
            format!(
                "{}|{}|{:.2}",
                proposal.kind.as_str(),
                proposal.ids.join(","),
                proposal.score
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "proposals": proposals.iter().map(|proposal| json!({
            "kind": proposal.kind.as_str(),
            "ids": proposal.ids,
            "why": proposal.why,
            "score": proposal.score,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Ação de `kd maintenance index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexAction {
    /// Mostra o estado da fila.
    Status,
    /// Drena a fila agora.
    Drain,
}

/// `kd maintenance index` — status/drain da fila de embeddings.
///
/// # Errors
/// Propaga erros de leitura do índice e de execução do provedor.
pub fn index(session: &Session, action: IndexAction) -> Result<Output> {
    if action == IndexAction::Drain {
        return drain_queue(session);
    }
    let pending = embedder::pending(session)?;
    let data = json!({ "pending": pending });
    Ok(Output::new(format!("pending: {pending}"), data))
}

fn drain_queue(session: &Session) -> Result<Output> {
    let Some(embedder) = embedder::build(session)? else {
        return Ok(Output::new(
            "embeddings desligado (provider = none)",
            json!({ "enabled": false, "indexed": 0, "pending": 0 }),
        ));
    };
    let store = session.store();
    let input = DrainInput {
        store: &store,
        embedder: embedder.as_ref(),
        config: session.config(),
        now_ms: session.now_ms(),
    };
    let outcome = drain(&input)?;
    let text = format!(
        "indexed={} pending={} stale={} cache_hits={}",
        outcome.indexed, outcome.pending, outcome.stale, outcome.cache_hits
    );
    let data = json!({
        "enabled": true,
        "indexed": outcome.indexed,
        "pending": outcome.pending,
        "stale": outcome.stale,
        "cache_hits": outcome.cache_hits,
    });
    Ok(Output::new(text, data).with_warnings(outcome.warnings))
}

/// `kd maintenance eval` — métricas de retrieval (Recall@k/nDCG@k/MRR).
///
/// # Errors
/// Propaga erros de leitura do índice.
pub fn eval(session: &Session, ab: &[String]) -> Result<Output> {
    let index = session.index()?;
    let mut warnings = Vec::new();
    if !ab.is_empty() {
        warnings
            .push("eval --ab requer um dataset de consultas; nenhum configurado (D90)".to_string());
    }
    let data = json!({
        "docs": index.docs.len(),
        "ab": ab,
    });
    Ok(Output::new(format!("docs: {}", index.docs.len()), data).with_warnings(warnings))
}

/// `kd maintenance prune` — propõe aposentadoria (`forget`) por shelf-life/decay (D112).
///
/// Nunca grava (D47): o agente aplica com `kd forget`. Membros de ciclo ficam de fora (D45).
///
/// # Errors
/// Propaga erros de leitura do store/grafo e varredura de âncoras.
pub fn prune(session: &Session, scope: Option<&str>) -> Result<Output> {
    let store = session.store();
    let graph = session.graph()?;
    let mut notes = Vec::new();
    for id in store.list_ids()? {
        notes.push(store.read(&id)?);
    }
    let shelf_life = ShelfLife::from_config(session.config());
    let decay = DecayPolicy::from_config(session.config());
    let validity = validity_map(session, &notes)?;
    let input = DemotionInput {
        now_ms: session.now_ms(),
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
    };
    let mut candidates = demotion_candidates(&notes, &input, &graph)?;
    if let Some(container) = scope {
        candidates.retain(|candidate| belongs_to(&graph, &candidate.id, container));
    }
    let text = candidates
        .iter()
        .map(|candidate| format!("forget|{}|{}", candidate.id, candidate.reason.as_str()))
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "proposals": candidates.iter().map(|candidate| json!({
            "action": "forget",
            "id": candidate.id,
            "reason": candidate.reason.as_str(),
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

/// Validade de âncoras por id (varredura off-path) para o plano de demolição.
fn validity_map(session: &Session, notes: &[Note]) -> Result<BTreeMap<String, AnchorValidity>> {
    let mut map = BTreeMap::new();
    for note in notes {
        let anchors: Vec<String> = note
            .frontmatter
            .string_list("anchors")?
            .into_iter()
            .map(str::to_string)
            .collect();
        if anchors.is_empty() {
            continue;
        }
        let validity = compute_anchor_validity(session.fs_dyn(), session.project_root(), &anchors);
        let _ignored = map.insert(note.id()?.to_string(), validity);
    }
    Ok(map)
}
