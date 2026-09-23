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

/// Executa `kd` num diretório isolado com variáveis de ambiente extras.
fn run_env(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> std::io::Result<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kd"));
    command
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output()
}

/// Executa `kd` num diretório isolado com `stdin` alimentado.
fn run_stdin(dir: &Path, args: &[&str], input: &str) -> std::io::Result<Output> {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }
    child.wait_with_output()
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

/// Cria uma nota com expiração explícita e devolve o id.
fn write_expiring(
    dir: &Path,
    statement: &str,
    expires_at: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "write",
            statement,
            "--type",
            "fact",
            "--expires-at",
            expires_at,
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
fn write_batch_jsonl_creates_and_dry_run() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let drafts = concat!(
        "{\"type\":\"fact\",\"statement\":\"batch alfa\"}\n",
        "{\"type\":\"decision\",\"statement\":\"batch beta\",\"anchors\":[\"src/x.ts\"]}\n",
    );
    let out = run_stdin(&dir, &["--json", "write", "--batch", "-"], drafts)?;
    assert!(out.status.success(), "batch falhou: {:?}", out.stderr);
    let envelope = json(&out)?;
    let items = envelope
        .get("data")
        .and_then(|data| data.get("items"))
        .and_then(|items| items.as_array())
        .ok_or("sem items")?;
    assert_eq!(items.len(), 2);
    assert!(
        items.iter().all(|item| {
            item.get("action").and_then(|action| action.as_str()) == Some("created")
        })
    );

    let mixed = "{\"type\":\"fact\",\"statement\":\"efêmera\"}\n{\"type\":\"fact\"}\n";
    let dry = run_stdin(
        &dir,
        &["--json", "write", "--batch", "-", "--dry-run"],
        mixed,
    )?;
    assert!(dry.status.success(), "dry-run falhou: {:?}", dry.stderr);
    let envelope = json(&dry)?;
    let data = envelope.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("dry_run").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    let warnings = envelope
        .get("warnings")
        .and_then(|warnings| warnings.as_array())
        .ok_or("sem warnings")?;
    assert_eq!(warnings.len(), 1);

    let ask = run_in(&dir, &["ask", "efêmera"])?;
    assert!(
        !String::from_utf8(ask.stdout)?.contains("efêmera"),
        "dry-run gravou a nota efêmera"
    );

    let cap = run_in(&dir, &["config", "set", "write.batch_max", "1"])?;
    assert!(cap.status.success(), "config set falhou: {:?}", cap.stderr);
    let over = run_stdin(&dir, &["write", "--batch", "-"], drafts)?;
    assert_eq!(over.status.code(), Some(2), "teto devia dar invalid_input");
    Ok(())
}

#[test]
fn ask_rank_orders_by_confidence() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let _plain = write_anchored(&dir, "o cache usa body_hash", "src/cache.rs")?;
    let confirmed = write_anchored(&dir, "a fila usa backoff", "src/queue.rs")?;
    let out = run_in(&dir, &["write", "--outcome", "success", &confirmed])?;
    assert!(out.status.success(), "outcome falhou: {:?}", out.stderr);

    let ranked = run_in(&dir, &["ask", "--rank"])?;
    assert!(ranked.status.success(), "rank falhou: {:?}", ranked.stderr);
    let text = String::from_utf8(ranked.stdout)?;
    let first = text.lines().next().ok_or("sem linhas")?;
    assert!(
        first.starts_with(&confirmed),
        "primeiro não é o confirmado: {text}"
    );
    assert!(first.contains("stars"), "why esperado stars: {first}");

    let envelope = json(&run_in(&dir, &["--json", "ask", "--rank"])?)?;
    let ranked_json = envelope
        .get("data")
        .and_then(|data| data.get("ranked"))
        .and_then(|ranked| ranked.as_array())
        .ok_or("sem ranked")?;
    assert_eq!(
        ranked_json
            .first()
            .and_then(|row| row.get("id"))
            .and_then(|id| id.as_str()),
        Some(confirmed.as_str())
    );
    Ok(())
}

