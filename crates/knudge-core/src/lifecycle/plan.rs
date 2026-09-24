//! Planejamento de demolição: shelf-life × decay × proteção de ciclo (E10-T01/T02/T04).
//!
//! Função **pura** que combina as políticas num plano revisável; a execução fica em
//! [`super::supersession::demote`]. Membros de ciclo **nunca** entram no plano (D45).

use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::graph::Graph;
use crate::store::Note;

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
}

impl DemotionReason {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::AnchorDecay => "anchor_decay",
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
        }
    }
    candidates.sort_by(|left, right| (&left.id, left.reason).cmp(&(&right.id, right.reason)));
    Ok(candidates)
}
