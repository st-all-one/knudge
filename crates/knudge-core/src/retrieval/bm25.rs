//! BM25 com IDF por campo e boost por confirmação (D35–D38).
//!
//! `k1=1.5`, `b=0.75`. O IDF é calculado **por campo** (o `statement` domina, por peso e por
//! IDF próprio), há **peso por tipo**, e a confirmação derivada de `outcomes` aplica
//! `score * (1 + 0.1 * (success + partial*0.5))`.

use std::collections::BTreeSet;

use crate::retrieval::filter::Meta;
use crate::retrieval::index::{Field, Index, NoteDoc};
use crate::retrieval::token::content_terms;
use crate::schema::NoteType;

/// Constante de saturação de termo do BM25.
pub const K1: f64 = 1.5;

/// Constante de normalização por comprimento do BM25.
pub const B: f64 = 0.75;

/// Passo do boost por confirmação (D38).
pub const CONFIRMATION_STEP: f64 = 0.1;

/// Hit lexical.
#[derive(Debug, Clone, PartialEq)]
pub struct Bm25Hit {
    /// Id do documento.
    pub id: String,
    /// Score BM25 já com pesos/boost.
    pub score: f64,
}

/// Peso por tipo (D37) — tipos de decisão/erro valem mais que containers.
#[must_use]
pub const fn type_weight(note_type: NoteType) -> f64 {
    match note_type {
        NoteType::Decision => 1.20,
        NoteType::Error => 1.15,
        NoteType::Fact => 1.10,
        NoteType::Def | NoteType::Risk => 1.05,
        NoteType::Task | NoteType::Question => 1.00,
        NoteType::Snippet => 0.95,
        NoteType::Link | NoteType::Meta => 0.90,
        NoteType::Epic => 0.70,
    }
}

impl Index {
    /// Pontua o corpus (restrito a `allowed`) por uma consulta livre.
    #[must_use]
    pub fn score(&self, query: &str, allowed: &BTreeSet<String>) -> Vec<Bm25Hit> {
        self.score_with(query, allowed, |_| 0.0)
    }

    /// Como [`Index::score`], mas multiplica o score de cada doc por `1 + boost(meta)`.
    ///
    /// O `boost` é a confirmação derivada de tarefas (X1/D108); o closure só é chamado para
    /// documentos que casam a consulta (o termo lexical é o gate).
    #[must_use]
    pub fn score_with<F: Fn(&Meta) -> f64>(
        &self,
        query: &str,
        allowed: &BTreeSet<String>,
        boost: F,
    ) -> Vec<Bm25Hit> {
        let terms = content_terms(query);
        if terms.is_empty() {
            return Vec::new();
        }
        let mut hits = Vec::new();
        for doc in &self.docs {
            if !allowed.contains(&doc.meta.id) {
                continue;
            }
            let base = self.score_doc(doc, &terms);
            if base <= 0.0 {
                continue;
            }
            let score = base * (1.0 + boost(&doc.meta).max(0.0));
            hits.push(Bm25Hit {
                id: doc.meta.id.clone(),
                score,
            });
        }
        hits.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
        hits
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::suboptimal_flops,
        reason = "fórmula BM25 em f64 com termos não negativos"
    )]
    fn score_doc(&self, doc: &NoteDoc, terms: &[std::borrow::Cow<'_, str>]) -> f64 {
        let mut total = 0.0;
        for term in terms {
            for field in Field::ALL {
                let tf = doc.tf(field, term.as_ref());
                if tf == 0 {
                    continue;
                }
                let idf = self.idf(field, term.as_ref());
                let length = f64::from(doc.len(field));
                let avg = self.stats.avg_len.get(&field).copied().unwrap_or(0.0);
                let norm = if avg > 0.0 {
                    1.0 - B + B * length / avg
                } else {
                    1.0
                };
                let tf = f64::from(tf);
                let denom = tf + K1 * norm;
                if denom > 0.0 {
                    total += field.weight() * idf * (tf * (K1 + 1.0)) / denom;
                }
            }
        }
        total * type_weight(doc.meta.note_type) * (1.0 + CONFIRMATION_STEP * doc.meta.confirmation)
    }

    /// IDF de um termo no campo (via document frequency do campo).
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "fórmula IDF em f64; `n` e `df` não negativos"
    )]
    pub fn idf(&self, field: Field, term: &str) -> f64 {
        let df = self
            .stats
            .df
            .get(&field)
            .and_then(|terms| terms.get(term))
            .copied()
            .unwrap_or(0);
        let n = f64::from(self.stats.n);
        let df = f64::from(df);
        ((n - df + 0.5) / (df + 0.5)).ln_1p()
    }
}
