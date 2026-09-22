//! Protocolo de dedup em duas fases (D26/D80).
//!
//! O `write` é precedido por um `recall`: o índice ranqueia candidatos por BM25 e a decisão usa
//! uma **similaridade lexical em `[0,1]`** (Dice sobre o conjunto de termos) — calibrada para os
//! limiares `0.75`/`0.92`. Sem embeddings o score é lexical; o dedup **semântico é eventual**
//! (E11) e só **propõe** merges, nunca funde em silêncio.

use std::collections::BTreeSet;

use crate::config::Config;
use crate::retrieval::{Field, Index, NoteDoc};
use crate::{Error, Result};

use super::Draft;

/// Número máximo de candidatos avaliados por `write`/reconciliação.
pub const MAX_CANDIDATES: usize = 10;

/// Limiares do protocolo de escrita (config `dedup.*`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DedupThresholds {
    /// Abaixo disto, cria (`dedup.create_below`).
    pub create_below: f64,
    /// A partir disto, rejeita (`dedup.merge_below`); no meio, merge.
    pub merge_below: f64,
}

impl Default for DedupThresholds {
    fn default() -> Self {
        Self {
            create_below: 0.75,
            merge_below: 0.92,
        }
    }
}

impl DedupThresholds {
    /// Constrói limiares validando `0 ≤ create_below ≤ merge_below ≤ 1`.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para limiares fora de ordem/intervalo.
    pub fn new(create_below: f64, merge_below: f64) -> Result<Self> {
        let valid = (0.0..=1.0).contains(&create_below)
            && (0.0..=1.0).contains(&merge_below)
            && create_below <= merge_below;
        if !valid {
            return Err(Error::config(format!(
                "limiares de dedup inválidos: create_below={create_below}, merge_below={merge_below}"
            )));
        }
        Ok(Self {
            create_below,
            merge_below,
        })
    }

    /// Lê os limiares da config efetiva, caindo para os defaults.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se os limiares lidos forem inconsistentes.
    pub fn from_config(config: &Config) -> Result<Self> {
        let defaults = Self::default();
        let create_below = config
            .get_float("dedup.create_below")
            .unwrap_or(defaults.create_below);
        let merge_below = config
            .get_float("dedup.merge_below")
            .unwrap_or(defaults.merge_below);
        Self::new(create_below, merge_below)
    }
}

/// Candidato retornado na fase 1 (`--dry-run`).
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Id da nota existente.
    pub id: String,
    /// `statement` da candidata.
    pub statement: String,
    /// Similaridade lexical com o rascunho.
    pub score: f64,
}

/// Decisão da fase 1.
#[derive(Debug, Clone, PartialEq)]
pub enum DedupDecision {
    /// Não há quase-duplicata: cria.
    Create,
    /// Faixa de merge: funde no candidato.
    Merge {
        /// Id do candidato.
        candidate: String,
        /// Similaridade.
        score: f64,
    },
    /// Acima do teto: rejeita (use `update`).
    Reject {
        /// Id do candidato.
        candidate: String,
        /// Similaridade.
        score: f64,
    },
}

impl DedupDecision {
    /// Id do candidato envolvido, se houver.
    #[must_use]
    pub fn candidate(&self) -> Option<&str> {
        match self {
            Self::Create => None,
            Self::Merge { candidate, .. } | Self::Reject { candidate, .. } => Some(candidate),
        }
    }

    /// Similaridade envolvida, se houver.
    #[must_use]
    pub fn score(&self) -> Option<f64> {
        match self {
            Self::Create => None,
            Self::Merge { score, .. } | Self::Reject { score, .. } => Some(*score),
        }
    }
}

/// Resultado da fase 1: candidatos + decisão sugerida.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteProposal {
    /// Candidatos ordenados por similaridade.
    pub candidates: Vec<Candidate>,
    /// Decisão sugerida pelo sistema.
    pub decision: DedupDecision,
}

/// Propõe merge de quase-duplicados (reconciliação eventual — D47/D80).
#[derive(Debug, Clone, PartialEq)]
pub struct MergeProposal {
    /// Id mantido.
    pub keep: String,
    /// Id proposto para merge.
    pub drop: String,
    /// Similaridade.
    pub score: f64,
    /// Motivo legível.
    pub reason: String,
}

