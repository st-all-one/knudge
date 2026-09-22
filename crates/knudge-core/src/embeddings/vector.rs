//! Álgebra de vetores de embedding (E11-T01).
//!
//! Vetores são `f32`, **normalizados** na entrada do índice. A similaridade default é cosseno
//! (D79); o produto interno é oferecido para modelos dot-product. As conversões de/para o codec
//! JSON ficam confinadas a [`from_f64`]/[`to_f64`].

use super::meta::Similarity;

/// Norma L2 do vetor.
#[must_use]
pub fn l2_norm(vector: &[f32]) -> f32 {
    vector.iter().map(|value| value * value).sum::<f32>().sqrt()
}

/// Normaliza o vetor in-place (no-op para vetor nulo).
pub fn normalize(vector: &mut [f32]) {
    let norm = l2_norm(vector);
    if norm > 0.0 {
        for value in &mut *vector {
            *value /= norm;
        }
    }
}

/// Produto interno.
#[must_use]
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Similaridade de cosseno (0.0 se algum vetor for nulo).
#[must_use]
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let denom = l2_norm(a) * l2_norm(b);
    if denom > 0.0 { dot(a, b) / denom } else { 0.0 }
}

/// Similaridade conforme a métrica.
#[must_use]
pub fn similarity(a: &[f32], b: &[f32], metric: Similarity) -> f32 {
    match metric {
        Similarity::Cosine => cosine(a, b),
        Similarity::Dot => dot(a, b),
    }
}

/// `true` se o vetor já está normalizado (tolerância `eps`).
#[must_use]
pub fn is_normalized(vector: &[f32], eps: f32) -> bool {
    (l2_norm(vector) - 1.0).abs() <= eps
}

/// Converte `f64` (JSON) em `f32`.
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "vetor de embedding em f32; perda de precisão aceita no codec"
)]
#[must_use]
pub fn from_f64(value: f64) -> f32 {
    value as f32
}

/// Converte `f32` em `f64` (JSON), sem perda.
#[must_use]
pub fn to_f64(value: f32) -> f64 {
    f64::from(value)
}