#[test]
fn maintenance_prune_proposes_forget_for_expired() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let expired = write_expiring(&dir, "nota vencida", "2000-01-01T00:00:00Z")?;
    let fresh = write_anchored(&dir, "nota viva", "src/x.rs")?;

    let out = run_in(&dir, &["--json", "maintenance", "prune"])?;
    assert!(out.status.success(), "prune falhou: {:?}", out.stderr);
    let envelope = json(&out)?;
    let proposals = envelope
        .get("data")
        .and_then(|data| data.get("proposals"))
        .and_then(|proposals| proposals.as_array())
        .ok_or("sem proposals")?;
    let ids: Vec<&str> = proposals
        .iter()
        .filter_map(|proposal| proposal.get("id").and_then(|id| id.as_str()))
        .collect();
    assert!(ids.contains(&expired.as_str()), "vencida ausente: {ids:?}");
    assert!(!ids.contains(&fresh.as_str()), "viva proposta: {ids:?}");
    assert!(proposals.iter().all(|proposal| {
        proposal.get("reason").and_then(|reason| reason.as_str()) == Some("expired")
    }));

    let ask = run_in(&dir, &["ask", "--id", &expired])?;
    assert!(ask.status.success(), "prune gravou: {:?}", ask.stderr);
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
        ],
    )?;
    let link = run_in(
        &dir,
        &["write", "--link", &format!("{blocked}:depends_on:{ready}")],
    )?;
    assert!(link.status.success(), "link falhou: {:?}", link.stderr);

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
fn task_list_sort_impact_orders_critical_path() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let base = task_new(&dir, &["task", "new", "Base", "--scope", "task"])?;
    let parallel = task_new(&dir, &["task", "new", "Paralela", "--scope", "task"])?;
    let middle = task_dep(&dir, "Depende da base", &base)?;
    let _leaf = task_dep(&dir, "Depende do meio", &middle)?;

    let out = run_in(
        &dir,
        &["task", "list", "--ready", "--sort", "impact", "--explain"],
    )?;
    assert!(out.status.success(), "list falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    let base_line = line_of(&text, &base)?;
    let parallel_line = line_of(&text, &parallel)?;
    assert!(base_line.ends_with("|unblocks=2"), "linha: {base_line}");
    assert!(
        parallel_line.ends_with("|unblocks=0"),
        "linha: {parallel_line}"
    );
    assert!(
        text.find(&base).ok_or("base ausente")? < text.find(&parallel).ok_or("paralela ausente")?,
        "impacto não ordenou: {text}"
    );

    let json_out = run_in(
        &dir,
        &["--json", "task", "list", "--ready", "--sort", "impact"],
    )?;
    let value = json(&json_out)?;
    let first = value
        .get("data")
        .and_then(|data| data.get("tasks"))
        .and_then(|tasks| tasks.as_array())
        .and_then(|tasks| tasks.first())
        .ok_or("sem tasks")?;
    assert_eq!(
        first.get("id").and_then(|id| id.as_str()),
        Some(base.as_str())
    );
    assert_eq!(
        first.get("impact").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    Ok(())
}

#[test]
fn task_list_sort_impact_skips_closed() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let done = task_new(&dir, &["task", "new", "Feita", "--scope", "task"])?;
    let _waiting = task_dep(&dir, "Espera a feita", &done)?;
    let out = run_in(&dir, &["task", "update", &done, "--status", "closed"])?;
    assert!(out.status.success(), "close falhou: {:?}", out.stderr);

    let sorted = run_in(&dir, &["task", "list", "--ready", "--sort", "impact"])?;
    let text = String::from_utf8(sorted.stdout)?;
    assert!(!text.contains(&done), "closed apareceu no impacto: {text}");

    let ready = run_in(&dir, &["task", "list", "--ready"])?;
    let text = String::from_utf8(ready.stdout)?;
    assert!(text.contains(&done), "closed sumiu do --ready: {text}");
    Ok(())
}

