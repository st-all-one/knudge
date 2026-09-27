//! Testes da confiança derivada (E09-T07/D189).

use proptest::prelude::{ProptestConfig, prop_assert, proptest};

use crate::lifecycle::confidence::{
    AGE_HALF_LIFE_DAYS, ConfidenceInput, age_factor, confidence_score, drift_factor,
};

#[test]
fn result_is_always_in_unit_range() {
    let extreme = ConfidenceInput {
        similarity: 10.0,
        successes: 10.0,
        failures: 10.0,
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

#[test]
fn more_successes_outrank_a_single_one() {
    // O ponto de D189: uma nota com 20 sucessos é mais confiável que uma com 1.
    let one = confidence_score(&ConfidenceInput {
        successes: 1.0,
        ..ConfidenceInput::default()
    });
    let many = confidence_score(&ConfidenceInput {
        successes: 20.0,
        ..ConfidenceInput::default()
    });
    assert!(
        one > 0.0 && one < 0.3,
        "1 sucesso deveria ser conservador: {one}"
    );
    assert!(many > 0.8, "20 sucessos deveriam ser fortes: {many}");
    assert!(many > one);
}

#[test]
fn failures_reduce_confidence() {
    let clean = confidence_score(&ConfidenceInput {
        successes: 5.0,
        ..ConfidenceInput::default()
    });
    let mixed = confidence_score(&ConfidenceInput {
        successes: 5.0,
        failures: 5.0,
        ..ConfidenceInput::default()
    });
    assert!(mixed < clean, "{mixed} deveria ser menor que {clean}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn monotone_in_similarity_and_successes(
        similarity in 0.0_f64..1.0,
        delta in 0.0_f64..1.0,
        successes in 0.0_f64..1.0,
    ) {
        let low = ConfidenceInput { similarity, successes, ..Default::default() };
        let high = ConfidenceInput { similarity: (similarity + delta).min(1.0), successes, ..Default::default() };
        prop_assert!(confidence_score(&high) >= confidence_score(&low) - 1e-12);

        let less = ConfidenceInput { similarity, successes, ..Default::default() };
        let more = ConfidenceInput { similarity, successes: (successes + delta).min(1.0), ..Default::default() };
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
        successes in -5.0_f64..5.0,
        failures in -5.0_f64..5.0,
        drift in -5.0_f64..5.0,
        age_days in 0.0_f64..1_000_000.0,
        feedback in -5.0_f64..5.0,
        task_confirmation in -5.0_f64..5.0,
    ) {
        let score = confidence_score(&ConfidenceInput {
            similarity,
            successes,
            failures,
            drift,
            age_days,
            feedback,
            task_confirmation,
        });
        prop_assert!((0.0..=1.0).contains(&score));
        prop_assert!(!score.is_nan());
    }
}
