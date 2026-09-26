//! Similaridade lexical e propostas de merge (D26/D47/D80).

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::retrieval::token::content_terms;
use crate::retrieval::{Field, Index, NoteDoc};
use crate::schema::Status;

use super::{DedupThresholds, MAX_CANDIDATES, MergeProposal};

/// `true` se a nota ainda participa do dedup (`forgotten`/`superseded` ficam fora — D43).
fn is_live(doc: &NoteDoc) -> bool {
    !matches!(doc.meta.status, Status::Forgotten | Status::Superseded)
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
/// Usa uma peneira de postings (E15-T04/O3): só pontua documentos que compartilham ≥1 termo,
/// preservando a fórmula BM25 e a **ordem de soma doc-major** de [`Index::score`] (o resultado é
/// idêntico, byte a byte). Conjuntos de termos e termos da consulta são cacheados uma vez.
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
    let postings = build_postings(&full_sets);
    let mut sieve = Sieve::new(index.docs.len());
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proposals = Vec::new();
    for (position, doc) in index.docs.iter().enumerate() {
        if !live.get(position).is_some_and(|live| *live) {
            continue;
        }
        let Some(terms) = query_terms.get(position) else {
            continue;
        };
        let candidates = sieve.candidates(&postings, terms);
        for candidate in top_candidates(index, &live, candidates, terms, position) {
            let Some(other) = index.docs.get(candidate) else {
                continue;
            };
            // Itens com `scope` (task/issue/grupo) comparam só o `statement`: o corpo costuma
            // ser template de import e não identifica a nota (E08/D80).
            let scoped = doc.meta.scope.is_some() || other.meta.scope.is_some();
            let similarity = if scoped {
                dice_at(&stmt_sets, position, candidate).map(|score| (score, "statement"))
            } else {
                dice_at(&full_sets, position, candidate).map(|score| (score, "similaridade"))
            };
            let Some((score, basis)) = similarity else {
                continue;
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

/// Posições dos melhores candidatos (≤ [`MAX_CANDIDATES`]) por BM25, a partir da peneira.
fn top_candidates(
    index: &Index,
    live: &[bool],
    candidates: &[u32],
    terms: &[Cow<'_, str>],
    position: usize,
) -> Vec<usize> {
    let mut ranked: Vec<(usize, f64)> = candidates
        .iter()
        .filter_map(|&candidate| {
            let candidate = usize::try_from(candidate).ok()?;
            let other = index.docs.get(candidate)?;
            if candidate == position || !live.get(candidate).is_some_and(|live| *live) {
                return None;
            }
            let base = index.score_doc(other, terms);
            (base > 0.0).then_some((candidate, base))
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.1.total_cmp(&a.1).then_with(|| {
            let a_id = index.docs.get(a.0).map_or("", |doc| doc.meta.id.as_str());
            let b_id = index.docs.get(b.0).map_or("", |doc| doc.meta.id.as_str());
            a_id.cmp(b_id)
        })
    });
    ranked
        .into_iter()
        .take(MAX_CANDIDATES)
        .map(|(position, _score)| position)
        .collect()
}

/// Similaridade Dice entre dois documentos de `sets` (por posição), se ambos existirem.
fn dice_at(sets: &[BTreeSet<&str>], left: usize, right: usize) -> Option<f64> {
    Some(dice_sets(sets.get(left)?, sets.get(right)?))
}

/// Índice invertido `termo → posições` (reconstruído a cada chamada, nunca persistido).
fn build_postings<'a>(sets: &[BTreeSet<&'a str>]) -> BTreeMap<&'a str, Vec<u32>> {
    let mut postings: BTreeMap<&'a str, Vec<u32>> = BTreeMap::new();
    for (position, set) in sets.iter().enumerate() {
        let position = u32::try_from(position).unwrap_or(u32::MAX);
        for term in set {
            postings.entry(*term).or_default().push(position);
        }
    }
    postings
}

/// Máscara reutilizável da peneira: marca as posições alcançadas por ≥1 termo e zera depois.
///
/// Evita `BTreeSet` por documento (caro em vocabulário denso): cada posição é marcada/desmarcada
/// uma vez, em tempo linear no tamanho das postings consultadas.
struct Sieve {
    marked: Vec<bool>,
    positions: Vec<u32>,
}

impl Sieve {
    fn new(len: usize) -> Self {
        Self {
            marked: vec![false; len],
            positions: Vec::new(),
        }
    }

    /// Posições de documentos que compartilham ≥1 termo (sem repetição), ordenadas por doc.
    fn candidates<'a>(
        &'a mut self,
        postings: &BTreeMap<&str, Vec<u32>>,
        terms: &[Cow<'_, str>],
    ) -> &'a [u32] {
        self.positions.clear();
        for term in terms {
            let Some(list) = postings.get(term.as_ref()) else {
                continue;
            };
            for &position in list {
                let Ok(slot) = usize::try_from(position) else {
                    continue;
                };
                let Some(marked) = self.marked.get_mut(slot) else {
                    continue;
                };
                if !*marked {
                    *marked = true;
                    self.positions.push(position);
                }
            }
        }
        for &position in &self.positions {
            if let Ok(slot) = usize::try_from(position)
                && let Some(marked) = self.marked.get_mut(slot)
            {
                *marked = false;
            }
        }
        &self.positions
    }
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
