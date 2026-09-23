//! Templates de plano (D105).

use crate::Result;
use crate::task::template::TemplateCatalog;

#[test]
fn builtins_include_feature_bug_refactor() {
    let catalog = TemplateCatalog::builtin();
    assert_eq!(catalog.names(), vec!["bug", "feature", "refactor"]);
    let Some(feature) = catalog.get("feature") else {
        return;
    };
    assert_eq!(feature.min_steps, 2);
    assert!(feature.required.contains(&"acceptance".to_string()));
}

#[test]
fn project_overrides_builtin_by_name() -> Result<()> {
    let text = "[templates.feature]\nsections = [\"a\"]\nrequired = [\"a\"]\nmin_steps = 1\n";
    let catalog = TemplateCatalog::parse(text)?;
    let Some(feature) = catalog.get("feature") else {
        return Ok(());
    };
    assert_eq!(feature.min_steps, 1);
    assert_eq!(feature.sections, vec!["a"]);
    Ok(())
}

#[test]
fn required_section_must_exist() {
    let text = "[templates.x]\nsections = [\"a\"]\nrequired = [\"b\"]\n";
    assert!(TemplateCatalog::parse(text).is_err());
}