/// Fase 1: ranqueia candidatos e decide sem escrever.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se o rascunho não gerar um documento indexável.
pub fn propose(
    index: &Index,
    draft: &Draft,
    thresholds: &DedupThresholds,
) -> Result<WriteProposal> {
    let note = draft.to_note(0)?;
    let draft_doc = NoteDoc::from_note(&note)?;
    let allowed: BTreeSet<String> = index.docs.iter().map(|doc| doc.meta.id.clone()).collect();
    let mut candidates = Vec::new();
    for hit in index
        .score(&draft.text(), &allowed)
        .into_iter()
        .take(MAX_CANDIDATES)
    {
        let Some(doc) = index.docs.iter().find(|doc| doc.meta.id == hit.id) else {
            continue;
        };
        candidates.push(Candidate {
            id: hit.id,
            statement: doc.statement.clone(),
            score: dice(&draft_doc, doc),
        });
    }
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    let decision = decide(thresholds, candidates.first());
    Ok(WriteProposal {
        candidates,
        decision,
    })
}

/// Decide a ação a partir do melhor candidato.
#[must_use]
pub fn decide(thresholds: &DedupThresholds, best: Option<&Candidate>) -> DedupDecision {
    let Some(candidate) = best else {
        return DedupDecision::Create;
    };
    if candidate.score >= thresholds.merge_below {
        DedupDecision::Reject {
            candidate: candidate.id.clone(),
            score: candidate.score,
        }
    } else if candidate.score >= thresholds.create_below {
        DedupDecision::Merge {
            candidate: candidate.id.clone(),
            score: candidate.score,
        }
    } else {
        DedupDecision::Create
    }
}

/// Similaridade Dice sobre o conjunto de termos de dois documentos.
#[must_use]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "contagens de termos cabem em f64; soma com domínio limitado ao vocabulário"
)]
pub fn dice(a: &NoteDoc, b: &NoteDoc) -> f64 {
    let sa = term_set(a);
    let sb = term_set(b);
    if sa.is_empty() && sb.is_empty() {
        return 1.0;
    }
    let shared = sa.intersection(&sb).count();
    let total = sa.len().saturating_add(sb.len());
    if total == 0 {
        return 0.0;
    }
    (2.0 * shared as f64) / (total as f64)
}

/// Propõe pares quase-duplicados para revisão (`compact`), sem escrever nada.
#[must_use]
pub fn propose_merges(index: &Index, thresholds: &DedupThresholds) -> Vec<MergeProposal> {
    let allowed: BTreeSet<String> = index.docs.iter().map(|doc| doc.meta.id.clone()).collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proposals = Vec::new();
    for doc in &index.docs {
        for hit in index
            .score(&doc.statement, &allowed)
            .into_iter()
            .take(MAX_CANDIDATES)
        {
            if hit.id == doc.meta.id {
                continue;
            }
            let Some(other) = index.docs.iter().find(|other| other.meta.id == hit.id) else {
                continue;
            };
            let score = dice(doc, other);
            if score < thresholds.merge_below {
                continue;
            }
            let (keep, drop) = order_pair(doc, other);
            if !seen.insert((keep.clone(), drop.clone())) {
                continue;
            }
            proposals.push(MergeProposal {
                keep,
                drop,
                score,
                reason: format!("similaridade {score:.2} ≥ {:.2}", thresholds.merge_below),
            });
        }
    }
    proposals.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.keep.cmp(&b.keep))
            .then_with(|| a.drop.cmp(&b.drop))
    });
    proposals
}

fn order_pair<'a>(a: &'a NoteDoc, b: &'a NoteDoc) -> (String, String) {
    let a_key = (a.meta.created_ms, a.meta.id.as_str());
    let b_key = (b.meta.created_ms, b.meta.id.as_str());
    if a_key <= b_key {
        (a.meta.id.clone(), b.meta.id.clone())
    } else {
        (b.meta.id.clone(), a.meta.id.clone())
    }
}

fn term_set(doc: &NoteDoc) -> BTreeSet<&str> {
    let mut set = BTreeSet::new();
    for field in Field::ALL {
        if let Some(entry) = doc.fields.get(&field) {
            set.extend(entry.tf.keys().map(String::as_str));
        }
    }
    set
}
