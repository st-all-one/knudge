//! Ranking por confiança derivada, sem query textual (K2/D107).
//!
//! Espelha o `rank` do mulch: consumidores com contexto curto querem "as notas mais
//! confirmadas" sem fazer uma pergunta. A confiança é **derivada** (D87) — usa
//! `outcomes`, o feedback de tarefas (X1/D108) e a idade; `similarity` é `0` porque não
//! há texto. Ordem determinística: `(confidence desc, id asc)`.

use crate::lifecycle::confidence::{ConfidenceInput, confidence_score, from_tasks_with};
use crate::retrieval::DEFAULT_LIMIT;
use crate::retrieval::filter::Filter;
use crate::retrieval::index::Index;
use crate::retrieval::pipeline::{age_days, task_confirmers};
use crate::retrieval::{RecallHit, Why};

/// Universo do ranking (K2/D107).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Universe {
    /// Só conhecimento (notas **sem** `scope`); itens de trabalho têm ordenação própria
    /// (`task list --sort impact`).
    #[default]
    Knowledge,
    /// Todas as notas (conhecimento + itens de trabalho).
    All,
}

/// Consulta de `rank` (K2/D107).
#[derive(Debug, Clone, PartialEq)]
pub struct RankQuery {
    /// Universo considerado.
    pub universe: Universe,
    /// Instante atual (para a idade da confiança).
    pub now_ms: Option<i64>,
    /// Máximo de hits (0 = sem limite).
    pub limit: usize,
    /// Peso da confirmação derivada de tarefas (X1/D108).
    pub task_weight: f64,
}

impl Default for RankQuery {
    fn default() -> Self {
        Self {
            universe: Universe::Knowledge,
            now_ms: None,
            limit: DEFAULT_LIMIT,
            task_weight: 0.0,
        }
    }
}

/// Notas mais confiáveis para o filtro dado, sem pergunta textual (D107).
///
/// Ordem determinística: `(confidence desc, id asc)`.
#[must_use]
pub fn rank(index: &Index, filter: &Filter, query: &RankQuery) -> Vec<RecallHit> {
    let confirmers = task_confirmers(index);
    let mut hits: Vec<RecallHit> = index
        .docs
        .iter()
        .filter(|doc| filter.matches(&doc.meta))
        .filter(|doc| query.universe == Universe::All || doc.meta.scope.is_none())
        .map(|doc| {
            let task_confirmation = from_tasks_with(&doc.meta, &confirmers, query.task_weight);
            let confidence = confidence_score(&ConfidenceInput {
                confirmation: doc.meta.confirmation,
                age_days: age_days(doc, query.now_ms),
                task_confirmation,
                ..ConfidenceInput::default()
            });
            RecallHit {
                id: doc.meta.id.clone(),
                statement: doc.statement.clone(),
                score: confidence,
                confidence,
                why: why(doc.meta.confirmation, task_confirmation),
            }
        })
        .collect();
    hits.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| left.id.cmp(&right.id))
    });
    if query.limit > 0 {
        hits.truncate(query.limit);
    }
    hits
}

/// `why` de um hit de `--rank`: `stars` quando há confirmação; senão `universal`.
fn why(confirmation: f64, task_confirmation: f64) -> Why {
    if confirmation > 0.0 || task_confirmation > 0.0 {
        Why::Stars
    } else {
        Why::Universal
    }
}
