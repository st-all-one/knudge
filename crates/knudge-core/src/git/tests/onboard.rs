//! Testes de `onboard()` idempotente (E04-T04).

use std::path::Path;

use super::*;
use crate::git::{OnboardOptions, onboard};
use crate::ports::Fs;

#[test]
fn creates_layout_and_default_config_outside_repo() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(false, None, None, None);
    let report = onboard(&fs, &git, &env_at("/work/proj"), OnboardOptions::default())?;
    assert!(!report.in_repo);
    assert!(report.config_written);
    assert!(fs.is_dir(Path::new("/work/proj/.knudge/notas")));
    assert!(fs.is_dir(Path::new("/work/proj/.knudge/eventos")));
    assert!(fs.exists(Path::new("/work/proj/.knudge/config.toml")));
    assert!(fs.exists(Path::new("/work/proj/AGENTS.md")));
    assert!(!fs.exists(Path::new("/work/proj/.gitattributes")));
    Ok(())
}

#[test]
fn clones_global_config_literally() -> Result<()> {
    let fs = MemFs::new();
    let global_dir = Path::new("/home/u/.config/local/knudge");
    fs.create_dir_all(global_dir)?;
    let global = b"# curado\n[behavior]\nstrict = false\n";
    fs.write_atomic(&global_dir.join("config.toml"), global)?;

    let mut env = env_at("/work/proj");
    env.vars.insert("HOME".into(), "/home/u".into());
    let git = git_with(false, None, None, None);
    onboard(&fs, &git, &env, OnboardOptions::default())?;

    let cloned = fs.read(Path::new("/work/proj/.knudge/config.toml"))?;
    assert_eq!(cloned, global, "clone deve ser literal (D62)");
    Ok(())
}

#[test]
fn does_not_overwrite_project_config_without_force() -> Result<()> {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/work/proj/.knudge"))?;
    let project_config = Path::new("/work/proj/.knudge/config.toml");
    let custom = b"[behavior]\nstrict = true\n";
    fs.write_atomic(project_config, custom)?;

    let git = git_with(false, None, None, None);
    let options = OnboardOptions::default();
    let report = onboard(&fs, &git, &env_at("/work/proj"), options)?;
    assert!(!report.config_written);
    assert_eq!(fs.read(project_config)?, custom);

    let forced = OnboardOptions { force: true };
    let report = onboard(&fs, &git, &env_at("/work/proj"), forced)?;
    assert!(report.config_written);
    assert_ne!(fs.read(project_config)?, custom);
    Ok(())
}

#[test]
fn applies_git_files_when_in_repo_and_is_idempotent() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    let env = env_at("/repo");
    let options = OnboardOptions::default();

    let first = onboard(&fs, &git, &env, options)?;
    assert!(first.exclude_changed);
    assert!(first.attributes_changed);
    assert!(first.agents_changed);
    assert!(first.skill_changed);
    assert!(fs.exists(Path::new("/repo/.agents/skill/kd/SKILL.md")));
    assert!(fs.exists(Path::new("/repo/.git/info/exclude")));
    let attributes =
        String::from_utf8(fs.read(Path::new("/repo/.gitattributes"))?).unwrap_or_default();
    assert!(attributes.contains("merge=union"));

    let second = onboard(&fs, &git, &env, options)?;
    assert!(!second.config_written);
    assert!(!second.exclude_changed);
    assert!(!second.attributes_changed);
    assert!(!second.agents_changed);
    assert!(!second.skill_changed);
    Ok(())
}

#[test]
fn local_only_config_excludes_whole_dir_and_reverts() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    let env = env_at("/repo");
    let options = OnboardOptions::default();

    // Primeiro onboard com o default (versionado): bloco de union presente.
    onboard(&fs, &git, &env, options)?;
    let attributes_path = Path::new("/repo/.gitattributes");
    let versioned = String::from_utf8(fs.read(attributes_path)?).unwrap_or_default();
    assert!(versioned.contains("merge=union"));

    // Usuário muda para local-only e reexecuta.
    let config_path = Path::new("/repo/.knudge/config.toml");
    fs.write_atomic(config_path, b"[knowledge]\npersist_in_project = false\n")?;
    onboard(&fs, &git, &env, options)?;

    let exclude =
        String::from_utf8(fs.read(Path::new("/repo/.git/info/exclude"))?).unwrap_or_default();
    assert!(exclude.lines().any(|line| line.trim() == "/.knudge/"));
    assert!(!exclude.contains("/.knudge/.idx/"));
    let attributes = String::from_utf8(fs.read(attributes_path)?).unwrap_or_default();
    assert!(
        !attributes.contains("merge=union"),
        "bloco deve ser revertido"
    );
    Ok(())
}
