//! Subcomandos de manutenção: compact e learn (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::maintenance::{LearnInput, learn, propose_compact};
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