#[test]
fn task_list_filters_by_tag_anchor_and_since() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let retry = task_new(
        &dir,
        &[
            "task",
            "new",
            "Ajustar backoff",
            "--scope",
            "task",
            "--tag",
            "retry",
            "--anchors",
            "src/retry.ts",
        ],
    )?;
    let storage = task_new(
        &dir,
        &[
            "task",
            "new",
            "Trocar storage",
            "--scope",
            "task",
            "--tag",
            "storage",
            "--anchors",
            "src/storage.ts",
        ],
    )?;

    let by_tag = run_in(&dir, &["task", "list", "--tag", "retry"])?;
    let text = String::from_utf8(by_tag.stdout)?;
    assert!(text.contains(&retry), "tag retry sumiu: {text}");
    assert!(
        !text.contains(&storage),
        "storage vazou no filtro de tag: {text}"
    );

    let by_anchor = run_in(&dir, &["task", "list", "--anchor", "src/retry.ts"])?;
    let text = String::from_utf8(by_anchor.stdout)?;
    assert!(text.contains(&retry), "âncora retry sumiu: {text}");
    assert!(
        !text.contains(&storage),
        "storage vazou no filtro de âncora: {text}"
    );

    let future = run_in(&dir, &["task", "list", "--since", "2999-01-01T00:00:00Z"])?;
    let text = String::from_utf8(future.stdout)?;
    assert!(
        !text.contains(&retry),
        "--since futuro devia excluir: {text}"
    );
    Ok(())
}

#[test]
fn task_show_multiple_ids_separator_and_partial() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let first = task_new(&dir, &["task", "new", "Primeira", "--scope", "task"])?;
    let second = task_new(&dir, &["task", "new", "Segunda", "--scope", "task"])?;

    let out = run_in(&dir, &["task", "show", &first, &second])?;
    assert!(out.status.success(), "show falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&first) && text.contains(&second),
        "ids ausentes: {text}"
    );
    assert!(text.contains("\n---\n"), "separador ausente: {text}");

    let partial = run_in(&dir, &["--json", "task", "show", &first, "task_zzzzzzzz"])?;
    assert!(
        partial.status.success(),
        "show parcial falhou: {:?}",
        partial.stderr
    );
    let value = json(&partial)?;
    let tasks = value
        .get("data")
        .and_then(|data| data.get("tasks"))
        .and_then(|tasks| tasks.as_array())
        .ok_or("sem tasks")?;
    assert_eq!(tasks.len(), 1);
    let warnings = value
        .get("warnings")
        .and_then(|warnings| warnings.as_array())
        .ok_or("sem warnings")?;
    assert_eq!(warnings.len(), 1);

    let missing = run_in(&dir, &["task", "show", "task_zzzzzzzz"])?;
    assert!(!missing.status.success(), "id único ausente devia falhar");
    Ok(())
}

#[test]
fn task_close_note_records_outcome_reason() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let task = task_new(
        &dir,
        &["task", "new", "Fechar com motivo", "--scope", "task"],
    )?;
    let out = run_in(
        &dir,
        &[
            "task",
            "close",
            &task,
            "--outcome",
            "success",
            "--note",
            "aplicado no job de retry",
        ],
    )?;
    assert!(out.status.success(), "close falhou: {:?}", out.stderr);

    let path = dir.join(".knudge").join("notas").join(format!("{task}.md"));
    let text = std::fs::read_to_string(path)?;
    assert!(
        text.contains("aplicado no job de retry"),
        "motivo ausente: {text}"
    );
    assert!(text.contains("outcomes"), "outcomes ausente: {text}");

    let bad = run_in(&dir, &["task", "close", &task, "--note", "sem outcome"])?;
    assert_eq!(
        bad.status.code(),
        Some(2),
        "--note sem --outcome deve ser 2"
    );
    Ok(())
}

/// Cria uma tarefa `--scope task` dependente de `dep` (dependência via `write --link`).
fn task_dep(dir: &Path, statement: &str, dep: &str) -> Result<String, Box<dyn std::error::Error>> {
    let id = task_new(dir, &["task", "new", statement, "--scope", "task"])?;
    let link = run_in(dir, &["write", "--link", &format!("{id}:depends_on:{dep}")])?;
    if !link.status.success() {
        return Err(format!("link falhou: {:?}", link.stderr).into());
    }
    Ok(id)
}

