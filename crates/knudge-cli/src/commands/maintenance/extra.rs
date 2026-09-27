//! Subcomandos de manutenção: compact, learn e prune (E12-T01).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{
    AnchorValidity, DecayPolicy, DemotionCandidate, DemotionInput, DriftStore, ShelfLife,
    UsageIndex, UsageStore, compute_anchor_validity_cached, demotion_candidates,
    entries_from_validity, retention_for, walk_paths,
};
use knudge_core::maintenance::{LearnInput, learn, propose_compact};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::CorpusArgs;
use crate::output::Output;
use crate::session::Session;

use super::super::corpus::CorpusScope;
use super::super::hooks::{self, HookEvent};
use super::proposals::{VerifyMode, annotate, build_reports, count_kinds, render_proposals};

/// `kd maintenance compact` — propõe merge/supersede (nunca aplica em silêncio).
///
/// # Errors
/// Propaga erros de leitura do store/índice.
pub fn compact(
    session: &Session,
    scope: Option<&str>,
    corpus: &CorpusArgs,
    verify: VerifyMode,
) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance compact")?;
    let hook = hooks::run(session, HookEvent::PreCompact, &json!({ "scope": scope }))?;
    if hook.blocked {
        return Err(Error::invalid_input("hook `pre-compact` bloqueou"));
    }
    let store = session.store();
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let selection = corpus.select(index, graph)?;
    let thresholds = session.thresholds()?;
    let proposals: Vec<_> = propose_compact(&store, index, &thresholds)
        .into_iter()
        .filter(|proposal| match scope {
            Some(container) => belongs_to(graph, &proposal.keep, container),
            None => true,
        })
        .filter(|proposal| selection.matches_id(&proposal.keep))
        .collect();
    let (reports, warnings) = build_reports(session, verify, &proposals, |proposal| {
        (
            "merge".to_string(),
            json!({
                "strategy": proposal.strategy.as_str(),
                "keep": proposal.keep,
                "ids": proposal.ids,
                "score": proposal.score,
            }),
        )
    })?;
    let output = render_proposals(
        &proposals,
        &reports,
        warnings,
        |proposal| {
            format!(
                "{}|{}|{}|{:.2}",
                proposal.strategy.as_str(),
                proposal.keep,
                proposal.ids.join(","),
                proposal.score
            )
        },
        |proposal| {
            json!({
                "strategy": proposal.strategy.as_str(),
                "keep": proposal.keep,
                "ids": proposal.ids,
                "why": proposal.why,
                "score": proposal.score,
            })
        },
    );
    let counts = count_kinds(&proposals, |proposal| proposal.strategy.as_str());
    Ok(annotate(
        output,
        &counts,
        "aplique com `kd write --supersede <keep> <ids>` e revalide com `kd doctor`",
    ))
}

