//! Planejamento de demolição: shelf-life × decay × proteção de ciclo (E10-T01/T02/T04).
//!
//! Função **pura** que combina as políticas num plano revisável; a execução fica em
//! [`super::supersession::demote`]. Membros de ciclo **nunca** entram no plano (D45).

use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::graph::Graph;
use crate::schema::{EdgeKind, outcome_stats};
use crate::store::Note;

use super::confidence::{ConfidenceInput, confidence_score};
use super::decay::{AnchorValidity, DecayPolicy, should_demote};
use super::shelf_life::{ShelfLife, age_days, is_expired_with};
use super::usage::UsageIndex;

/// Motivo da demolição proposta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DemotionReason {
    /// Prazo de shelf-life vencido.
    Expired,
    /// Âncoras majoritariamente quebradas após o grace.
    AnchorDecay,
    /// Lado perdedor de uma contradição declarada (D177).
    Contradicted,
}

impl DemotionReason {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::AnchorDecay => "anchor_decay",
            Self::Contradicted => "contradicted",
        }
    }
}

/// Candidato à demolição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemotionCandidate {
    /// Id da nota.
    pub id: String,
    /// Motivo.
    pub reason: DemotionReason,
}

/// Entradas do planejamento de demolição.
pub struct DemotionInput<'a> {
    /// Instante atual (ms).
    pub now_ms: i64,
    /// Política de shelf-life.
    pub shelf_life: &'a ShelfLife,
    /// Política de decay de âncoras.
    pub decay: &'a DecayPolicy,
    /// Validade de âncoras por id (off-path).
    pub validity: &'a BTreeMap<String, AnchorValidity>,
    /// Uso por id (renovação de shelf-life — D154); `None` desliga.
    pub usage: Option<&'a UsageIndex>,
}

/// Planeja as demolições, excluindo membros de ciclo (D45).
///
/// `validity` traz a validade de âncoras por id (calculada off-path pelo rebuild).
///
/// # Errors
/// Retorna `ErrorKind::Schema` se algum campo tipado estiver malformado.
pub fn demotion_candidates(
    notes: &[Note],
    input: &DemotionInput<'_>,
    graph: &Graph,
) -> Result<Vec<DemotionCandidate>> {
    let protected: BTreeSet<String> = graph.cycle_members();
    let contradicted = contradiction_losers(notes, graph, input.now_ms)?;
    let mut candidates = Vec::new();
    for note in notes {
        let id = note.id()?.to_string();
        if protected.contains(&id) {
            continue;
        }
        if is_expired_with(
            note,
            input.now_ms,
            input.shelf_life,
            input.usage.and_then(|usage| usage.last_seen(&id)),
        )? {
            candidates.push(DemotionCandidate {
                id,
                reason: DemotionReason::Expired,
            });
            continue;
        }
        if let Some(validity) = input.validity.get(&id)
            && should_demote(validity, input.decay, age_days(note, input.now_ms))
        {
            candidates.push(DemotionCandidate {
                id,
                reason: DemotionReason::AnchorDecay,
            });
            continue;
        }
        if contradicted.contains(&id) {
            candidates.push(DemotionCandidate {
                id,
                reason: DemotionReason::Contradicted,
            });
        }
    }
    candidates
        .sort_unstable_by(|left, right| (&left.id, left.reason).cmp(&(&right.id, right.reason)));
    Ok(candidates)
}

/// Lado **perdedor** de cada contradição declarada, por confiança derivada (D177).
///
/// Mesmo critério do `rank`: vence a maior confiança; empate não elege perdedor.
fn contradiction_losers(notes: &[Note], graph: &Graph, now_ms: i64) -> Result<BTreeSet<String>> {
    let mut confidence: BTreeMap<&str, f64> = BTreeMap::new();
    for note in notes {
        confidence.insert(note.id()?, derived_confidence(note, now_ms));
    }
    let mut losers = BTreeSet::new();
    for note in notes {
        let id = note.id()?;
        for target in graph.targets(id, EdgeKind::Contradicts) {
            let (Some(&left), Some(&right)) = (confidence.get(id), confidence.get(target.as_str()))
            else {
                continue;
            };
            if left < right {
                losers.insert(id.to_string());
            } else if right < left {
                losers.insert(target.clone());
            }
        }
    }
    Ok(losers)
}

/// Confiança derivada (sem similaridade textual) de uma nota — desempate de D177.
fn derived_confidence(note: &Note, now_ms: i64) -> f64 {
    let stats = outcome_stats(&note.frontmatter);
    let age = f64::from(i32::try_from(age_days(note, now_ms)).unwrap_or(i32::MAX));
    confidence_score(&ConfidenceInput {
        successes: stats.successes,
        failures: stats.failures,
        age_days: age,
        ..ConfidenceInput::default()
    })
}