#[test]
fn task_show_includes_context() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let issue = task_new(&dir, &["task", "new", "Issue", "--scope", "issue"])?;
    let a = task_new(
        &dir,
        &["task", "new", "A", "--scope", "task", "--parent", &issue],
    )?;
    let b = task_new(
        &dir,
        &["task", "new", "B", "--scope", "task", "--parent", &issue],
    )?;
    let link = run_in(&dir, &["write", "--link", &format!("{b}:depends_on:{a}")])?;
    assert!(link.status.success(), "link falhou: {:?}", link.stderr);

    let out = run_in(&dir, &["task", "show", &b])?;
    assert!(out.status.success(), "show falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("pai: {issue}|Issue")),
        "pai ausente: {text}"
    );
    assert!(
        text.contains(&format!("bloqueado_por: {a}|A")),
        "bloqueador ausente: {text}"
    );

    let out = run_in(&dir, &["task", "show", &a, "--json"])?;
    let json = String::from_utf8(out.stdout)?;
    assert!(json.contains("\"blocks\""), "blocks ausente: {json}");
    assert!(
        json.contains(&format!("\"id\":\"{b}\"")),
        "blocks sem {b}: {json}"
    );
    Ok(())
}

#[test]
fn task_close_reports_epic_progress() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(&dir, &["task", "new", "Épico", "--scope", "epic"])?;
    let issue = task_new(
        &dir,
        &[
            "task", "new", "Issue", "--scope", "issue", "--parent", &epic,
        ],
    )?;
    let a = task_new(
        &dir,
        &["task", "new", "A", "--scope", "task", "--parent", &issue],
    )?;
    let _b = task_new(
        &dir,
        &["task", "new", "B", "--scope", "task", "--parent", &issue],
    )?;

    let out = run_in(&dir, &["task", "close", &a, "--outcome", "success"])?;
    assert!(out.status.success(), "close falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("epico: {epic}|Épico (1/2)")),
        "rollup ausente: {text}"
    );

    let out = run_in(&dir, &["task", "show", &a])?;
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("epico: {epic}|Épico (1/2)")),
        "show sem épico: {text}"
    );

    let out = run_in(&dir, &["task", "graph", "--root", &epic])?;
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("(1/2)"), "graph sem progresso: {text}");
    Ok(())
}

#[test]
fn knowledge_map_reports_container_axis() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(&dir, &["task", "new", "Épico", "--scope", "epic"])?;
    let issue = task_new(
        &dir,
        &[
            "task", "new", "Issue", "--scope", "issue", "--parent", &epic,
        ],
    )?;
    let a = task_new(
        &dir,
        &["task", "new", "A", "--scope", "task", "--parent", &issue],
    )?;

    let out = run_in(
        &dir,
        &["knowledge", "map", "--axis", "container", "--members"],
    )?;
    assert!(out.status.success(), "map falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("container|{epic}|Épico|2")),
        "cluster do épico ausente: {text}"
    );
    assert!(text.contains(&format!("{a}|A")), "membro A ausente: {text}");

    let out = run_in(&dir, &["--json", "knowledge", "map", "--axis", "container"])?;
    let json = String::from_utf8(out.stdout)?;
    assert!(
        json.contains("\"axis\":\"container\""),
        "json sem eixo: {json}"
    );
    assert!(
        json.contains(&format!("\"key\":\"{epic}\"")),
        "json sem o épico: {json}"
    );

    let bad = run_in(&dir, &["knowledge", "map", "--axis", "foo"])?;
    assert_eq!(bad.status.code(), Some(2), "eixo inválido deve ser 2");
    Ok(())
}

/// Linha de `task list` que começa com `id`.
fn line_of(text: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    text.lines()
        .find(|line| line.starts_with(id))
        .map(str::to_string)
        .ok_or_else(|| format!("linha ausente: {id}").into())
}

