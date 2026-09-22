//! Golden/snapshot do contrato de bytes do binário `kd` (E13-T01).
//!
//! Qualquer mudança de bytes/mensagem reprova aqui até o golden ser atualizado de propósito.
//! Caminhos voláteis (diretório temporário e nome do projeto) são normalizados para
//! `<ROOT>`/`<NAME>` antes da comparação.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Alias de resultado (sem `unwrap`/`expect`, proibidos por D92).
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn temp_project() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let serial = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("kd-golden-{}-{serial}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&dir);
    let _ignored = std::fs::create_dir_all(&dir);
    dir
}

fn run_in(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir)
        .output()
}

/// Normaliza caminhos voláteis para permitir o golden.
fn normalize(bytes: &[u8], dir: &Path) -> String {
    let text = String::from_utf8_lossy(bytes);
    let root = dir.to_string_lossy();
    let name = dir
        .file_name()
        .map_or_else(String::new, |value| value.to_string_lossy().into_owned());
    let text = text.replace(root.as_ref(), "<ROOT>");
    if name.is_empty() {
        text
    } else {
        text.replace(&name, "<NAME>")
    }
}

fn assert_golden(actual: &[u8], expected: &str, dir: &Path) {
    let normalized = normalize(actual, dir);
    assert_eq!(
        normalized, expected,
        "golden divergiu; atualize `tests/golden/` se a mudança for intencional"
    );
}

#[test]
fn prime_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["prime"])?;
    assert!(out.status.success());
    assert_golden(&out.stdout, include_str!("golden/prime.txt"), &dir);
    Ok(())
}

#[test]
fn json_prime_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["--json"])?;
    assert!(out.status.success());
    assert_golden(&out.stdout, include_str!("golden/json_prime.json"), &dir);
    Ok(())
}

#[test]
fn json_version_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["--json", "self", "version"])?;
    assert!(out.status.success());
    assert_golden(&out.stdout, include_str!("golden/json_version.json"), &dir);
    Ok(())
}

#[test]
fn json_error_envelope_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["--json", "ask", "--id", "fact_zzzzzzzz"])?;
    assert_eq!(out.status.code(), Some(5));
    assert_golden(&out.stdout, include_str!("golden/json_error_io.json"), &dir);
    Ok(())
}

#[test]
fn text_error_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["ask", "--id", "fact_zzzzzzzz"])?;
    assert_eq!(out.status.code(), Some(5));
    assert_golden(&out.stderr, include_str!("golden/error_io.txt"), &dir);
    Ok(())
}

#[test]
fn init_message_matches_golden() -> TestResult {
    let dir = temp_project();
    let out = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(out.status.success());
    assert_golden(&out.stdout, include_str!("golden/init.txt"), &dir);
    Ok(())
}

#[test]
fn epipe_is_exit_zero() -> TestResult {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(["prime"])
        .stdout(Stdio::piped())
        .spawn()?;
    drop(child.stdout.take());
    let status = child.wait()?;
    assert!(status.success(), "EPIPE deve terminar com exit 0");
    Ok(())
}
