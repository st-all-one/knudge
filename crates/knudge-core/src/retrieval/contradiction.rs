//! Lado perdedor de contradições declaradas (D177).
//!
//! Para cada aresta `A ⊣ B`, vence o lado de maior confiança derivada; o perdedor é rebaixado no
//! `rank`/`recall` (penalidade) e proposto no `prune` (`DemotionReason::Contradicted`). Módulo
//! separado de `pipeline.rs` para manter os arquivos abaixo do teto.

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::lifecycle::DriftIndex;
use crate::lifecycle::confidence::{ConfidenceInput, confidence_score, from_tasks_with};
use crate::retrieval::filter::Meta;
use crate::retrieval::index::{Index, NoteDoc};
use crate::retrieval::pipeline::age_days;
use crate::schema::EdgeKind;

/// Contexto do desempate de confiança (D177/D203).
///
/// Agrupa os insumos comuns a `losers`/`rank`/`build_hits` para não estourar o limite de
/// argumentos e manter uma única fonte do cálculo de confiança.
pub(super) struct ContradictionContext<'a> {
    /// Instante atual (idade da confiança).
    pub now_ms: Option<i64>,
    /// Tarefas de sucesso confirmadoras (D108).
    pub confirmers: &'a [&'a Meta],
    /// Peso da confirmação de tarefas.
    pub weight: f64,
    /// Drift de âncoras por id (D203).
    pub drift: &'a DriftIndex,
}

/// Ids do **lado perdedor** de cada contradição declarada (D177).
///
/// Para cada aresta `A ⊣ B`, vence o lado de maior confiança derivada (Beta + idade + tarefa);
/// empate não elege perdedor. Determinístico e O(C) no nº de arestas de contradição.
pub(super) fn losers(
    index: &Index,
    graph: &Graph,
    ctx: &ContradictionContext<'_>,
) -> BTreeSet<String> {
    // Caminho rápido: sem aresta `contradicts`, nada a comparar (nem alocação).
    if !graph.has_contradictions() {
        return BTreeSet::new();
    }
    let mut pairs: Vec<(&NoteDoc, &str)> = Vec::new();
    for doc in &index.docs {
        for target in graph.targets(&doc.meta.id, EdgeKind::Contradicts) {
            pairs.push((doc, target.as_str()));
        }
    }
    if pairs.is_empty() {
        return BTreeSet::new();
    }
    let by_id: BTreeMap<&str, &NoteDoc> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc))
        .collect();
    let mut confidence: BTreeMap<&str, f64> = BTreeMap::new();
    let mut losers = BTreeSet::new();
    for (doc, target) in pairs {
        let Some(other) = by_id.get(target) else {
            continue;
        };
        let left = *confidence
            .entry(doc.meta.id.as_str())
            .or_insert_with(|| doc_confidence(doc, ctx));
        let right = *confidence
            .entry(other.meta.id.as_str())
            .or_insert_with(|| doc_confidence(other, ctx));
        if left < right {
            losers.insert(doc.meta.id.clone());
        } else if right < left {
            losers.insert(other.meta.id.clone());
        }
    }
    losers
}

/// Confiança derivada de uma nota, sem similaridade textual (base do desempate de D177).
fn doc_confidence(doc: &NoteDoc, ctx: &ContradictionContext<'_>) -> f64 {
    confidence_score(&ConfidenceInput {
        successes: doc.meta.confirmation,
        failures: doc.meta.failures,
        drift: ctx.drift.get(&doc.meta.id),
        age_days: age_days(doc, ctx.now_ms),
        task_confirmation: from_tasks_with(&doc.meta, ctx.confirmers, ctx.weight),
        ..ConfidenceInput::default()
    })
}