#[test]
fn task_kind_sets_type_and_filters() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let bug = task_new(
        &dir,
        &[
            "task",
            "new",
            "off-by-one",
            "--scope",
            "issue",
            "--kind",
            "error",
        ],
    )?;
    assert!(bug.starts_with("error_"), "id inesperado: {bug}");
    let story = task_new(
        &dir,
        &[
            "task",
            "new",
            "medir latencia",
            "--scope",
            "issue",
            "--kind",
            "question",
        ],
    )?;

    let out = run_in(&dir, &["task", "list", "--kind", "error"])?;
    assert!(out.status.success(), "list falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains(&bug), "--kind error não achou {bug}: {text}");
    assert!(
        !text.contains(&story),
        "--kind error incluiu {story}: {text}"
    );
    Ok(())
}

#[test]
fn task_claim_sets_and_clears_owner() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let task = task_new(&dir, &["task", "new", "fazer", "--scope", "task"])?;

    let claim = run_in(&dir, &["task", "claim", &task, "--by", "agente-a"])?;
    assert!(claim.status.success(), "claim falhou: {:?}", claim.stderr);
    let owned = run_in(&dir, &["--json", "task", "list", "--owner", "agente-a"])?;
    assert!(owned.status.success(), "list falhou: {:?}", owned.stderr);
    let value = json(&owned)?;
    let tasks = value
        .get("data")
        .and_then(|data| data.get("tasks"))
        .and_then(|tasks| tasks.as_array())
        .ok_or("sem tasks")?;
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks
            .first()
            .and_then(|row| row.get("owner"))
            .and_then(|owner| owner.as_str()),
        Some("agente-a")
    );

    let mine = run_env(
        &dir,
        &["--json", "task", "list", "--mine"],
        &[("KNUDGE_AGENT", "agente-a")],
    )?;
    assert!(mine.status.success(), "mine falhou: {:?}", mine.stderr);
    let missing = run_in(&dir, &["task", "list", "--mine"])?;
    assert_eq!(missing.status.code(), Some(2));

    let release = run_in(&dir, &["task", "claim", &task, "--release"])?;
    assert!(
        release.status.success(),
        "release falhou: {:?}",
        release.stderr
    );
    let after = run_in(&dir, &["task", "list", "--owner", "agente-a"])?;
    assert!(
        !String::from_utf8(after.stdout)?.contains(&task),
        "release não limpou o dono de {task}"
    );
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
fn ask_semantic_channel_reads_vector_index() -> TestResult {
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

    let provider = run_in(
        &dir,
        &["config", "set", "embeddings.provider", "lightweight"],
    )?;
    assert!(
        provider.status.success(),
        "config falhou: {:?}",
        provider.stderr
    );
    let drain = run_in(&dir, &["maintenance", "index", "--drain"])?;
    assert!(drain.status.success(), "index falhou: {:?}", drain.stderr);

    // Com o canal vetorial ligado (default), o `ask` acha a nota indexada.
    let ask = run_in(&dir, &["--json", "ask", "body_hash cache"])?;
    assert!(ask.status.success(), "ask falhou: {:?}", ask.stderr);
    let value = json(&ask)?;
    let hits = value
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .ok_or("ask sem hits")?;
    assert!(
        hits.iter()
            .any(|hit| hit.get("id").and_then(|id| id.as_str()) == Some(id.as_str())),
        "hit ausente: {hits:?}"
    );

    // Desligar o canal preserva o resultado lexical.
    let off = run_in(&dir, &["config", "set", "recall.semantic", "false"])?;
    assert!(off.status.success(), "config falhou: {:?}", off.stderr);
    let ask_off = run_in(&dir, &["--json", "ask", "body_hash cache"])?;
    assert!(ask_off.status.success(), "ask falhou: {:?}", ask_off.stderr);
    let off_hits = json(&ask_off)?;
    assert!(
        off_hits
            .get("data")
            .and_then(|data| data.get("hits"))
            .and_then(|hits| hits.as_array())
            .is_some_and(|hits| {
                hits.iter()
                    .any(|hit| hit.get("id").and_then(|id| id.as_str()) == Some(id.as_str()))
            })
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

#[test]
fn task_plan_prompt_and_submit_from_file() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(&dir, &["task", "new", "Programa", "--scope", "epic"])?;

    let prompt = run_in(
        &dir,
        &["task", "plan", &epic, "--prompt", "--template", "feature"],
    )?;
    assert!(
        prompt.status.success(),
        "prompt falhou: {:?}",
        prompt.stderr
    );
    let text = String::from_utf8(prompt.stdout)?;
    assert!(text.contains("template: feature"), "{text}");
    assert!(text.contains("min_steps: 2"), "{text}");

    let plan = "template: feature\nsections:\n  context: ctx\n  approach: app\n  steps:\n    - title: Corrigir off-by-one\n      kind: error\n    - title: Implementar retry\n  acceptance:\n    - teste passa\n";
    let path = dir.join("plan.toon");
    std::fs::write(&path, plan)?;
    let file = path.to_str().ok_or("path inválido")?;
    let submit = run_in(&dir, &["task", "plan", &epic, "--submit", "--from", file])?;
    assert!(
        submit.status.success(),
        "submit falhou: {:?}",
        submit.stderr
    );
    let ids = String::from_utf8(submit.stdout)?;
    assert_eq!(ids.lines().count(), 2, "esperava 2 filhos: {ids}");

    let graph = run_in(&dir, &["task", "graph", "--root", &epic])?;
    assert!(graph.status.success(), "graph falhou: {:?}", graph.stderr);
    let tree = String::from_utf8(graph.stdout)?;
    assert!(tree.contains("Bug"), "sem papel Bug: {tree}");
    assert!(tree.contains("Epic"), "sem papel Epic: {tree}");
    Ok(())
}

#[test]
fn task_plan_invalid_from_writes_nothing() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let epic = task_new(&dir, &["task", "new", "Programa", "--scope", "epic"])?;
    let before = run_in(&dir, &["--json", "task", "list"])?;
    let before = json(&before)?;

    // `acceptance` obrigatório ausente.
    let plan = "template: feature\nsections:\n  context: ctx\n  approach: app\n  steps:\n    - title: A\n    - title: B\n";
    let path = dir.join("plan.toon");
    std::fs::write(&path, plan)?;
    let file = path.to_str().ok_or("path inválido")?;
    let out = run_in(&dir, &["task", "plan", &epic, "--submit", "--from", file])?;
    assert_eq!(
        out.status.code(),
        Some(2),
        "esperava invalid_input: {out:?}"
    );

    let after = run_in(&dir, &["--json", "task", "list"])?;
    assert_eq!(json(&after)?, before, "plano inválido escreveu algo");
    Ok(())
}

