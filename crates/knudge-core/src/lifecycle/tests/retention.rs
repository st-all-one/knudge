//! Testes da retenção por curva de esquecimento (D190, E19-T02).

use proptest::prelude::{ProptestConfig, prop_assert, proptest};

use crate::Result;
use crate::lifecycle::ShelfLife;
use crate::lifecycle::retention::{DEFAULT_REVIEW_THRESHOLD, retention, stability_days};
use crate::lifecycle::shelf_life::{DAY_MS, is_expired};
use crate::schema::Classification;

use super::{NOW, classified, with_outcomes_at};

/// `n` dias em milissegundos (saturante).
fn days(n: i64) -> i64 {
    DAY_MS.saturating_mul(n)
}

#[test]
fn retention_crosses_the_threshold_at_the_prazo() {
    let at_ttl = retention(30.0, 30.0, DEFAULT_REVIEW_THRESHOLD);
    assert!((at_ttl - DEFAULT_REVIEW_THRESHOLD).abs() < 1e-9, "{at_ttl}");
    assert!((retention(0.0, 30.0, DEFAULT_REVIEW_THRESHOLD) - 1.0).abs() < 1e-9);
    assert!(retention(60.0, 30.0, DEFAULT_REVIEW_THRESHOLD) < DEFAULT_REVIEW_THRESHOLD);
}

#[test]
fn stability_is_the_prazo_over_ln() {
    let stability = stability_days(30.0, DEFAULT_REVIEW_THRESHOLD);
    assert!((stability - 30.0 / 2.0_f64.ln()).abs() < 1e-9);
}

#[test]
fn successes_extend_the_prazo() {
    let policy = ShelfLife::default();
    assert_eq!(
        policy.effective_ttl_days(Classification::Observational, 0),
        Some(30)
    );
    // +50 % por revisão (default).
    assert_eq!(
        policy.effective_ttl_days(Classification::Observational, 1),
        Some(45)
    );
    assert_eq!(
        policy.effective_ttl_days(Classification::Observational, 2),
        Some(60)
    );
    // `foundational` nunca expira, mesmo com revisões.
    assert_eq!(
        policy.effective_ttl_days(Classification::Foundational, 5),
        None
    );
}

#[test]
fn a_successful_review_resets_the_clock_and_extends() -> Result<()> {
    let policy = ShelfLife::default();
    let plain = classified("sem revisao", Classification::Observational, NOW)?;
    let reviewed = with_outcomes_at(
        classified("com revisao", Classification::Observational, NOW)?,
        &[("success", NOW)],
    )?;
    let later = NOW.saturating_add(days(40));
    assert!(is_expired(&plain, later, &policy)?);
    assert!(!is_expired(&reviewed, later, &policy)?);
    Ok(())
}

#[test]
fn a_recent_review_rescues_an_old_note() -> Result<()> {
    let policy = ShelfLife::default();
    let old = classified(
        "antiga",
        Classification::Observational,
        NOW.saturating_sub(days(100)),
    )?;
    assert!(is_expired(&old, NOW, &policy)?);
    let rescued = with_outcomes_at(old, &[("success", NOW.saturating_sub(days(1)))])?;
    assert!(!is_expired(&rescued, NOW, &policy)?);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn retention_decreases_in_time(elapsed in 0.0_f64..1e4, ttl in 1.0_f64..1e4) {
        let now = retention(elapsed, ttl, DEFAULT_REVIEW_THRESHOLD);
        let later = retention(elapsed + 1.0, ttl, DEFAULT_REVIEW_THRESHOLD);
        prop_assert!(later <= now + 1e-12);
        prop_assert!((0.0..=1.0).contains(&now));
        prop_assert!(!now.is_nan());
    }

    #[test]
    fn retention_grows_with_the_prazo(
        elapsed in 0.0_f64..1e4,
        ttl in 1.0_f64..1e4,
        delta in 0.0_f64..1e4,
    ) {
        let low = retention(elapsed, ttl, DEFAULT_REVIEW_THRESHOLD);
        let high = retention(elapsed, ttl + delta, DEFAULT_REVIEW_THRESHOLD);
        prop_assert!(high >= low - 1e-12);
    }
}
