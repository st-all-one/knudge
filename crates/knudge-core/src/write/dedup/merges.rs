//! Similaridade lexical e propostas de merge (D26/D47/D80).

use std::borrow::Cow;
use std::collections::BTreeSet;

use crate::retrieval::token::content_terms;
use crate::retrieval::{Field, Index, NoteDoc};

use super::lsh;
use super::sieve::{Sieve, build_postings, top_candidates};
use super::{DedupThresholds, MergeProposal};

/// `true` se a nota ainda participa do dedup (`forgotten`/`superseded` ficam fora — D43).
fn is_live(doc: &NoteDoc) -> bool {
    !doc.meta.status.is_deprecated()
}

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
///
/// Abaixo de [`lsh::MIN_LSH_CORPUS`] usa a peneira de postings (E15-T04/O3): só pontua documentos
/// que compartilham ≥1 termo, preservando a fórmula BM25 e a **ordem de soma doc-major** de
/// [`Index::score`] (o resultado é idêntico, byte a byte). Acima do limiar, usa *blocking*
/// MinHash/LSH (E19/T07/D204), que reduz os pares candidatos em vocabulário denso.
#[must_use]
pub fn propose_merges(index: &Index, thresholds: &DedupThresholds) -> Vec<MergeProposal> {
    let live: Vec<bool> = index.docs.iter().map(is_live).collect();
    let full_sets: Vec<BTreeSet<&str>> = index.docs.iter().map(term_set).collect();
    let stmt_sets: Vec<BTreeSet<&str>> = index
        .docs
        .iter()
        .map(|doc| term_set_field(doc, Field::Statement))
        .collect();
    let query_terms: Vec<Vec<Cow<'_, str>>> = index
        .docs
        .iter()
        .map(|doc| content_terms(&doc.statement))
        .collect();
    let pairs = Pairs {
        index,
        thresholds,
        full: &full_sets,
        stmt: &stmt_sets,
    };
    let postings = build_postings(&full_sets);
    // O LSH só compensa quando o vocabulário é denso (algum termo aparece em muitos docs); em
    // corpus esparso a peneira lexical exata já é O(N).
    let dense = index.docs.len() >= lsh::MIN_LSH_CORPUS
        && postings.values().any(|list| list.len() >= DENSE_MIN_DF);
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proposals = Vec::new();
    if dense {
        let signatures: Vec<[u64; lsh::SIGNATURE_LEN]> = query_terms
            .iter()
            .map(|terms| lsh::signature(terms.iter().map(Cow::as_ref)))
            .collect();
        for (left, right) in lsh::candidate_pairs(&signatures) {
            let live_pair = live.get(left).is_some_and(|live| *live)
                && live.get(right).is_some_and(|live| *live);
            if live_pair {
                push_proposal(&pairs, left, right, &mut seen, &mut proposals);
            }
        }
    } else {
        let mut sieve = Sieve::new(index.docs.len());
        for (position, _doc) in index.docs.iter().enumerate() {
            if !live.get(position).is_some_and(|live| *live) {
                continue;
            }
            let Some(terms) = query_terms.get(position) else {
                continue;
            };
            let candidates = sieve.candidates(&postings, terms);
            for candidate in top_candidates(index, &live, candidates, terms, position) {
                push_proposal(&pairs, position, candidate, &mut seen, &mut proposals);
            }
        }
    }
    proposals.sort_unstable_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.keep.cmp(&b.keep))
            .then_with(|| a.drop.cmp(&b.drop))
    });
    proposals
}

/// Frequência mínima de documento que sinaliza vocabulário denso (ver `propose_merges`).
const DENSE_MIN_DF: usize = 8;

/// Contexto imutável para avaliar um par (conjuntos de termos + limiar).
struct Pairs<'a> {
    index: &'a Index,
    thresholds: &'a DedupThresholds,
    full: &'a [BTreeSet<&'a str>],
    stmt: &'a [BTreeSet<&'a str>],
}

impl Pairs<'_> {
    /// Avalia o par `(left, right)`; devolve a proposta se a similaridade cruzar o limiar.
    fn evaluate(&self, left: usize, right: usize) -> Option<MergeProposal> {
        let doc = self.index.docs.get(left)?;
        let other = self.index.docs.get(right)?;
        // Itens com `scope` (task/issue/grupo) comparam só o `statement`: o corpo costuma
        // ser template de import e não identifica a nota (E08/D80).
        let scoped = doc.meta.scope.is_some() || other.meta.scope.is_some();
        let (score, basis) = if scoped {
            (dice_at(self.stmt, left, right)?, "statement")
        } else {
            (dice_at(self.full, left, right)?, "similaridade")
        };
        if score < self.thresholds.merge_below {
            return None;
        }
        let (keep, drop) = order_pair(doc, other);
        Some(MergeProposal {
            keep,
            drop,
            score,
            reason: format!("{basis} {score:.2} ≥ {:.2}", self.thresholds.merge_below),
        })
    }
}

/// Avalia o par e acumula a proposta (dedup por par ordenado `keep`/`drop`).
fn push_proposal(
    pairs: &Pairs<'_>,
    left: usize,
    right: usize,
    seen: &mut BTreeSet<(String, String)>,
    proposals: &mut Vec<MergeProposal>,
) {
    let Some(proposal) = pairs.evaluate(left, right) else {
        return;
    };
    if seen.insert((proposal.keep.clone(), proposal.drop.clone())) {
        proposals.push(proposal);
    }
}

/// Similaridade Dice entre dois documentos de `sets` (por posição), se ambos existirem.
fn dice_at(sets: &[BTreeSet<&str>], left: usize, right: usize) -> Option<f64> {
    Some(dice_sets(sets.get(left)?, sets.get(right)?))
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