#[test]
fn task_graph_reports_supervisor_mode() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let epic = task_new(&dir, &["task", "new", "Épico", "--scope", "epic"])?;
    let claimed = run_in(&dir, &["task", "claim", &epic, "--by", "supervisor"])?;
    assert!(
        claimed.status.success(),
        "claim falhou: {:?}",
        claimed.stderr
    );
    let first = task_new(
        &dir,
        &["task", "new", "A", "--scope", "issue", "--parent", &epic],
    )?;
    let second = task_new(
        &dir,
        &["task", "new", "B", "--scope", "issue", "--parent", &epic],
    )?;
    for (id, agent) in [(&first, "agente-a"), (&second, "agente-b")] {
        let out = run_in(&dir, &["task", "claim", id, "--by", agent])?;
        assert!(out.status.success(), "claim falhou: {:?}", out.stderr);
    }

    let graph = run_in(&dir, &["task", "graph", "--root", &epic])?;
    assert!(graph.status.success());
    let tree = String::from_utf8(graph.stdout)?;
    assert!(tree.contains("supervisor"), "sem modo supervisor: {tree}");
    assert!(tree.contains("agente-a"), "sem dono: {tree}");
    Ok(())
}

#[test]
fn ask_tags_lists_vocabulary() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    for (statement, tag) in [
        ("fila com retry", "retry"),
        ("backoff", "retry"),
        ("buffer", "queue"),
    ] {
        let out = run_in(&dir, &["write", statement, "--type", "fact", "--tag", tag])?;
        assert!(out.status.success(), "write falhou: {:?}", out.stderr);
    }

    let out = run_in(&dir, &["ask", "--tags"])?;
    assert!(out.status.success(), "ask --tags falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("retry|2"));
    assert_eq!(lines.next(), Some("queue|1"));

    let json_out = run_in(&dir, &["--json", "ask", "--tags"])?;
    let tags = json(&json_out)?
        .get("data")
        .and_then(|data| data.get("tags"))
        .and_then(|tags| tags.as_array())
        .cloned()
        .ok_or("sem tags")?;
    assert_eq!(tags.len(), 2);
    Ok(())
}

