//! `kd init --git-excluded` / `--git-tracked`: modo de persistência por invocação (D34).
//!
//! As flags sobrepõem `knowledge.persist_in_project` no config do **projeto** — inclusive com
//! `--force` — e reaplicam `.git/info/exclude`/`.gitattributes` de forma idempotente.

mod common;

use std::path::Path;
use std::process::Command;

use common::{TestResult, expect_code, ok_json, run_in, stderr, temp_project};

/// Inicializa um repositório git mínimo (determinístico: sem config global do sistema).
fn git_init(dir: &Path) -> TestResult {
    let out = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .output()?;
    assert!(
        out.status.success(),
        "git init falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}

/// Lê um arquivo do projeto temporário.
fn read(dir: &Path, rel: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(std::fs::read_to_string(dir.join(rel))?)
}

#[test]
fn git_excluded_excludes_whole_dir_and_writes_config() -> TestResult {
    let dir = temp_project();
    git_init(&dir)?;

    let data = ok_json(&dir, &["init", "--no-prompt", "--git-excluded"])?;
    assert_eq!(
        data.get("exclude_changed")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let exclude = read(&dir, ".git/info/exclude")?;
    assert!(
        exclude.lines().any(|line| line.trim() == "/.knudge/"),
        "exclude: {exclude}"
    );
    let config = read(&dir, ".knudge/config.toml")?;
    assert!(
        config.contains("persist_in_project = false"),
        "config: {config}"
    );
    Ok(())
}

#[test]
fn git_tracked_reverts_to_versioned_exclusion() -> TestResult {
    let dir = temp_project();
    git_init(&dir)?;
    let _ignored = ok_json(&dir, &["init", "--no-prompt", "--git-excluded"])?;

    let data = ok_json(&dir, &["init", "--no-prompt", "--git-tracked"])?;
    assert_eq!(
        data.get("exclude_changed")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let exclude = read(&dir, ".git/info/exclude")?;
    assert!(
        !exclude.lines().any(|line| line.trim() == "/.knudge/"),
        "exclude: {exclude}"
    );
    assert!(exclude.contains("/.knudge/.idx/"));
    let config = read(&dir, ".knudge/config.toml")?;
    assert!(
        config.contains("persist_in_project = true"),
        "config: {config}"
    );
    Ok(())
}

#[test]
fn git_excluded_wins_over_force() -> TestResult {
    let dir = temp_project();
    git_init(&dir)?;
    let _ignored = ok_json(&dir, &["init", "--no-prompt"])?;

    let set = run_in(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "knowledge.persist_in_project",
            "--value",
            "true",
        ],
    )?;
    assert!(set.status.success(), "config set: {}", stderr(&set)?);

    let data = ok_json(&dir, &["init", "--no-prompt", "--force", "--git-excluded"])?;
    assert_eq!(
        data.get("exclude_changed")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let config = read(&dir, ".knudge/config.toml")?;
    assert!(
        config.contains("persist_in_project = false"),
        "a flag deve vencer o clone com --force: {config}"
    );
    let exclude = read(&dir, ".git/info/exclude")?;
    assert!(exclude.lines().any(|line| line.trim() == "/.knudge/"));
    Ok(())
}

#[test]
fn conflicting_persistence_flags_exit_2() -> TestResult {
    let dir = temp_project();
    expect_code(&dir, &["init", "--git-excluded", "--git-tracked"], 2)
}
