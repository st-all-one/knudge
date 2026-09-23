//! Testes da confiança derivada (E09-T07).

use proptest::prelude::{ProptestConfig, prop_assert, proptest};

use crate::lifecycle::confidence::{
    AGE_HALF_LIFE_DAYS, ConfidenceInput, age_factor, confidence_score, drift_factor,
};

#[test]
fn result_is_always_in_unit_range() {
    let extreme = ConfidenceInput {
        similarity: 10.0,
        confirmation: 10.0,
        drift: 10.0,
        age_days: 1_000_000.0,
        feedback: 10.0,
        task_confirmation: 10.0,
    };
    let score = confidence_score(&extreme);
    assert!((0.0..=1.0).contains(&score), "score fora de [0,1]: {score}");
}

#[test]
fn factors_have_expected_anchors() {
    assert!((drift_factor(0.0) - 1.0).abs() < 1e-9);
    assert!((drift_factor(1.0) - 0.5).abs() < 1e-9);
    assert!((age_factor(0.0) - 1.0).abs() < 1e-9);
    assert!((age_factor(AGE_HALF_LIFE_DAYS) - 0.5).abs() < 1e-9);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn monotone_in_similarity_and_confirmation(
        similarity in 0.0_f64..1.0,
        delta in 0.0_f64..1.0,
        confirmation in 0.0_f64..1.0,
    ) {
        let low = ConfidenceInput { similarity, confirmation, ..Default::default() };
        let high = ConfidenceInput { similarity: (similarity + delta).min(1.0), confirmation, ..Default::default() };
        prop_assert!(confidence_score(&high) >= confidence_score(&low) - 1e-12);

        let less = ConfidenceInput { similarity, confirmation, ..Default::default() };
        let more = ConfidenceInput { similarity, confirmation: (confirmation + delta).min(1.0), ..Default::default() };
        prop_assert!(confidence_score(&more) >= confidence_score(&less) - 1e-12);
    }

    #[test]
    fn monotone_in_task_confirmation(
        similarity in 0.0_f64..1.0,
        delta in 0.0_f64..1.0,
        task_confirmation in 0.0_f64..1.0,
    ) {
        let low = ConfidenceInput { similarity, task_confirmation, ..Default::default() };
        let high = ConfidenceInput { similarity, task_confirmation: (task_confirmation + delta).min(1.0), ..Default::default() };
        prop_assert!(confidence_score(&high) >= confidence_score(&low) - 1e-12);
    }

    #[test]
    fn decreasing_in_drift_and_age(
        similarity in 0.0_f64..1.0,
        drift in 0.0_f64..1.0,
        delta in 0.0_f64..1.0,
        age_days in 0.0_f64..100_000.0,
    ) {
        let low = ConfidenceInput { similarity, drift, age_days, ..Default::default() };
        let high = ConfidenceInput { similarity, drift: (drift + delta).min(1.0), age_days, ..Default::default() };
        prop_assert!(confidence_score(&high) <= confidence_score(&low) + 1e-12);

        let young = ConfidenceInput { similarity, drift, age_days, ..Default::default() };
        let old = ConfidenceInput { similarity, drift, age_days: delta.mul_add(100.0, age_days), ..Default::default() };
        prop_assert!(confidence_score(&old) <= confidence_score(&young) + 1e-12);
    }

    #[test]
    fn always_within_unit_range(
        similarity in -5.0_f64..5.0,
        confirmation in -5.0_f64..5.0,
        drift in -5.0_f64..5.0,
        age_days in 0.0_f64..1_000_000.0,
        feedback in -5.0_f64..5.0,
        task_confirmation in -5.0_f64..5.0,
    ) {
        let score = confidence_score(&ConfidenceInput { similarity, confirmation, drift, age_days, feedback, task_confirmation });
        prop_assert!((0.0..=1.0).contains(&score));
        prop_assert!(!score.is_nan());
    }
}
