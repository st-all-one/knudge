//! Construção de candidatos e hits do `recall` (E06).
//!
//! Separado de `mod.rs` para manter o arquivo abaixo do teto: filtros determinísticos,
//! canal lexical com boost de tarefa (X1/D108), fusão RRF → [`RecallHit`].

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::lifecycle::confidence::{
    ConfidenceInput, confidence_score, from_tasks_with, is_success_task,
};
use crate::retrieval::anchor;
use crate::retrieval::filter::Meta;
use crate::retrieval::index::{Index, NoteDoc};
use crate::retrieval::rrf::Fused;
use crate::retrieval::{RECENT_WINDOW_MS, RecallHit, RecallQuery, Why};
use crate::schema::EdgeKind;

/// Candidatos após filtros estruturais e `container` (D41/D53).
pub(super) fn candidates(index: &Index, graph: &Graph, query: &RecallQuery) -> BTreeSet<String> {
    let mut allowed = BTreeSet::new();
    for doc in &index.docs {
        if !query.filter.matches(&doc.meta) {
            continue;
        }
        if let Some(container) = query.container.as_deref()
            && !belongs_to(graph, &doc.meta.id, container)
        {
            continue;
        }
        allowed.insert(doc.meta.id.clone());
    }
    allowed
}

/// Tarefas com `outcomes` de sucesso — confirmadoras de notas que compartilham âncoras (D108).
pub(super) fn task_confirmers(index: &Index) -> Vec<&Meta> {
    index
        .docs
        .iter()
        .map(|doc| &doc.meta)
        .filter(|meta| is_success_task(meta))
        .collect()
}

/// Canal lexical, com o boost da confirmação derivada de tarefas (X1/D108).
pub(super) fn lexical_channel(
    index: &Index,
    allowed: &BTreeSet<String>,
    query: &RecallQuery,
    confirmers: &[&Meta],
    weight: f64,
) -> Vec<String> {
    index
        .score_with(&query.text, allowed, |meta| {
            from_tasks_with(meta, confirmers, weight)
        })
        .into_iter()
        .map(|hit| hit.id)
        .collect()
}

/// Entradas da montagem dos hits: a fusão e os ids do canal vetorial (D121).
pub(super) struct FusedChannels<'a> {
    /// Fusão RRF já ordenada.
    pub fused: &'a [Fused],
    /// Ids que vieram pelo canal vetorial (para o `why`).
    pub semantic: &'a BTreeSet<&'a str>,
}

/// Monta os hits finais (com confiança derivada e `why`) a partir da fusão RRF.
pub(super) fn build_hits(
    index: &Index,
    query: &RecallQuery,
    graph: &Graph,
    channels: &FusedChannels<'_>,
    limit: usize,
) -> Vec<RecallHit> {
    let confirmers = task_confirmers(index);
    let weight = query.task_confirmation_weight;
    let semantic_ids = channels.semantic;
    let max_score = channels.fused.first().map_or(0.0, |hit| hit.score);
    let by_id: BTreeMap<&str, &NoteDoc> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc))
        .collect();
    let mut hits = Vec::new();
    for fused_hit in channels.fused.iter().take(limit) {
        let Some(doc) = by_id.get(fused_hit.id.as_str()) else {
            continue;
        };
        let similarity = if max_score > 0.0 {
            fused_hit.score / max_score
        } else {
            0.0
        };
        let confidence = confidence_score(&ConfidenceInput {
            similarity,
            confirmation: doc.meta.confirmation,
            age_days: age_days(doc, query.now_ms),
            task_confirmation: from_tasks_with(&doc.meta, &confirmers, weight),
            ..ConfidenceInput::default()
        });
        hits.push(RecallHit {
            id: fused_hit.id.clone(),
            statement: doc.statement.clone(),
            score: fused_hit.score,
            confidence,
            why: choose_why(doc, query, graph, semantic_ids),
        });
    }
    hits
}

/// Canal vetorial **filtrado** por `allowed` (D102).
///
/// As views/filtros determinísticos são contrato e não podem ser furados por uma nota só
/// semanticamente próxima.
pub(super) fn semantic_channel(query: &RecallQuery, allowed: &BTreeSet<String>) -> Vec<String> {
    query
        .vector
        .as_ref()
        .map(|ids| {
            ids.iter()
                .filter(|id| allowed.contains(id.as_str()))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn belongs_to(graph: &Graph, id: &str, container: &str) -> bool {
    if id == container {
        return true;
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![id.to_string()];
    while let Some(current) = stack.pop() {
        for dependency in graph.targets(&current, EdgeKind::DependsOn) {
            if dependency == container {
                return true;
            }
            if seen.insert(dependency.clone()) {
                stack.push(dependency.clone());
            }
        }
    }
    false
}

fn choose_why(doc: &NoteDoc, query: &RecallQuery, graph: &Graph, semantic: &BTreeSet<&str>) -> Why {
    let matched = anchor::match_note(&doc.meta, &query.working_paths, &query.working_ids);
    if matched.file {
        return Why::FileMatch;
    }
    if matched.id {
        return Why::AnchorMatch;
    }
    if let Some(container) = query.container.as_deref()
        && belongs_to(graph, &doc.meta.id, container)
    {
        return Why::TrackerMatch;
    }
    if doc.meta.confirmation > 0.0 {
        return Why::Stars;
    }
    // O canal vetorial é um sinal mais forte que a recência genérica (D121): um hit que veio
    // de um sinônimo não deve ser rotulado `recent` só por ser novo.
    if semantic.contains(doc.meta.id.as_str()) {
        return Why::Semantic;
    }
    if let Some(now) = query.now_ms
        && now.saturating_sub(doc.meta.created_ms) <= RECENT_WINDOW_MS
    {
        return Why::Recent;
    }
    Why::Universal
}

pub(super) fn age_days(doc: &NoteDoc, now_ms: Option<i64>) -> f64 {
    let Some(now) = now_ms else {
        return 0.0;
    };
    let elapsed = now.saturating_sub(doc.meta.created_ms).max(0);
    f64::from(i32::try_from(elapsed / 86_400_000).unwrap_or(i32::MAX))
}
