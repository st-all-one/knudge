//! Confiança derivada em tempo de consulta (D87, E09-T07).
//!
//! A `confidence` **declarada** no frontmatter é uma promessa do autor; a confiança
//! **derivada** é calculada no `recall` a partir de evidência, feedback, idade e drift de
//! âncoras e **nunca é armazenada**. O resultado é sempre `[0,1]`, com pisos para não zerar.

use crate::retrieval::Index;
use crate::retrieval::filter::{Meta, anchor_matches};

/// Peso do termo de feedback (confirmação + feedback explícito).
pub const FEEDBACK_WEIGHT: f64 = 0.2;

/// Peso padrão da confirmação derivada de tarefas (`recall.confirmation_from_tasks` — D108).
pub const DEFAULT_TASK_CONFIRMATION: f64 = 0.1;

/// Piso do fator de drift: uma âncora muito desviada ainda mantém metade do peso.
pub const DRIFT_FLOOR: f64 = 0.5;

/// Meia-vida da idade (dias): o fator cai à metade a cada `AGE_HALF_LIFE_DAYS`.
pub const AGE_HALF_LIFE_DAYS: f64 = 90.0;

/// Entradas da confiança derivada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidenceInput {
    /// Similaridade normalizada `0..=1` (quanto maior, mais relevante).
    pub similarity: f64,
    /// Confirmação derivada de `outcomes` (`success + partial*0.5`).
    pub confirmation: f64,
    /// Drift de âncoras `0..=1` (0 = sem drift; 1 = totalmente desviado).
    pub drift: f64,
    /// Idade em dias.
    pub age_days: f64,
    /// Feedback explícito (confirmações menos contradições, normalizado).
    pub feedback: f64,
    /// Confirmação derivada de tarefas com `outcomes` de sucesso que compartilham âncoras
    /// (X1/D108) — **já ponderada** por `recall.confirmation_from_tasks`.
    pub task_confirmation: f64,
}

impl Default for ConfidenceInput {
    fn default() -> Self {
        Self {
            similarity: 0.0,
            confirmation: 0.0,
            drift: 0.0,
            age_days: 0.0,
            feedback: 0.0,
            task_confirmation: 0.0,
        }
    }
}

/// Fator de drift em `[DRIFT_FLOOR, 1]` — decrescente no drift.
#[must_use]
pub fn drift_factor(drift: f64) -> f64 {
    let drift = clamp01(drift);
    (1.0 - DRIFT_FLOOR).mul_add(-drift, 1.0)
}

/// Fator de idade em `(0, 1]` — decrescente na idade (meia-vida em dias).
#[must_use]
pub fn age_factor(age_days: f64) -> f64 {
    let age = age_days.max(0.0);
    1.0 / (1.0 + age / AGE_HALF_LIFE_DAYS)
}

/// Confiança derivada em `[0,1]` (monotônica em `similarity`/`confirmation`/`feedback`,
/// decrescente em `drift`/`age_days`).
#[must_use]
pub fn confidence_score(input: &ConfidenceInput) -> f64 {
    let base = clamp01(input.similarity) * drift_factor(input.drift) * age_factor(input.age_days);
    let feedback = FEEDBACK_WEIGHT * (input.confirmation.max(0.0) + input.feedback);
    clamp01(base + feedback + input.task_confirmation.max(0.0))
}

/// `true` se a nota é uma **tarefa** (tem `scope`) com `outcomes` de sucesso (D108).
#[must_use]
pub fn is_success_task(meta: &Meta) -> bool {
    meta.scope.is_some() && meta.confirmation > 0.0
}

/// `true` se as âncoras das duas notas se cruzam (qualquer direção de glob — D81).
#[must_use]
fn shares_anchor(left: &Meta, right: &Meta) -> bool {
    left.anchors
        .iter()
        .any(|a| right.anchors.iter().any(|b| anchor_matches(a, b)))
}

/// Confirmação derivada de tarefas de sucesso que compartilham âncoras (X1/D108).
///
/// `Σ weight` por tarefa distinta, saturado em `1.0`. A distância no grafo colapsa a `0`
/// porque o vínculo tarefa↔nota é a **âncora**, não uma aresta (DIVERGENCES #35).
#[must_use]
pub fn from_tasks_with(meta: &Meta, confirmers: &[&Meta], weight: f64) -> f64 {
    let weight = weight.max(0.0);
    if weight == 0.0 || meta.anchors.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for confirmer in confirmers {
        if shares_anchor(meta, confirmer) {
            total += weight;
        }
    }
    clamp01(total)
}

/// Confirmação derivada de tarefas (X1/D108) — varre o índice uma vez.
#[must_use]
pub fn from_tasks(meta: &Meta, index: &Index, weight: f64) -> f64 {
    let confirmers: Vec<&Meta> = index
        .docs
        .iter()
        .map(|doc| &doc.meta)
        .filter(|meta| is_success_task(meta))
        .collect();
    from_tasks_with(meta, &confirmers, weight)
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}
