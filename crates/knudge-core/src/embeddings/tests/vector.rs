//! Testes da álgebra de vetores (E11-T01).

use crate::embeddings::meta::Similarity;
use crate::embeddings::vector::{cosine, dot, is_normalized, l2_norm, normalize, similarity};

#[test]
fn normalize_makes_unit_vector() {
    let mut vector = vec![3.0_f32, 4.0];
    normalize(&mut vector);
    assert!(is_normalized(&vector, 1e-5));
    assert!((l2_norm(&vector) - 1.0).abs() <= 1e-5);
}

#[test]
fn normalize_ignores_zero_vector() {
    let mut vector = vec![0.0_f32, 0.0];
    normalize(&mut vector);
    assert_eq!(vector, vec![0.0_f32, 0.0]);
}

#[test]
fn cosine_and_dot_dispatch() {
    let a = vec![1.0_f32, 0.0];
    let b = vec![0.0_f32, 1.0];
    assert!(cosine(&a, &b).abs() <= 1e-6);
    assert!(dot(&a, &b).abs() <= 1e-6);
    assert!((similarity(&a, &a, Similarity::Cosine) - 1.0).abs() <= 1e-6);
    assert!((similarity(&a, &a, Similarity::Dot) - 1.0).abs() <= 1e-6);
}

#[test]
fn cosine_handles_zero_norm() {
    let zero = vec![0.0_f32, 0.0];
    let other = vec![1.0_f32, 0.0];
    assert!(cosine(&zero, &other).abs() <= 1e-6);
}