#[test]
fn rewind_manifest_shows_next_and_fresh() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(&dir, &["task", "new", "Épico", "--scope", "epic"])?;
    let story = task_new(
        &dir,
        &[
            "task", "new", "Story", "--scope", "issue", "--parent", &epic,
        ],
    )?;
    let _ready = task_new(
        &dir,
        &[
            "task", "new", "Pronta", "--scope", "task", "--parent", &story,
        ],
    )?;

    let out = run_in(&dir, &["rewind"])?;
    assert!(out.status.success(), "rewind falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("\nnext: "), "sem next: {text}");
    assert!(
        text.contains("\nfresh: stale=0 expiring=0 pending="),
        "sem fresh: {text}"
    );
    Ok(())
}

/// Escreve uma nota tipada com âncora e devolve o id.
fn write_typed(
    dir: &Path,
    statement: &str,
    note_type: &str,
    anchor: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "write",
            statement,
            "--type",
            note_type,
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

#[test]
fn task_outcome_promotes_anchored_note_in_ask() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let confirmed = write_typed(&dir, "jitter backoff alfa", "decision", "src/retry.ts")?;
    let plain = write_typed(&dir, "jitter backoff beta", "decision", "src/other.ts")?;

    let task = task_new(
        &dir,
        &[
            "task",
            "new",
            "implementar retry",
            "--scope",
            "task",
            "--anchors",
            "src/retry.ts",
        ],
    )?;
    let outcome = run_in(&dir, &["write", "--outcome", "success", &task])?;
    assert!(
        outcome.status.success(),
        "outcome falhou: {:?}",
        outcome.stderr
    );

    let after = run_in(&dir, &["--json", "ask", "jitter backoff"])?;
    assert!(after.status.success(), "ask falhou: {:?}", after.stderr);
    let hits = json(&after)?
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .cloned()
        .ok_or("sem hits")?;
    let confidence = |target: &str| {
        hits.iter()
            .find(|hit| hit.get("id").and_then(|value| value.as_str()) == Some(target))
            .and_then(|hit| hit.get("confidence"))
            .and_then(serde_json::Value::as_f64)
    };
    let confirmed_confidence = confidence(&confirmed).ok_or("confirmed ausente")?;
    let plain_confidence = confidence(&plain).ok_or("plain ausente")?;
    assert!(
        confirmed_confidence > plain_confidence,
        "confirmação não elevou: {confirmed_confidence} <= {plain_confidence}"
    );
    Ok(())
}

#[test]
fn rewind_files_promotes_task_confirmed_note() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let confirmed = write_typed(&dir, "usar full jitter", "decision", "src/retry.ts")?;
    let task = task_new(
        &dir,
        &[
            "task",
            "new",
            "implementar retry",
            "--scope",
            "task",
            "--anchors",
            "src/retry.ts",
        ],
    )?;
    let outcome = run_in(&dir, &["write", "--outcome", "success", &task])?;
    assert!(
        outcome.status.success(),
        "outcome falhou: {:?}",
        outcome.stderr
    );

    let out = run_in(&dir, &["rewind", "--files", "src/retry.ts"])?;
    assert!(out.status.success(), "rewind falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    let promoted = text
        .lines()
        .any(|line| line.starts_with(&format!("{confirmed}|")) && line.ends_with("|star"));
    assert!(promoted, "nota não promovida a star: {text}");
    Ok(())
}