/// `kd maintenance learn` — propostas determinísticas (write-gap, dedup, link).
///
/// # Errors
/// Propaga erros de leitura de índice/eventos.
pub fn learn_cmd(
    session: &Session,
    scope: Option<&str>,
    corpus: &CorpusArgs,
    verify: VerifyMode,
) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance learn")?;
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let selection = corpus.select(index, graph)?;
    let (events, warnings) = session.events().read_all()?;
    let changed = session.changed_paths()?;
    let thresholds = session.thresholds()?;
    let input = LearnInput {
        index,
        graph,
        events: &events,
        changed_paths: &changed,
        scope,
        thresholds: &thresholds,
    };
    let proposals: Vec<_> = learn(&input)
        .into_iter()
        .filter(|proposal| proposal.ids.iter().any(|id| selection.matches_id(id)))
        .collect();
    let mut warnings = warnings;
    let (reports, gate_warnings) = build_reports(session, verify, &proposals, |proposal| {
        (
            proposal.kind.as_str().to_string(),
            json!({
                "kind": proposal.kind.as_str(),
                "ids": proposal.ids,
                "score": proposal.score,
            }),
        )
    })?;
    warnings.extend(gate_warnings);
    let output = render_proposals(
        &proposals,
        &reports,
        warnings,
        |proposal| {
            format!(
                "{}|{}|{:.2}",
                proposal.kind.as_str(),
                proposal.ids.join(","),
                proposal.score
            )
        },
        |proposal| {
            json!({
                "kind": proposal.kind.as_str(),
                "ids": proposal.ids,
                "why": proposal.why,
                "score": proposal.score,
            })
        },
    );
    let counts = count_kinds(&proposals, |proposal| proposal.kind.as_str());
    Ok(annotate(
        output,
        &counts,
        "aplique com `kd write --link`/`--supersede` e revalide com `kd doctor`",
    ))
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
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let selection = corpus.select(index, graph)?;
    let notes = &loaded.notes;
    let shelf_life = ShelfLife::from_config(session.config());
    let decay = DecayPolicy::from_config(session.config());
    let validity = validity_map(session, notes)?;
    let usage = UsageStore::new(session.fs_dyn(), session.knowledge_dir()).index()?;
    let input = DemotionInput {
        now_ms: session.now_ms(),
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: Some(&usage),
    };
    let mut candidates = demotion_candidates(notes, &input, graph)?;
    if let Some(container) = scope {
        candidates.retain(|candidate| belongs_to(graph, &candidate.id, container));
    }
    candidates.retain(|candidate| selection.matches_id(&candidate.id));
    let mut text = candidates
        .iter()
        .map(|candidate| format!("forget|{}|{}", candidate.id, candidate.reason.as_str()))
        .collect::<Vec<_>>()
        .join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    let _ignored = write!(
        text,
        "propostas: forget={}\npróximos: kd forget --id <ID>",
        candidates.len()
    );
    let retentions = retention_map(session, &shelf_life, notes, &usage, &candidates)?;
    let data = json!({
        "proposals": candidates.iter().map(|candidate| json!({
            "action": "forget",
            "id": candidate.id,
            "reason": candidate.reason.as_str(),
            "retention": retentions.get(&candidate.id),
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

/// Retenção atual (D190) das notas candidatas, por id — campo aditivo do `--json`.
fn retention_map(
    session: &Session,
    shelf_life: &ShelfLife,
    notes: &[Note],
    usage: &UsageIndex,
    candidates: &[DemotionCandidate],
) -> Result<BTreeMap<String, f64>> {
    let by_id: BTreeMap<&str, &Note> = notes
        .iter()
        .filter_map(|note| note.id().ok().map(|id| (id, note)))
        .collect();
    let mut retentions = BTreeMap::new();
    for candidate in candidates {
        if let Some(note) = by_id.get(candidate.id.as_str()) {
            let value = retention_for(
                note,
                session.now_ms(),
                shelf_life,
                usage.last_seen(&candidate.id),
            )?;
            let _ignored = retentions.insert(candidate.id.clone(), value);
        }
    }
    Ok(retentions)
}

/// Validade de âncoras por id (varredura off-path) para o plano de demolição.
///
/// Caminha o projeto **uma vez** e reusa os caminhos para todas as notas (E16/T10): antes era um
/// walk por nota, `O(notas × projeto)`.
fn validity_map(session: &Session, notes: &[Note]) -> Result<BTreeMap<String, AnchorValidity>> {
    let paths = walk_paths(session.fs_dyn(), session.project_root());
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
        let validity = compute_anchor_validity_cached(
            session.fs_dyn(),
            session.project_root(),
            &paths,
            &anchors,
        );
        let _ignored = map.insert(note.id()?.to_string(), validity);
    }
    // Persiste o drift derivado (`.idx/drift.jsonl`, D203) do mesmo walk: alimenta a confiança
    // derivada de `ask`/`knowledge rank` sem custo no caminho quente.
    DriftStore::new(session.fs_dyn(), session.knowledge_dir())
        .persist(&entries_from_validity(&map))?;
    Ok(map)
}
