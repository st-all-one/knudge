//! Subcomandos de manutenção: compact, learn e prune (E12-T01).

use std::collections::BTreeMap;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{
    AnchorValidity, DecayPolicy, DemotionInput, ShelfLife, compute_anchor_validity,
    demotion_candidates,
};
use knudge_core::maintenance::{LearnInput, learn, propose_compact};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::CorpusArgs;
use crate::output::Output;
use crate::session::Session;

use super::super::corpus::CorpusScope;
use super::super::hooks::{self, HookEvent};

/// `kd maintenance compact` — propõe merge/supersede (nunca aplica em silêncio).
///
/// # Errors
/// Propaga erros de leitura do store/índice.
pub fn compact(session: &Session, scope: Option<&str>, corpus: &CorpusArgs) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance compact")?;
    let hook = hooks::run(session, HookEvent::PreCompact, &json!({ "scope": scope }))?;
    if hook.blocked {
        return Err(Error::invalid_input("hook `pre-compact` bloqueou"));
    }
    let store = session.store();
    let index = session.index()?;
    let graph = session.graph()?;
    let selection = corpus.select(&index, &graph)?;
    let thresholds = session.thresholds()?;
    let proposals: Vec<_> = propose_compact(&store, &index, &thresholds)
        .into_iter()
        .filter(|proposal| match scope {
            Some(container) => belongs_to(&graph, &proposal.keep, container),
            None => true,
        })
        .filter(|proposal| selection.matches_id(&proposal.keep))
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
pub fn learn_cmd(session: &Session, scope: Option<&str>, corpus: &CorpusArgs) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance learn")?;
    let index = session.index()?;
    let graph = session.graph()?;
    let selection = corpus.select(&index, &graph)?;
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
    let proposals: Vec<_> = learn(&input)
        .into_iter()
        .filter(|proposal| proposal.ids.iter().any(|id| selection.matches_id(id)))
        .collect();
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

/// `kd maintenance prune` — propõe aposentadoria (`forget`) por shelf-life/decay (D112).
///
/// Nunca grava (D47): o agente aplica com `kd forget`. Membros de ciclo ficam de fora (D45).
///
/// # Errors
/// Propaga erros de leitura do store/grafo e varredura de âncoras.
pub fn prune(session: &Session, scope: Option<&str>, corpus: &CorpusArgs) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance prune")?;
    let store = session.store();
    let index = session.index()?;
    let graph = session.graph()?;
    let selection = corpus.select(&index, &graph)?;
    let mut notes = Vec::new();
    for id in store.list_ids()? {
        if let Some(note) = store.read_optional(&id)? {
            notes.push(note);
        }
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
    candidates.retain(|candidate| selection.matches_id(&candidate.id));
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
