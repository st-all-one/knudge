//! Testes de integração do binário `kd` (E01-T04/E12): contrato de saída, `--json`, EPIPE e
//! fluxos com estado num projeto temporário.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Alias de resultado dos testes (sem `unwrap`/`expect`, proibidos por D92).
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Executa `kd` com os argumentos dados e captura a saída.
fn run(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kd")).args(args).output()
}

/// Executa `kd` num diretório isolado (projeto temporário, sem config global).
fn run_in(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir)
        .output()
}

/// Cria um diretório temporário único para um teste.
fn temp_project() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let serial = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("kd-it-{}-{serial}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&dir);
    let _ignored = std::fs::create_dir_all(&dir);
    dir
}

/// Interpreta o stdout como uma linha JSON.
fn json(output: &Output) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let text = String::from_utf8(output.stdout.clone())?;
    Ok(serde_json::from_str(text.trim())?)
}

#[test]
fn help_exits_zero_and_mentions_usage() -> TestResult {
    let out = run(&["--help"])?;
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("Usage: kd"), "help inesperado: {text}");
    Ok(())
}

#[test]
fn no_args_equals_prime() -> TestResult {
    let bare = run(&[])?.stdout;
    let prime = run(&["prime"])?.stdout;
    assert_eq!(bare, prime, "`kd` deve ser idêntico a `kd prime`");
    Ok(())
}

#[test]
fn prime_is_byte_identical_across_runs() -> TestResult {
    let first = run(&["prime"])?.stdout;
    let second = run(&["prime"])?.stdout;
    assert_eq!(first, second, "`prime` deve ser byte-idêntico");
    Ok(())
}

#[test]
fn json_envelope_for_prime_is_valid() -> TestResult {
    let out = run(&["--json"])?;
    assert!(out.status.success());
    let value = json(&out)?;
    assert_eq!(value.get("success"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        value.get("command"),
        Some(&serde_json::Value::String("prime".to_string()))
    );
    Ok(())
}

#[test]
fn self_version_works_in_both_modes() -> TestResult {
    let text = run(&["self", "version"])?;
    assert!(text.status.success());
    assert!(String::from_utf8(text.stdout)?.starts_with("kd "));

    let out = run(&["--json", "self", "version"])?;
    assert!(out.status.success());
    let value = json(&out)?;
    assert_eq!(value.get("success"), Some(&serde_json::Value::Bool(true)));
    Ok(())
}

#[test]
fn unknown_command_exits_two() -> TestResult {
    let out = run(&["definitely-not-a-command"])?;
    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
    Ok(())
}

#[test]
fn broken_pipe_exits_zero() -> TestResult {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(["self", "version"])
        .stdout(Stdio::piped())
        .spawn()?;
    drop(child.stdout.take());
    let status = child.wait()?;
    assert!(status.success(), "EPIPE deve terminar com exit 0");
    Ok(())
}

#[test]
fn init_write_ask_roundtrip() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let write = run_in(
        &dir,
        &["--json", "write", "o cache usa body_hash", "--type", "fact"],
    )?;
    assert!(write.status.success(), "write falhou: {:?}", write.stderr);
    let envelope = json(&write)?;
    let id = envelope
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("write sem id")?
        .to_string();
    assert!(id.starts_with("fact_"), "id inesperado: {id}");

    let ask = run_in(&dir, &["ask", "cache"])?;
    assert!(ask.status.success(), "ask falhou: {:?}", ask.stderr);
    let text = String::from_utf8(ask.stdout)?;
    assert!(text.contains(&id), "ask não recuperou {id}: {text}");
    Ok(())
}

#[test]
fn forgotten_note_is_hidden_from_default_ask() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let write = run_in(
        &dir,
        &[
            "--json",
            "write",
            "segredo temporário do cache",
            "--type",
            "fact",
        ],
    )?;
    assert!(write.status.success(), "write falhou: {:?}", write.stderr);
    let envelope = json(&write)?;
    let id = envelope
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("write sem id")?
        .to_string();

    let forget = run_in(&dir, &["forget", id.as_str()])?;
    assert!(
        forget.status.success(),
        "forget falhou: {:?}",
        forget.stderr
    );

    let hidden = run_in(&dir, &["ask", "cache"])?;
    assert!(hidden.status.success(), "ask falhou: {:?}", hidden.stderr);
    let text = String::from_utf8(hidden.stdout)?;
    assert!(
        !text.contains(&id),
        "nota esquecida apareceu no ask: {text}"
    );

    let shown = run_in(&dir, &["ask", "cache", "--status", "forgotten"])?;
    assert!(shown.status.success(), "ask falhou: {:?}", shown.stderr);
    let text = String::from_utf8(shown.stdout)?;
    assert!(
        text.contains(&id),
        "ask --status forgotten não achou {id}: {text}"
    );
    Ok(())
}

#[test]
fn write_rejects_task_type() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["--json", "write", "algo", "--type", "task"])?;
    assert_eq!(out.status.code(), Some(2));
    let value = json(&out)?;
    assert_eq!(value.get("success"), Some(&serde_json::Value::Bool(false)));
    let code = value
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_str());
    assert_eq!(code, Some("invalid_input"));
    Ok(())
}

#[test]
fn task_new_requires_scope() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["--json", "task", "new", "passo", "--scope", "task"])?;
    assert!(out.status.success(), "task falhou: {:?}", out.stderr);
    let value = json(&out)?;
    let id = value
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("task sem id")?;
    assert!(id.starts_with("task_"), "id inesperado: {id}");
    Ok(())
}

#[test]
fn config_set_get_roundtrip() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let set = run_in(&dir, &["config", "set", "recall.default_limit", "7"])?;
    assert!(set.status.success(), "set falhou: {:?}", set.stderr);
    let get = run_in(&dir, &["--json", "config", "get", "recall.default_limit"])?;
    assert!(get.status.success());
    let value = json(&get)?;
    assert_eq!(
        value
            .get("data")
            .and_then(|data| data.get("value"))
            .and_then(|value| value.as_str()),
        Some("7")
    );
    Ok(())
}

#[test]
fn missing_key_reports_not_found() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["--json", "config", "get", "nao.existe"])?;
    assert_eq!(out.status.code(), Some(3));
    Ok(())
}
