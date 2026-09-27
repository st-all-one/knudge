//! Testes da confiança bayesiana Beta-Bernoulli (D189, E19-T01).

use proptest::prelude::{ProptestConfig, prop_assert, proptest};

use crate::lifecycle::beta::{lower_bound, posterior_mean};

#[test]
fn no_evidence_is_neutral() {
    assert!(posterior_mean(0.0, 0.0).abs() < 1e-12);
    assert!(lower_bound(0.0, 0.0).abs() < 1e-12);
}

#[test]
fn a_single_success_is_conservative_and_twenty_are_strong() {
    let one = lower_bound(1.0, 0.0);
    let twenty = lower_bound(20.0, 0.0);
    assert!((0.19..0.22).contains(&one), "1 sucesso: {one}");
    assert!(twenty > 0.83, "20 sucessos: {twenty}");
    assert!(twenty > one);
}

#[test]
fn failures_pull_the_bound_down() {
    let clean = lower_bound(10.0, 0.0);
    let mixed = lower_bound(10.0, 10.0);
    assert!(mixed < clean);
}

#[test]
fn partial_counts_half_on_each_side() {
    // 0,5 sucesso e 0,5 falha equivalem a um ensaio dividido; a média fica em 0,5.
    assert!((posterior_mean(0.5, 0.5) - 0.5).abs() < 1e-9);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn bounds_are_sane(successes in 0.0_f64..1e6, failures in 0.0_f64..1e6) {
        let mean = posterior_mean(successes, failures);
        let low = lower_bound(successes, failures);
        prop_assert!((0.0..=1.0).contains(&mean));
        prop_assert!((0.0..=1.0).contains(&low));
        prop_assert!(!mean.is_nan() && !low.is_nan());
        // O limite inferior nunca supera a média posterior.
        prop_assert!(low <= mean + 1e-9);
    }

    #[test]
    fn lower_bound_is_monotone(
        successes in 0.0_f64..1e4,
        failures in 0.0_f64..1e4,
        delta in 1e-3_f64..1e3,
    ) {
        let base = lower_bound(successes, failures);
        let more = lower_bound(successes + delta, failures);
        let less = lower_bound(successes, failures + delta);
        prop_assert!(more >= base - 1e-12);
        prop_assert!(less <= base + 1e-12);
    }
}
