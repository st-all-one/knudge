//! Testes do catálogo de validators e resolução de `checks` (E09-T01).

use crate::Result;

use super::super::validator::{CheckSource, Severity, ValidatorCatalog, resolve_checks};

const CATALOG: &str = r#"
globals = ["phpstan"]

[phpstan]
cmd = "phpstan analyse --level=9"
severity = "error"

[phpunit]
cmd = "phpunit"
scope = ["V2/**"]

[no-storage]
cmd = "grep -rn storage V2/"
scope = ["V2/**"]
severity = "warn"
timeout = 300
"#;

#[test]
fn resolves_three_sources() -> Result<()> {
    let catalog = ValidatorCatalog::parse(CATALOG)?;
    let anchors = vec!["V2/Modules/**".to_string()];
    let resolved = resolve_checks(&catalog, &["phpunit".to_string()], &anchors);

    let names: Vec<(&str, CheckSource)> = resolved
        .checks
        .iter()
        .map(|check| (check.name.as_str(), check.source))
        .collect();
    assert_eq!(
        names,
        vec![
            ("no-storage", CheckSource::Anchor),
            ("phpstan", CheckSource::Global),
            ("phpunit", CheckSource::Explicit),
        ]
    );
    assert!(resolved.missing.is_empty());
    Ok(())
}

#[test]
fn severity_and_timeout_come_from_catalog() -> Result<()> {
    let catalog = ValidatorCatalog::parse(CATALOG)?;
    let no_storage = catalog
        .get("no-storage")
        .ok_or_else(|| crate::Error::internal("validator ausente"))?;
    assert_eq!(no_storage.severity, Severity::Warn);
    assert_eq!(no_storage.timeout_ms, 300);
    assert_eq!(catalog.globals(), &["phpstan".to_string()]);
    Ok(())
}

#[test]
fn missing_validator_is_reported_not_fatal() -> Result<()> {
    let catalog = ValidatorCatalog::parse(CATALOG)?;
    let resolved = resolve_checks(&catalog, &["nao-existe".to_string()], &[]);
    assert_eq!(resolved.missing, vec!["nao-existe".to_string()]);
    assert!(
        resolved
            .checks
            .iter()
            .all(|check| check.name != "nao-existe")
    );
    Ok(())
}

#[test]
fn unknown_global_is_rejected() {
    let bad = "globals = [\"fantasma\"]\n";
    assert!(ValidatorCatalog::parse(bad).is_err());
}

#[test]
fn validator_without_cmd_is_rejected() {
    let bad = "[broken]\nseverity = \"warn\"\n";
    assert!(ValidatorCatalog::parse(bad).is_err());
}
