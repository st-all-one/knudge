//! Escopo `lifecycle`: decay, confiança derivada e clusters (E09/E10).
//!
//! A confiança derivada (E09-T07) já vive aqui; o decay e os clusters entram em E10.

pub mod confidence;

#[cfg(test)]
mod tests;

pub use confidence::{ConfidenceInput, age_factor, confidence_score, drift_factor};
