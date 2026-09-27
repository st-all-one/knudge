//! Peneira de postings da peneira lexical exata (E15-T04/O3).

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::retrieval::Index;

use super::MAX_CANDIDATES;

/// Posições dos melhores candidatos (≤ [`MAX_CANDIDATES`]) por BM25, a partir da peneira.
pub(super) fn top_candidates(
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
    ranked.sort_unstable_by(|a, b| {
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

/// Índice invertido `termo → posições` (reconstruído a cada chamada, nunca persistido).
pub(super) fn build_postings<'a>(sets: &[BTreeSet<&'a str>]) -> BTreeMap<&'a str, Vec<u32>> {
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
pub(super) struct Sieve {
    marked: Vec<bool>,
    positions: Vec<u32>,
}

impl Sieve {
    pub(super) fn new(len: usize) -> Self {
        Self {
            marked: vec![false; len],
            positions: Vec::new(),
        }
    }

    /// Posições de documentos que compartilham ≥1 termo (sem repetição), ordenadas por doc.
    pub(super) fn candidates<'a>(
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
