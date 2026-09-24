//! Testes do shelf-life por classificação (E10-T01).

use crate::Result;
use crate::config::Config;
use crate::lifecycle::shelf_life::{
    DAY_MS, ShelfLife, age_days, expiry_for, freshness, is_expired,
};
use crate::schema::{Classification, NoteType};

use super::{NOW, classified, note_created};

/// `n` dias em milissegundos (saturante, para não disparar aritmética não-checada).
fn days(n: i64) -> i64 {
    DAY_MS.saturating_mul(n)
}

#[test]
fn foundational_never_expires() -> Result<()> {
    let old = NOW.saturating_sub(days(3_650));
    let note = classified("fundamento", Classification::Foundational, old)?;
    let policy = ShelfLife::default();
    assert!(!is_expired(&note, NOW, &policy)?);
    assert_eq!(expiry_for(&note, &policy)?, None);
    Ok(())
}

#[test]
fn observational_expires_at_default_prazo() -> Result<()> {
    let policy = ShelfLife::default();
    let expired = classified(
        "observação",
        Classification::Observational,
        NOW.saturating_sub(days(31)),
    )?;
    let fresh = classified(
        "observação",
        Classification::Observational,
        NOW.saturating_sub(days(29)),
    )?;
    assert!(is_expired(&expired, NOW, &policy)?);
    assert!(!is_expired(&fresh, NOW, &policy)?);
    Ok(())
}

#[test]
fn tactical_boundary_is_conservative() -> Result<()> {
    let policy = ShelfLife::default();
    let within = classified(
        "tática",
        Classification::Tactical,
        NOW.saturating_sub(days(364)),
    )?;
    let beyond = classified(
        "tática",
        Classification::Tactical,
        NOW.saturating_sub(days(366)),
    )?;
    assert!(!is_expired(&within, NOW, &policy)?);
    assert!(is_expired(&beyond, NOW, &policy)?);
    Ok(())
}

#[test]
fn config_overrides_defaults() -> Result<()> {
    let mut config = Config::defaults();
    config.set_str("retention.observational_days", "1")?;
    let policy = ShelfLife::from_config(&config);
    let note = classified(
        "observação",
        Classification::Observational,
        NOW.saturating_sub(days(2)),
    )?;
    assert!(is_expired(&note, NOW, &policy)?);
    assert_eq!(policy.ttl_days(Classification::Observational), Some(1));
    Ok(())
}

#[test]
fn age_days_is_clamped_and_floored() -> Result<()> {
    let note = note_created(NoteType::Fact, "nova", "", NOW)?;
    assert_eq!(age_days(&note, NOW), 0);
    assert_eq!(
        age_days(&note, NOW.saturating_add(days(3)).saturating_sub(1)),
        2
    );
    assert_eq!(age_days(&note, NOW.saturating_sub(DAY_MS)), 0);
    Ok(())
}

#[test]
fn freshness_counts_stale_expiring_and_pending() -> Result<()> {
    let policy = ShelfLife::default();
    let expired = classified(
        "velha",
        Classification::Observational,
        NOW.saturating_sub(days(31)),
    )?;
    let expiring = classified(
        "quase",
        Classification::Observational,
        NOW.saturating_sub(days(28)),
    )?;
    let fresh = classified(
        "nova",
        Classification::Observational,
        NOW.saturating_sub(days(1)),
    )?;
    let result = freshness(&[expired, expiring, fresh], NOW, &policy, 4)?;
    assert_eq!(result.stale, 1);
    assert_eq!(result.expiring, 1);
    assert_eq!(result.pending, 4);
    assert_eq!(result.render(), "stale=1 expiring=1 pending=4");
    Ok(())
}
