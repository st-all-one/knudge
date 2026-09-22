//! Confiança derivada em tempo de consulta (D87, E09-T07).
//!
//! A `confidence` **declarada** no frontmatter é uma promessa do autor; a confiança
//! **derivada** é calculada no `recall` a partir de evidência, feedback, idade e drift de
//! âncoras e **nunca é armazenada**. O resultado é sempre `[0,1]`, com pisos para não zerar.

/// Peso do termo de feedback (confirmação + feedback explícito).
pub const FEEDBACK_WEIGHT: f64 = 0.2;

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
}

impl Default for ConfidenceInput {
    fn default() -> Self {
        Self {
            similarity: 0.0,
            confirmation: 0.0,
            drift: 0.0,
            age_days: 0.0,
            feedback: 0.0,
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
    clamp01(base + feedback)
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}
