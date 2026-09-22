//! `compact` como proposta revisável (D47, E08-T08).
//!
//! Funde acúmulo gradual de redundância, mas **nunca em silêncio**: gera propostas com
//! estratégia (`concat`/`keep_latest`/`merge_outcomes`) que o agente aceita ou rejeita.

use crate::Result;
use crate::retrieval::Index;
use crate::schema::Value;
use crate::store::{Note, Store};
use crate::write::dedup::{DedupThresholds, propose_merges};
use crate::write::{WriteContext, forget, merge_into};

/// Estratégia de consolidação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactStrategy {
    /// Concatena os corpos.
    Concat,
    /// Mantém o mais recente.
    KeepLatest,
    /// Funde `outcomes` e corpos.
    MergeOutcomes,
}

impl CompactStrategy {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concat => "concat",
            Self::KeepLatest => "keep_latest",
            Self::MergeOutcomes => "merge_outcomes",
        }
    }
}

/// Proposta de consolidação (nunca aplicada sem aceite).
#[derive(Debug, Clone, PartialEq)]
pub struct CompactProposal {
    /// Estratégia sugerida.
    pub strategy: CompactStrategy,
    /// Ids envolvidos.
    pub ids: Vec<String>,
    /// Id que permanece.
    pub keep: String,
    /// Motivo legível.
    pub why: String,
    /// Similaridade.
    pub score: f64,
}

/// Resultado de [`apply_compact`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    /// Nota que permaneceu.
    pub kept: String,
    /// Notas esquecidas (soft).
    pub forgotten: Vec<String>,
}

/// Propõe consolidações a partir de quase-duplicatas.
#[must_use]
pub fn propose_compact(
    store: &Store<'_>,
    index: &Index,
    thresholds: &DedupThresholds,
) -> Vec<CompactProposal> {
    propose_merges(index, thresholds)
        .into_iter()
        .map(|merge| {
            let strategy = choose_strategy(store, &merge.keep, &merge.drop);
            CompactProposal {
                strategy,
                ids: vec![merge.keep.clone(), merge.drop.clone()],
                keep: merge.keep,
                why: merge.reason,
                score: merge.score,
            }
        })
        .collect()
}

/// Aplica uma proposta aceita: funde no `keep` e esquece os demais (soft).
///
/// # Errors
/// Propaga erros de leitura/escrita; retorna `ErrorKind::NotFound` se o `keep` não existir.
pub fn apply_compact(ctx: &WriteContext<'_>, proposal: &CompactProposal) -> Result<ApplyOutcome> {
    let _keep = ctx.store().read(&proposal.keep)?;
    let mut forgotten = Vec::new();
    for id in &proposal.ids {
        if id == &proposal.keep {
            continue;
        }
        let incoming = ctx.store().read(id)?;
        merge_into(ctx, &proposal.keep, &incoming)?;
        forget(ctx, id, Some("compact"))?;
        forgotten.push(id.clone());
    }
    Ok(ApplyOutcome {
        kept: proposal.keep.clone(),
        forgotten,
    })
}

fn choose_strategy(store: &Store<'_>, keep: &str, drop: &str) -> CompactStrategy {
    let keep_has = store.read(keep).is_ok_and(|note| has_outcomes(&note));
    let drop_has = store.read(drop).is_ok_and(|note| has_outcomes(&note));
    if keep_has && drop_has {
        return CompactStrategy::MergeOutcomes;
    }
    if keep_has || drop_has {
        return CompactStrategy::KeepLatest;
    }
    CompactStrategy::Concat
}

fn has_outcomes(note: &Note) -> bool {
    note.frontmatter
        .get("outcomes")
        .and_then(Value::as_list)
        .is_some_and(|items| !items.is_empty())
}
