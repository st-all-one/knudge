//! Métricas de retrieval para decidir o modelo por A/B no corpus (E11-T07, D90).
//!
//! `Recall@k`, `nDCG@k` e `MRR` são funções puras sobre um ranqueamento injetado — testáveis
//! com um golden fixo, sem carregar modelo. É o alicerce do `kd maintenance eval --ab`.

use std::collections::BTreeMap;

/// Caso do golden set.
#[derive(Debug, Clone, PartialEq)]
pub struct GoldenCase {
    /// Consulta.
    pub query: String,
    /// Ids relevantes (ganho default 1.0).
    pub relevant: Vec<String>,
    /// Ganho explícito por id (opcional; vence o default).
    pub gains: BTreeMap<String, f64>,
}

impl GoldenCase {
    /// Caso com ganho 1.0 para todos os relevantes.
    #[must_use]
    pub const fn new(query: String, relevant: Vec<String>) -> Self {
        Self {
            query,
            relevant,
            gains: BTreeMap::new(),
        }
    }

    /// Define o ganho de um id.
    #[must_use]
    pub fn with_gain(mut self, id: impl Into<String>, gain: f64) -> Self {
        self.gains.insert(id.into(), gain);
        self
    }

    /// Ganho do id (0.0 se não relevante).
    #[must_use]
    pub fn gain(&self, id: &str) -> f64 {
        if let Some(gain) = self.gains.get(id) {
            return *gain;
        }
        if self.relevant.iter().any(|relevant| relevant == id) {
            1.0
        } else {
            0.0
        }
    }
}

/// Métricas agregadas de uma avaliação.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalMetrics {
    /// Número de casos.
    pub cases: usize,
    /// `Recall@k` médio.
    pub recall: f64,
    /// `nDCG@k` médio.
    pub ndcg: f64,
    /// `MRR` médio.
    pub mrr: f64,
}

/// Vencedor de uma comparação A/B.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winner {
    /// O modelo A venceu.
    A,
    /// O modelo B venceu.
    B,
    /// Empate.
    Tie,
}

/// Relatório A/B reproduzível.
#[derive(Debug, Clone, PartialEq)]
pub struct AbReport {
    /// Métricas de A.
    pub a: EvalMetrics,
    /// Métricas de B.
    pub b: EvalMetrics,
    /// Vencedor (por `ndcg`, depois `recall`, depois `mrr`).
    pub winner: Winner,
}

/// `Recall@k` de um ranqueamento.
#[must_use]
pub fn recall_at_k(ranked: &[String], relevant: &[String], k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let hits = ranked
        .iter()
        .take(k)
        .filter(|id| relevant.iter().any(|relevant| relevant == *id))
        .count();
    ratio(hits, relevant.len())
}

/// `MRR` de um ranqueamento (0.0 se nenhum relevante aparece).
#[must_use]
pub fn mrr(ranked: &[String], relevant: &[String]) -> f64 {
    for (index, id) in ranked.iter().enumerate() {
        if relevant.iter().any(|relevant| relevant == id) {
            let rank = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
            return 1.0 / f64::from(rank);
        }
    }
    0.0
}

/// `nDCG@k` de um ranqueamento.
#[must_use]
pub fn ndcg_at_k(ranked: &[String], case: &GoldenCase, k: usize) -> f64 {
    let dcg: f64 = ranked
        .iter()
        .take(k)
        .enumerate()
        .map(|(index, id)| discount(case.gain(id), index))
        .sum();
    let mut ideal: Vec<f64> = case.relevant.iter().map(|id| case.gain(id)).collect();
    ideal.sort_by(|a, b| b.total_cmp(a));
    let idcg: f64 = ideal
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, g)| discount(*g, i))
        .sum();
    if idcg > 0.0 { dcg / idcg } else { 0.0 }
}

/// Avalia os casos com um ranqueador injetado.
pub fn evaluate<F>(cases: &[GoldenCase], k: usize, rank: F) -> EvalMetrics
where
    F: Fn(&str) -> Vec<String>,
{
    let mut recall = 0.0;
    let mut ndcg = 0.0;
    let mut reciprocal = 0.0;
    for case in cases {
        let ranked = rank(&case.query);
        recall += recall_at_k(&ranked, &case.relevant, k);
        ndcg += ndcg_at_k(&ranked, case, k);
        reciprocal += mrr(&ranked, &case.relevant);
    }
    let count = cases.len();
    let divisor = f64::from(u32::try_from(count).unwrap_or(u32::MAX));
    let divisor = if divisor > 0.0 { divisor } else { 1.0 };
    EvalMetrics {
        cases: count,
        recall: recall / divisor,
        ndcg: ndcg / divisor,
        mrr: reciprocal / divisor,
    }
}

/// Compara dois relatórios de forma determinística.
#[must_use]
pub fn ab_compare(a: EvalMetrics, b: EvalMetrics) -> AbReport {
    let winner = if approx_gt(b.ndcg, a.ndcg) {
        Winner::B
    } else if approx_gt(a.ndcg, b.ndcg) {
        Winner::A
    } else if approx_gt(b.recall, a.recall) {
        Winner::B
    } else if approx_gt(a.recall, b.recall) {
        Winner::A
    } else if approx_gt(b.mrr, a.mrr) {
        Winner::B
    } else if approx_gt(a.mrr, b.mrr) {
        Winner::A
    } else {
        Winner::Tie
    };
    AbReport { a, b, winner }
}

fn discount(gain: f64, index: usize) -> f64 {
    let position = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(2);
    let weighted = gain.exp2() - 1.0;
    weighted / f64::from(position).log2()
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    f64::from(u32::try_from(numerator).unwrap_or(u32::MAX))
        / f64::from(u32::try_from(denominator).unwrap_or(u32::MAX))
}

fn approx_gt(a: f64, b: f64) -> bool {
    a - b > 1e-9
}
