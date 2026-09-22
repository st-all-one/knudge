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

/// Escreve uma nota ancorada num projeto e devolve o id.
fn write_anchored(
    dir: &Path,
    statement: &str,
    anchor: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "write",
            statement,
            "--type",
            "fact",
            "--anchors",
            anchor,
        ],
    )?;
    assert!(out.status.success(), "write falhou: {:?}", out.stderr);
    let id = json(&out)?
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("write sem id")?
        .to_string();
    Ok(id)
}

/// Cria uma tarefa via CLI e devolve o id.
fn task_new(dir: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut full = vec!["--json"];
    full.extend_from_slice(args);
    let out = run_in(dir, &full)?;
    assert!(out.status.success(), "task new falhou: {:?}", out.stderr);
    let id = json(&out)?
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("task sem id")?
        .to_string();
    Ok(id)
}

#[test]
fn ask_anchor_finds_note_without_query() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let id = write_anchored(&dir, "o cache usa body_hash", "src/cache.rs")?;

    let ask = run_in(&dir, &["ask", "--anchor", "src/cache.rs"])?;
    assert!(ask.status.success(), "ask falhou: {:?}", ask.stderr);
    let text = String::from_utf8(ask.stdout)?;
    assert!(
        text.contains(&id),
        "ask --anchor não recuperou {id}: {text}"
    );
    assert!(
        text.contains("file_match"),
        "why esperado file_match: {text}"
    );
    Ok(())
}

#[test]
fn ask_anchor_accepts_comma_separated_and_repeated() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let first_id = write_anchored(&dir, "o cache usa body_hash", "src/cache.rs")?;
    let second_id = write_anchored(&dir, "a fila usa backoff", "src/queue.rs")?;

    let comma = run_in(&dir, &["ask", "--anchor", "src/cache.rs,src/queue.rs"])?;
    assert!(comma.status.success(), "ask falhou: {:?}", comma.stderr);
    let text = String::from_utf8(comma.stdout)?;
    assert!(
        text.contains(&first_id),
        "comma não achou {first_id}: {text}"
    );
    assert!(
        text.contains(&second_id),
        "comma não achou {second_id}: {text}"
    );

    let repeated = run_in(
        &dir,
        &[
            "ask",
            "--anchor",
            "src/cache.rs",
            "--anchor",
            "src/queue.rs",
        ],
    )?;
    assert!(
        repeated.status.success(),
        "ask falhou: {:?}",
        repeated.stderr
    );
    let text = String::from_utf8(repeated.stdout)?;
    assert!(
        text.contains(&first_id),
        "repeat não achou {first_id}: {text}"
    );
    assert!(
        text.contains(&second_id),
        "repeat não achou {second_id}: {text}"
    );
    Ok(())
}

#[test]
fn write_outcome_on_note_returns_outcome_action() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let id = write_anchored(&dir, "o cache usa body_hash", "src/cache.rs")?;

    let out = run_in(
        &dir,
        &[
            "--json",
            "write",
            "--outcome",
            "success",
            id.as_str(),
            "--note",
            "confirmado",
        ],
    )?;
    assert!(out.status.success(), "outcome falhou: {:?}", out.stderr);
    let envelope = json(&out)?;
    let data = envelope.get("data").ok_or("sem data")?;
    assert_eq!(data.get("action").and_then(|v| v.as_str()), Some("outcome"));
    assert_eq!(
        data.get("outcome").and_then(|v| v.as_str()),
        Some("success")
    );
    assert_eq!(
        data.get("revision").and_then(serde_json::Value::as_i64),
        Some(2)
    );
    Ok(())
}

#[test]
fn task_list_ready_blocked_and_explain() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(&dir, &["task", "new", "Programa", "--scope", "epic"])?;
    let issue = task_new(
        &dir,
        &[
            "task", "new", "Story", "--scope", "issue", "--parent", &epic,
        ],
    )?;
    let ready = task_new(
        &dir,
        &[
            "task", "new", "Pronta", "--scope", "task", "--parent", &issue,
        ],
    )?;
    let blocked = task_new(
        &dir,
        &[
            "task",
            "new",
            "Bloqueada",
            "--scope",
            "task",
            "--parent",
            &issue,
            "--depends-on",
            &ready,
        ],
    )?;

    let out = run_in(&dir, &["task", "list", "--ready"])?;
    assert!(out.status.success(), "list falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(ready.as_str()),
        "ready não listou {ready}: {text}"
    );
    assert!(
        !text.contains(blocked.as_str()),
        "ready listou bloqueada {blocked}: {text}"
    );

    let out = run_in(&dir, &["task", "list", "--blocked", "--explain"])?;
    assert!(out.status.success(), "list falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(blocked.as_str()),
        "blocked não listou {blocked}: {text}"
    );
    assert!(
        text.contains(&format!("blocked_by={ready}")),
        "explain sem motivo: {text}"
    );

    let out = run_in(&dir, &["task", "list", "--explain"])?;
    assert_eq!(out.status.code(), Some(2));
    Ok(())
}

#[test]
fn task_graph_program_renders_subtree() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let root = task_new(
        &dir,
        &[
            "task",
            "new",
            "Programa",
            "--scope",
            "epic",
            "--anchors",
            "plan/foo.md",
            "--source",
            "plan/foo.md",
        ],
    )?;
    let story = task_new(
        &dir,
        &[
            "task", "new", "Story", "--scope", "issue", "--parent", &root,
        ],
    )?;

    let out = run_in(&dir, &["task", "graph", "--program", "plan/foo.md"])?;
    assert!(out.status.success(), "graph falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains("plan/foo.md"),
        "sem o path do programa: {text}"
    );
    assert!(text.contains(&root), "sem o Épico-raiz {root}: {text}");
    assert!(text.contains(&story), "sem a Story {story}: {text}");

    let out = run_in(&dir, &["task", "graph", "--program", "plan/nao-existe.md"])?;
    assert_eq!(out.status.code(), Some(3));
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
fn ask_with_body_and_brief_contract() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let write = run_in(
        &dir,
        &[
            "--json",
            "write",
            "formato TOON é linha a linha",
            "--type",
            "def",
            "--body",
            "detalhe do corpo",
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

    let brief = run_in(&dir, &["ask", "TOON", "--brief"])?;
    assert!(
        brief.status.success(),
        "ask --brief falhou: {:?}",
        brief.stderr
    );
    let text = String::from_utf8(brief.stdout)?;
    assert!(text.contains(&id), "--brief perdeu o hit: {text}");
    assert!(
        !text.contains("detalhe do corpo"),
        "--brief vazou o corpo: {text}"
    );

    let with_body = run_in(&dir, &["ask", "TOON", "--with-body"])?;
    assert!(
        with_body.status.success(),
        "ask --with-body falhou: {:?}",
        with_body.stderr
    );
    let text = String::from_utf8(with_body.stdout)?;
    assert!(
        text.contains("detalhe do corpo"),
        "--with-body não trouxe o corpo: {text}"
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
