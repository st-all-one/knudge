//! Helpers compartilhados dos testes de integração de "uso real".
#![allow(
    dead_code,
    reason = "cada arquivo de teste usa um subconjunto destes helpers"
)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "o módulo `common` é privado ao crate de teste; `pub(crate)` é o alcance correto"
)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Alias de resultado dos testes (sem `unwrap`/`expect` — D92).
pub(crate) type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Cria um diretório temporário isolado (projeto + `XDG_CONFIG_HOME`).
pub(crate) fn temp_project() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let serial = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("kd-real-{}-{serial}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&dir);
    let _ignored = std::fs::create_dir_all(&dir);
    dir
}

/// Executa `kd` num diretório isolado.
pub(crate) fn run_in(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir)
        .env("KNUDGE_NO_IDLE", "1")
        .output()
}

/// Executa `kd` num diretório isolado alimentando o `stdin`.
pub(crate) fn run_stdin(dir: &Path, args: &[&str], input: &str) -> std::io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir)
        .env("KNUDGE_NO_IDLE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }
    child.wait_with_output()
}

/// Extrai o `data` do envelope `--json`.
pub(crate) fn data(out: &Output) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let data = value.get("data").cloned().ok_or_else(|| {
        format!(
            "envelope sem data: {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })?;
    Ok(data)
}

/// `stdout` como string.
pub(crate) fn stdout(out: &Output) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(out.stdout.clone())?)
}

/// `stderr` como string.
pub(crate) fn stderr(out: &Output) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(out.stderr.clone())?)
}

/// `init --no-prompt` no diretório.
pub(crate) fn init(dir: &Path) -> TestResult {
    let out = run_in(dir, &["init", "--no-prompt"])?;
    assert!(out.status.success(), "init falhou: {}", stderr(&out)?);
    Ok(())
}

/// Roda e exige sucesso, devolvendo o `stdout`.
pub(crate) fn ok(dir: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(dir, args)?;
    assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
    stdout(&out)
}

/// Roda `--json` e exige sucesso, devolvendo o `data` do envelope.
pub(crate) fn ok_json(
    dir: &Path,
    args: &[&str],
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut full = vec!["--json"];
    full.extend_from_slice(args);
    let out = run_in(dir, &full)?;
    assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
    data(&out)
}

/// Roda e exige um exit code específico.
pub(crate) fn expect_code(dir: &Path, args: &[&str], code: i32) -> TestResult {
    let out = run_in(dir, args)?;
    assert_eq!(
        out.status.code(),
        Some(code),
        "{args:?} devia ser exit {code}: {}",
        stderr(&out)?
    );
    Ok(())
}

/// Escreve uma nota e devolve o id.
pub(crate) fn write_note(
    dir: &Path,
    summary: &str,
    note_type: &str,
    extra: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut args = vec!["--json", "write", "--summary", summary, "--type", note_type];
    args.extend_from_slice(extra);
    let out = run_in(dir, &args)?;
    assert!(out.status.success(), "write falhou: {}", stderr(&out)?);
    let id = data(&out)?
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or("write sem id")?
        .to_string();
    Ok(id)
}

/// Cria uma tarefa e devolve o id.
pub(crate) fn task_new(
    dir: &Path,
    summary: &str,
    extra: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut args = vec!["--json", "task", "new", "--summary", summary];
    args.extend_from_slice(extra);
    let out = run_in(dir, &args)?;
    assert!(out.status.success(), "task new falhou: {}", stderr(&out)?);
    let items = data(&out)?;
    // `task new --summary` devolve `data.id`; `--params`/`--batch` devolvem `data.items[]`.
    let id = items
        .get("id")
        .and_then(|value| value.as_str())
        .or_else(|| {
            items
                .get("items")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("id"))
                .and_then(|value| value.as_str())
        })
        .ok_or("task new sem id")?
        .to_string();
    Ok(id)
}
