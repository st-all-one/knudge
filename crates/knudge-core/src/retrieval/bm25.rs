//! BM25 com IDF por campo e boost por confirmação (D35–D38).
//!
//! `k1=1.5`, `b=0.75`. O IDF é calculado **por campo** (o `statement` domina, por peso e por
//! IDF próprio), há **peso por tipo**, e a confirmação derivada de `outcomes` aplica
//! `score * (1 + 0.1 * (success + partial*0.5))`.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

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
        let mut hits = if self.sieve_pays_off(&terms) {
            let positions = self.postings().sieve(&terms);
            self.collect_hits(positions.into_iter(), &terms, allowed, &boost)
        } else {
            let all = (0..self.docs.len()).filter_map(|position| u32::try_from(position).ok());
            self.collect_hits(all, &terms, allowed, &boost)
        };
        hits.sort_unstable_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
        hits
    }

    /// Pontua as posições dadas, devolvendo os hits (ordem doc-major preservada).
    fn collect_hits<F: Fn(&Meta) -> f64>(
        &self,
        positions: impl Iterator<Item = u32>,
        terms: &[Cow<'_, str>],
        allowed: &BTreeSet<String>,
        boost: &F,
    ) -> Vec<Bm25Hit> {
        let mut hits = Vec::new();
        for position in positions {
            let Ok(position) = usize::try_from(position) else {
                continue;
            };
            let Some(doc) = self.docs.get(position) else {
                continue;
            };
            if !allowed.contains(&doc.meta.id) {
                continue;
            }
            let base = self.score_doc(doc, terms);
            if base <= 0.0 {
                continue;
            }
            let score = base * (1.0 + boost(&doc.meta).max(0.0));
            hits.push(Bm25Hit {
                id: doc.meta.id.clone(),
                score,
            });
        }
        hits
    }

    /// Estima se a peneira compensa (O2.1).
    ///
    /// Soma a maior document frequency por termo (limite superior dos candidatos). Se os termos
    /// cobrem metade ou mais do corpus, construir o índice invertido custa mais do que varrer.
    fn sieve_pays_off(&self, terms: &[Cow<'_, str>]) -> bool {
        let n = self.docs.len();
        if n == 0 {
            return false;
        }
        let mut reach = 0usize;
        for term in terms {
            let df = Field::ALL
                .iter()
                .filter_map(|field| self.stats.df.get(field)?.get(term.as_ref()))
                .copied()
                .max()
                .unwrap_or(0);
            reach = reach.saturating_add(usize::try_from(df).unwrap_or(usize::MAX));
        }
        reach.saturating_mul(2) < n
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::suboptimal_flops,
        reason = "fórmula BM25 em f64 com termos não negativos"
    )]
    fn field_sum(&self, doc: &NoteDoc, terms: &[Cow<'_, str>], field: Field) -> f64 {
        let weight = field.weight();
        let length = f64::from(doc.len(field));
        let avg = self.stats.avg_len.get(&field).copied().unwrap_or(0.0);
        let norm = if avg > 0.0 {
            1.0 - B + B * length / avg
        } else {
            1.0
        };
        let df_terms = self.stats.df.get(&field);
        let mut total = 0.0;
        for term in terms {
            let tf = doc.tf(field, term.as_ref());
            if tf == 0 {
                continue;
            }
            let idf = idf_from(df_terms, self.stats.n, term.as_ref());
            let tf = f64::from(tf);
            let denom = tf + K1 * norm;
            if denom > 0.0 {
                total += weight * idf * (tf * (K1 + 1.0)) / denom;
            }
        }
        total
    }

    /// Soma crua dos campos (sem peso de tipo/confirmação).
    fn raw_score(&self, doc: &NoteDoc, terms: &[Cow<'_, str>]) -> f64 {
        Field::ALL
            .iter()
            .map(|field| self.field_sum(doc, terms, *field))
            .sum()
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::suboptimal_flops,
        reason = "multiplicação em f64 com fatores não negativos"
    )]
    #[must_use]
    /// Score BM25 final de um **documento** para termos já tokenizados (peso de tipo e boost de
    /// confirmação inclusos).
    ///
    /// Exposto para a peneira do dedup (E15-T04/O3), que pontua só os candidatos que
    /// compartilham ≥1 termo em vez de varrer o corpus inteiro.
    pub fn score_doc(&self, doc: &NoteDoc, terms: &[Cow<'_, str>]) -> f64 {
        self.raw_score(doc, terms)
            * type_weight(doc.meta.note_type)
            * (1.0 + CONFIRMATION_STEP * doc.meta.confirmation)
    }

    /// Fração do score BM25 cru que veio do campo `body`, em `[0,1]` (D161).
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "divisão em f64 com divisor > 0"
    )]
    pub fn body_share(&self, doc: &NoteDoc, query: &str) -> f64 {
        let terms = content_terms(query);
        if terms.is_empty() {
            return 0.0;
        }
        let raw = self.raw_score(doc, &terms);
        if raw <= 0.0 {
            return 0.0;
        }
        (self.field_sum(doc, &terms, Field::Body) / raw).clamp(0.0, 1.0)
    }

    /// IDF de um termo no campo (via document frequency do campo).
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "fórmula IDF em f64; `n` e `df` não negativos"
    )]
    pub fn idf(&self, field: Field, term: &str) -> f64 {
        idf_from(self.stats.df.get(&field), self.stats.n, term)
    }
}

/// IDF de um termo a partir do mapa de document frequency do campo (D37).
///
/// Fatorado para o `field_sum` reusar o mapa hoisted e economizar uma busca por termo (O2.2).
#[allow(
    clippy::arithmetic_side_effects,
    reason = "fórmula IDF em f64; `n` e `df` não negativos"
)]
fn idf_from(df_terms: Option<&BTreeMap<String, u32>>, n: u32, term: &str) -> f64 {
    let df = df_terms
        .and_then(|terms| terms.get(term))
        .copied()
        .unwrap_or(0);
    let n = f64::from(n);
    let df = f64::from(df);
    ((n - df + 0.5) / (df + 0.5)).ln_1p()
}
