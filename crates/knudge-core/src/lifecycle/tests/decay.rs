//! Testes do decay de âncoras (E10-T02).

use std::path::Path;

use crate::Result;
use crate::config::Config;
use crate::lifecycle::decay::{
    AnchorValidity, DecayPolicy, compute_anchor_validity, compute_anchor_validity_with,
    should_demote, walk_paths,
};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use proptest::prelude::*;

use super::PROJECT;

#[test]
fn validity_counts_with_injected_oracle() {
    let anchors = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let validity = compute_anchor_validity_with(&anchors, |anchor| anchor != "b");
    assert_eq!(validity.total, 3);
    assert_eq!(validity.valid, 2);
    assert_eq!(validity.broken, 1);
    assert!((validity.fraction() - (2.0 / 3.0)).abs() < 1e-9);
}

#[test]
fn validity_without_anchors_is_full() {
    let validity = compute_anchor_validity_with(&[], |_| false);
    assert_eq!(validity.total, 0);
    assert!((validity.fraction() - 1.0).abs() < 1e-9);
}

#[test]
fn walk_paths_finds_files_and_skips_ignored_dirs() {
    let fs = MemFs::new();
    fs.insert("/p/src/lib.rs", "fn a() {}");
    fs.insert("/p/README.md", "x");
    fs.insert("/p/.git/config", "[core]");
    fs.insert("/p/target/debug/bin", "x");
    let paths = walk_paths(&fs, Path::new(PROJECT));
    assert!(paths.contains(&"src/lib.rs".to_string()));
    assert!(paths.contains(&"README.md".to_string()));
    assert!(!paths.iter().any(|path| path.starts_with(".git/")));
    assert!(!paths.iter().any(|path| path.starts_with("target/")));
}

#[test]
fn compute_anchor_validity_handles_literal_and_glob() {
    let fs = MemFs::new();
    fs.insert("/p/src/lib.rs", "fn a() {}");
    let anchors = vec![
        "src/lib.rs".to_string(),
        "src/*.rs".to_string(),
        "gone.rs".to_string(),
    ];
    let validity = compute_anchor_validity(&fs, Path::new(PROJECT), &anchors);
    assert_eq!(validity.total, 3);
    assert_eq!(validity.valid, 2);
    assert_eq!(validity.broken, 1);
}

#[test]
fn directory_anchor_counts_as_broken() -> Result<()> {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/p/src/dir"))?;
    let anchors = vec!["src/dir".to_string()];
    let validity = compute_anchor_validity(&fs, Path::new(PROJECT), &anchors);
    assert_eq!(validity.total, 1);
    assert_eq!(validity.broken, 1, "diretório não é âncora de conteúdo");
    Ok(())
}

#[test]
fn should_demote_respects_grace_and_threshold() {
    let policy = DecayPolicy {
        threshold: 0.5,
        grace_days: 30,
    };
    let mostly_broken = AnchorValidity {
        total: 4,
        valid: 1,
        broken: 3,
    };
    assert!(should_demote(&mostly_broken, &policy, 30));
    assert!(!should_demote(&mostly_broken, &policy, 29));

    let mostly_valid = AnchorValidity {
        total: 4,
        valid: 3,
        broken: 1,
    };
    assert!(!should_demote(&mostly_valid, &policy, 300));

    let none = AnchorValidity::default();
    assert!(!should_demote(&none, &policy, 300));
}

#[test]
fn policy_reads_config() -> Result<()> {
    let mut config = Config::defaults();
    config.set_str("decay.anchor_threshold", "0.25")?;
    config.set_str("decay.grace_days", "7")?;
    let policy = DecayPolicy::from_config(&config);
    assert!((policy.threshold - 0.25).abs() < 1e-9);
    assert_eq!(policy.grace_days, 7);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// A fração de âncoras válidas vive em `[0, 1]`.
    #[test]
    fn fraction_is_within_unit_range(valid in 0_u32..64, total in 1_u32..64) {
        let valid = valid.min(total);
        let validity = AnchorValidity {
            valid,
            broken: total.saturating_sub(valid),
            total,
        };
        let fraction = validity.fraction();
        prop_assert!((0.0..=1.0).contains(&fraction));
        prop_assert!(!fraction.is_nan());
    }

    /// Uma vez que a demolição é devida, envelhecer mais nunca a desfaz.
    #[test]
    fn demote_is_monotone_in_age(
        valid in 0_u32..64,
        total in 1_u32..64,
        threshold in 0.0_f64..1.0,
        grace in 0_i64..100,
        age in 0_i64..200,
        delta in 0_i64..200,
    ) {
        let valid = valid.min(total);
        let validity = AnchorValidity {
            valid,
            broken: total.saturating_sub(valid),
            total,
        };
        let policy = DecayPolicy { threshold, grace_days: grace };
        if should_demote(&validity, &policy, age) {
            prop_assert!(should_demote(&validity, &policy, age.saturating_add(delta)));
        }
    }
}
