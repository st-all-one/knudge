//! Similaridade lexical e propostas de merge (D26/D47/D80).

use std::collections::BTreeSet;

use crate::retrieval::{Field, Index, NoteDoc};

use super::{DedupThresholds, MAX_CANDIDATES, MergeProposal, live_ids};

/// Similaridade Dice sobre o conjunto de termos de dois documentos.
#[must_use]
pub fn dice(a: &NoteDoc, b: &NoteDoc) -> f64 {
    dice_sets(&term_set(a), &term_set(b))
}

/// Similaridade Dice restrita a `statement`.
///
/// Itens de trabalho/grupo de import costumam compartilhar o **corpo** (template) e diferir no
/// `statement`; comparar o corpo inteiro produz quase-duplicatas em massa (E08/D80).
#[must_use]
pub fn dice_statement(a: &NoteDoc, b: &NoteDoc) -> f64 {
    dice_sets(
        &term_set_field(a, Field::Statement),
        &term_set_field(b, Field::Statement),
    )
}

#[allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "contagens de termos cabem em f64; soma com domínio limitado ao vocabulário"
)]
fn dice_sets(a: &BTreeSet<&str>, b: &BTreeSet<&str>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let shared = a.intersection(b).count();
    let total = a.len().saturating_add(b.len());
    if total == 0 {
        return 0.0;
    }
    (2.0 * shared as f64) / (total as f64)
}

/// Propõe pares quase-duplicados para revisão (`compact`), sem escrever nada.
#[must_use]
pub fn propose_merges(index: &Index, thresholds: &DedupThresholds) -> Vec<MergeProposal> {
    let allowed = live_ids(index);
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proposals = Vec::new();
    for doc in &index.docs {
        if !allowed.contains(&doc.meta.id) {
            continue;
        }
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
            // Itens com `scope` (task/issue/grupo) comparam só o `statement`: o corpo costuma
            // ser template de import e não identifica a nota (E08/D80).
            let scoped = doc.meta.scope.is_some() || other.meta.scope.is_some();
            let (score, basis) = if scoped {
                (dice_statement(doc, other), "statement")
            } else {
                (dice(doc, other), "similaridade")
            };
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
                reason: format!("{basis} {score:.2} ≥ {:.2}", thresholds.merge_below),
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

fn term_set_field(doc: &NoteDoc, field: Field) -> BTreeSet<&str> {
    doc.fields
        .get(&field)
        .map(|entry| entry.tf.keys().map(String::as_str).collect())
        .unwrap_or_default()
}
