//! Testes de integração do binário `kd` (E01-T04/E12): contrato de saída, `--json`, EPIPE e
//! fluxos com estado num projeto temporário.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Alias de resultado dos testes (sem `unwrap`/`expect`, proibidos por D92).
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Executa `kd` com os argumentos dados e captura a saída.
///
/// Roda no diretório do repositório (não isolado): desliga o auto-drain ocioso para os testes
/// não tocarem o `.knudge/` real do desenvolvedor.
fn run(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kd"))
        .args(args)
        .env("KNUDGE_NO_IDLE", "1")
        .output()
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
fn no_args_shows_help() -> TestResult {
    let bare = run(&[])?;
    assert!(bare.status.success(), "`kd` sozinho deve ter exit 0");
    let help = run(&["--help"])?.stdout;
    assert_eq!(bare.stdout, help, "`kd` deve ser idêntico a `kd --help`");
    Ok(())
}

#[test]
fn json_without_verb_is_invalid_input() -> TestResult {
    let out = run(&["--json"])?;
    assert_eq!(out.status.code(), Some(2), "`--json` sem verbo é uso (2)");
    let value = json(&out)?;
    assert_eq!(value.get("success"), Some(&serde_json::Value::Bool(false)));
    Ok(())
}

#[test]
fn verb_without_args_shows_help() -> TestResult {
    for verb in [
        "task",
        "knowledge",
        "maintenance",
        "config",
        "self",
        "drain",
    ] {
        let out = run(&[verb])?;
        assert!(out.status.success(), "`kd {verb}` deve ter exit 0");
        let text = String::from_utf8(out.stdout)?;
        assert!(text.contains("Usage"), "`kd {verb}` sem help: {text}");
    }
    Ok(())
}

#[test]
fn help_subcommand_lists_verbs() -> TestResult {
    let out = run(&["help"])?;
    assert!(out.status.success(), "`kd help` deve ter exit 0");
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("Usage"), "help ausente: {text}");
    assert!(text.contains("ask"), "verbo ausente: {text}");
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
    let out = run(&["--json", "prime"])?;
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
        &[
            "--json",
            "write",
            "--summary",
            "o cache usa body_hash",
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
            "--summary",
            statement,
            "--type",
            "fact",
            "--anchor",
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

/// Cria uma nota `observational` com `created_at` recuado para 2000 — assim o shelf-life
/// **derivado** (D135) a considera vencida. Devolve o id.
fn write_aged_observational(
    dir: &Path,
    statement: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "write",
            "--summary",
            statement,
            "--type",
            "fact",
            "--class",
            "observational",
        ],
    )?;
    assert!(out.status.success(), "write falhou: {:?}", out.stderr);
    let id = json(&out)?
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("write sem id")?
        .to_string();
    let path = note_path(dir, &id);
    let text = std::fs::read_to_string(&path)?;
    let aged = text
        .lines()
        .map(|line| {
            if line.starts_with("created_at:") {
                "created_at: 2000-01-01T00:00:00.000Z".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, format!("{aged}\n"))?;
    Ok(id)
}

/// Caminho de uma nota no layout por tipo (`notas/<tipo>/<id>.md` — D150).
fn note_path(dir: &Path, id: &str) -> PathBuf {
    let prefix = id.split_once('_').map_or(id, |(prefix, _)| prefix);
    let folder = if prefix == "container" {
        "epic"
    } else {
        prefix
    };
    dir.join(".knudge")
        .join("notas")
        .join(folder)
        .join(format!("{id}.md"))
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

/// Cria uma tarefa com `--summary` + argumentos extras (D140).
fn task(dir: &Path, summary: &str, extra: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut args = vec!["task", "new", "--summary", summary];
    args.extend_from_slice(extra);
    task_new(dir, &args)
}

/// Define uma config do projeto via CLI.
fn config_set(dir: &Path, key: &str, value: &str) -> std::io::Result<Output> {
    run_in(dir, &["config", "set", "--key", key, "--value", value])
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

    let cap = run_in(
        &dir,
        &["config", "set", "--key", "write.batch_max", "--value", "1"],
    )?;
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
    let out = run_in(&dir, &["write", "--outcome", "success", "--id", &confirmed])?;
    assert!(out.status.success(), "outcome falhou: {:?}", out.stderr);

    let ranked = run_in(&dir, &["knowledge", "rank", "--universe"])?;
    assert!(ranked.status.success(), "rank falhou: {:?}", ranked.stderr);
    let text = String::from_utf8(ranked.stdout)?;
    let first = text.lines().next().ok_or("sem linhas")?;
    assert!(
        first.starts_with(&confirmed),
        "primeiro não é o confirmado: {text}"
    );
    assert!(first.contains("stars"), "why esperado stars: {first}");

    let envelope = json(&run_in(
        &dir,
        &["--json", "knowledge", "rank", "--universe"],
    )?)?;
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

    let expired = write_aged_observational(&dir, "nota vencida")?;
    let fresh = write_anchored(&dir, "nota viva", "src/x.rs")?;

    let out = run_in(&dir, &["--json", "maintenance", "prune", "--universe"])?;
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
            "--id",
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

    let epic = task(&dir, "Programa", &["--scope", "epic"])?;
    let issue = task(&dir, "Story", &["--scope", "issue", "--parent", &epic])?;
    let ready = task(&dir, "Pronta", &["--scope", "task", "--parent", &issue])?;
    let blocked = task(&dir, "Bloqueada", &["--scope", "task", "--parent", &issue])?;
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

    let base = task_new(
        &dir,
        &["task", "new", "--summary", "Base", "--scope", "task"],
    )?;
    let parallel = task_new(
        &dir,
        &["task", "new", "--summary", "Paralela", "--scope", "task"],
    )?;
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

    let done = task_new(
        &dir,
        &["task", "new", "--summary", "Feita", "--scope", "task"],
    )?;
    let _waiting = task_dep(&dir, "Espera a feita", &done)?;
    let out = run_in(
        &dir,
        &["task", "update", "--id", &done, "--status", "closed"],
    )?;
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
            "--summary",
            "Ajustar backoff",
            "--scope",
            "task",
            "--tag",
            "retry",
            "--anchor",
            "src/retry.ts",
        ],
    )?;
    let storage = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Trocar storage",
            "--scope",
            "task",
            "--tag",
            "storage",
            "--anchor",
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

    let first = task_new(
        &dir,
        &["task", "new", "--summary", "Primeira", "--scope", "task"],
    )?;
    let second = task_new(
        &dir,
        &["task", "new", "--summary", "Segunda", "--scope", "task"],
    )?;

    let out = run_in(&dir, &["task", "show", "--id", &first, &second])?;
    assert!(out.status.success(), "show falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&first) && text.contains(&second),
        "ids ausentes: {text}"
    );
    assert!(text.contains("\n---\n"), "separador ausente: {text}");

    let partial = run_in(
        &dir,
        &["--json", "task", "show", "--id", &first, "task_zzzzzzzz"],
    )?;
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

    let missing = run_in(&dir, &["task", "show", "--id", "task_zzzzzzzz"])?;
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
        &[
            "task",
            "new",
            "--summary",
            "Fechar com motivo",
            "--scope",
            "task",
        ],
    )?;
    let out = run_in(
        &dir,
        &[
            "task",
            "close",
            "--id",
            &task,
            "--outcome",
            "success",
            "--note",
            "aplicado no job de retry",
        ],
    )?;
    assert!(out.status.success(), "close falhou: {:?}", out.stderr);

    let path = note_path(&dir, &task);
    let text = std::fs::read_to_string(path)?;
    assert!(
        text.contains("aplicado no job de retry"),
        "motivo ausente: {text}"
    );
    assert!(text.contains("outcomes"), "outcomes ausente: {text}");

    let bad = run_in(
        &dir,
        &["task", "close", "--id", &task, "--note", "sem outcome"],
    )?;
    assert_eq!(
        bad.status.code(),
        Some(2),
        "--note sem --outcome deve ser 2"
    );
    Ok(())
}

/// Cria uma tarefa `--scope task` dependente de `dep` (dependência via `write --link`).
fn task_dep(dir: &Path, statement: &str, dep: &str) -> Result<String, Box<dyn std::error::Error>> {
    let id = task_new(
        dir,
        &["task", "new", "--summary", statement, "--scope", "task"],
    )?;
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

    let issue = task_new(
        &dir,
        &["task", "new", "--summary", "Issue", "--scope", "issue"],
    )?;
    let a = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "A",
            "--scope",
            "task",
            "--parent",
            &issue,
        ],
    )?;
    let b = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "B",
            "--scope",
            "task",
            "--parent",
            &issue,
        ],
    )?;
    let link = run_in(&dir, &["write", "--link", &format!("{b}:depends_on:{a}")])?;
    assert!(link.status.success(), "link falhou: {:?}", link.stderr);

    let out = run_in(&dir, &["task", "show", "--id", &b])?;
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

    let out = run_in(&dir, &["task", "show", "--id", &a, "--json"])?;
    let json = String::from_utf8(out.stdout)?;
    assert!(json.contains("\"blocks\""), "blocks ausente: {json}");
    assert!(
        json.contains(&format!("\"id\":\"{b}\"")),
        "blocks sem {b}: {json}"
    );
    Ok(())
}

#[test]
fn task_show_includes_full_fields() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let id = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Endurecer",
            "--scope",
            "task",
            "--kind",
            "risk",
            "--tag",
            "parser",
            "--anchor",
            "src/toon/parse.rs",
            "--checks",
            "testes",
            "rejeitar NBSP",
        ],
    )?;

    let out = run_in(&dir, &["task", "show", "--id", &id])?;
    assert!(out.status.success(), "show falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    for needle in [
        "tipo: risk",
        "corpo:",
        "rejeitar NBSP",
        "checks: testes",
        "ancoras: src/toon/parse.rs",
        "tags: parser",
    ] {
        assert!(text.contains(needle), "falta `{needle}`: {text}");
    }

    let out = run_in(&dir, &["--json", "task", "show", "--id", &id])?;
    let value = json(&out)?;
    let task = value
        .get("data")
        .and_then(|data| data.get("tasks"))
        .and_then(|tasks| tasks.as_array())
        .and_then(|tasks| tasks.first())
        .ok_or("sem task")?;
    assert_eq!(task.get("kind").and_then(|k| k.as_str()), Some("risk"));
    assert_eq!(
        task.get("checks").and_then(|c| c.as_array()).map(Vec::len),
        Some(1)
    );
    assert_eq!(
        task.get("anchors").and_then(|a| a.as_array()).map(Vec::len),
        Some(1)
    );
    assert_eq!(
        task.get("tags").and_then(|t| t.as_array()).map(Vec::len),
        Some(1)
    );
    assert!(task.get("outcomes").and_then(|o| o.as_array()).is_some());
    Ok(())
}

#[test]
fn task_show_lists_outcomes() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let id = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Com evidência",
            "--scope",
            "task",
        ],
    )?;
    let close = run_in(
        &dir,
        &[
            "task",
            "close",
            "--id",
            &id,
            "--outcome",
            "success",
            "--note",
            "verde",
        ],
    )?;
    assert!(close.status.success(), "close falhou: {:?}", close.stderr);

    let out = run_in(&dir, &["task", "show", "--id", &id])?;
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains("outcomes: success(verde)"),
        "outcomes ausente: {text}"
    );
    Ok(())
}

#[test]
fn task_list_full_content_renders_blocks() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let a = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Alfa",
            "--scope",
            "task",
            "corpo alfa",
        ],
    )?;
    let b = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Beta",
            "--scope",
            "task",
            "corpo beta",
        ],
    )?;

    let out = run_in(&dir, &["task", "list", "--full-content", "--universe"])?;
    assert!(out.status.success(), "list falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("\n---\n"), "separador ausente: {text}");
    assert!(
        text.contains(&a) && text.contains(&b),
        "ids ausentes: {text}"
    );
    assert!(
        text.contains("corpo alfa") && text.contains("corpo beta"),
        "corpos ausentes: {text}"
    );
    assert!(text.contains("scope: task"), "sem escopo: {text}");

    let out = run_in(
        &dir,
        &["--json", "task", "list", "--full-content", "--universe"],
    )?;
    let value = json(&out)?;
    let tasks = value
        .get("data")
        .and_then(|data| data.get("tasks"))
        .and_then(|tasks| tasks.as_array())
        .ok_or("sem tasks")?;
    assert_eq!(tasks.len(), 2);
    let first = tasks.first().ok_or("sem task")?;
    assert!(first.get("body").and_then(|body| body.as_str()).is_some());
    assert!(first.get("checks").and_then(|c| c.as_array()).is_some());
    Ok(())
}

#[test]
fn task_close_reports_epic_progress() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task(&dir, "Épico", &["--scope", "epic"])?;
    let issue = task(&dir, "Issue", &["--scope", "issue", "--parent", &epic])?;
    let a = task(&dir, "A", &["--scope", "task", "--parent", &issue])?;
    let _b = task(&dir, "B", &["--scope", "task", "--parent", &issue])?;

    let out = run_in(&dir, &["task", "close", "--id", &a, "--outcome", "success"])?;
    assert!(out.status.success(), "close falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("epico: {epic}|Épico (1/2)")),
        "rollup ausente: {text}"
    );

    let out = run_in(&dir, &["task", "show", "--id", &a])?;
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
#[allow(
    clippy::too_many_lines,
    reason = "teste de integração encadeia setup e asserts"
)]
fn knowledge_map_reports_container_axis() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Épico", "--scope", "epic"],
    )?;
    let issue = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Issue",
            "--scope",
            "issue",
            "--parent",
            &epic,
        ],
    )?;
    let a = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "A",
            "--scope",
            "task",
            "--parent",
            &issue,
        ],
    )?;

    let out = run_in(
        &dir,
        &[
            "knowledge",
            "map",
            "--axis",
            "scope",
            "--members",
            "--universe",
        ],
    )?;
    assert!(out.status.success(), "map falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(
        text.contains(&format!("scope|{epic}|Épico|2")),
        "cluster do épico ausente: {text}"
    );
    assert!(text.contains(&format!("{a}|A")), "membro A ausente: {text}");

    let out = run_in(
        &dir,
        &[
            "--json",
            "knowledge",
            "map",
            "--axis",
            "scope",
            "--universe",
        ],
    )?;
    let json = String::from_utf8(out.stdout)?;
    assert!(json.contains("\"axis\":\"scope\""), "json sem eixo: {json}");
    assert!(
        json.contains(&format!("\"key\":\"{epic}\"")),
        "json sem o épico: {json}"
    );

    let bad = run_in(&dir, &["knowledge", "map", "--axis", "foo"])?;
    assert_eq!(bad.status.code(), Some(2), "eixo inválido deve ser 2");
    Ok(())
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "teste de integração encadeia setup e asserts"
)]
fn knowledge_map_write_materializes_map_and_hubs() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Épico", "--scope", "epic"],
    )?;
    let issue = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Issue",
            "--scope",
            "issue",
            "--parent",
            &epic,
        ],
    )?;
    let _a = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "A",
            "--scope",
            "task",
            "--parent",
            &issue,
        ],
    )?;

    let out = run_in(
        &dir,
        &[
            "knowledge",
            "map",
            "--axis",
            "scope",
            "--write",
            "--universe",
        ],
    )?;
    assert!(out.status.success(), "map --write falhou: {:?}", out.stderr);

    let map = dir.join(".knudge").join("notas").join("MAP.md");
    let text = std::fs::read_to_string(&map)?;
    assert!(
        text.contains("# Mapa de conhecimento"),
        "MAP.md sem título: {text}"
    );
    assert!(text.contains(&epic), "MAP.md sem o épico: {text}");

    // Hub é uma nota `meta` real que `references` o épico.
    let meta_dir = dir.join(".knudge").join("notas").join("meta");
    let mut found = false;
    for entry in std::fs::read_dir(&meta_dir)? {
        let content = std::fs::read_to_string(entry?.path())?;
        if content.contains("references") && content.contains(&epic) {
            found = true;
        }
    }
    assert!(found, "nenhum hub referenciando o épico");

    // O hub entra no `ask` (é nota de verdade).
    let out = run_in(&dir, &["--json", "ask", "Mapa de conhecimento"])?;
    let json = String::from_utf8(out.stdout)?;
    assert!(json.contains("meta_"), "ask não achou o hub: {json}");
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
            "--summary",
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
            "--summary",
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
fn task_claim_and_owner_flags_are_gone() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let _task = task_new(
        &dir,
        &["task", "new", "--summary", "fazer", "--scope", "task"],
    )?;

    for args in [
        vec!["task", "claim", "task_00000000", "--by", "agente-a"],
        vec!["task", "list", "--owner", "agente-a"],
        vec!["task", "list", "--since", "2020-01-01T00:00:00Z"],
    ] {
        let out = run_in(&dir, &args)?;
        assert_eq!(out.status.code(), Some(2), "deveria ser exit 2: {args:?}");
    }
    // `--mine` é desconhecido mesmo com `KNUDGE_AGENT` definido (não é mais lido).
    let mine = run_env(
        &dir,
        &["task", "list", "--mine"],
        &[("KNUDGE_AGENT", "agente-a")],
    )?;
    assert_eq!(mine.status.code(), Some(2), "`--mine` deveria ser exit 2");
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
            "--summary",
            "Programa",
            "--scope",
            "epic",
            "--anchor",
            "plan/foo.md",
        ],
    )?;
    let story = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Story",
            "--scope",
            "issue",
            "--parent",
            &root,
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
fn task_graph_program_renders_forest_for_multiple_epics() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let first = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Programa A",
            "--scope",
            "epic",
            "--anchor",
            "plan/foo.md",
        ],
    )?;
    let second = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Programa B",
            "--scope",
            "epic",
            "--anchor",
            "plan/foo.md",
        ],
    )?;

    let out = run_in(&dir, &["task", "graph", "--program", "plan/foo.md"])?;
    assert!(out.status.success(), "graph falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    let first_pos = text.find(&first).ok_or("sem o épico A")?;
    let second_pos = text.find(&second).ok_or("sem o épico B")?;
    let (a, b) = if first < second {
        (first_pos, second_pos)
    } else {
        (second_pos, first_pos)
    };
    assert!(a < b, "floresta fora da ordem de id: {text}");
    Ok(())
}

#[test]
fn task_graph_without_containers_warns() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let new = run_in(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "tarefa solta",
            "--scope",
            "task",
        ],
    )?;
    assert!(new.status.success(), "task falhou: {:?}", new.stderr);
    let out = run_in(&dir, &["task", "graph"])?;
    assert!(out.status.success(), "graph falhou: {:?}", out.stderr);
    let stderr = String::from_utf8(out.stderr)?;
    assert!(
        stderr.contains("nenhum container"),
        "aviso de container ausente: {stderr}"
    );
    Ok(())
}

#[test]
fn doctor_reports_health_and_audit() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["doctor"])?;
    assert!(out.status.success(), "doctor falhou: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("auditoria:"), "saída sem auditoria: {text}");
    assert!(text.contains("próximos:"), "saída sem próximos: {text}");
    Ok(())
}

#[test]
fn doctor_explain_lists_expected_found_action() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["doctor", "--explain"])?;
    assert!(out.status.success(), "doctor --explain: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("esperado:"), "sem esperado:\n{text}");
    assert!(text.contains("encontrado:"), "sem encontrado:\n{text}");
    assert!(text.contains("ação:"), "sem ação:\n{text}");
    Ok(())
}

#[test]
fn doctor_status_reflects_audit_and_fix_resolves() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src").join("gone.rs"), "x")?;
    let write = run_in(
        &dir,
        &[
            "write",
            "--summary",
            "nota ancorada do doctor",
            "--type",
            "fact",
            "--anchor",
            "src/gone.rs",
            "Por quê: smoke do doctor",
        ],
    )?;
    assert!(write.status.success(), "write: {:?}", write.stderr);
    std::fs::remove_file(dir.join("src").join("gone.rs"))?;

    let envelope = json(&run_in(&dir, &["--json", "doctor"])?)?;
    let data = envelope.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("healthy").and_then(serde_json::Value::as_bool),
        Some(false),
        "âncora quebrada não pode ficar saudável: {data}"
    );
    let audit = data.get("audit").ok_or("sem audit")?;
    let broken = audit
        .get("broken_anchor_details")
        .and_then(serde_json::Value::as_array)
        .ok_or("sem broken_anchor_details")?;
    assert!(!broken.is_empty(), "auditoria não listou a âncora quebrada");

    let _fixed = run_in(&dir, &["doctor", "--fix"])?;
    let envelope = json(&run_in(&dir, &["--json", "doctor"])?)?;
    let data = envelope.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("healthy").and_then(serde_json::Value::as_bool),
        Some(true),
        "após --fix o corpus deve ficar saudável: {data}"
    );
    assert_eq!(
        data.get("status").and_then(serde_json::Value::as_str),
        Some("healthy")
    );
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
            "--summary",
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

    let forget = run_in(&dir, &["forget", "--id", id.as_str()])?;
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
fn ask_full_content_and_brief_contract() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let write = run_in(
        &dir,
        &[
            "--json",
            "write",
            "--summary",
            "formato TOON é linha a linha",
            "--type",
            "def",
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

    let with_body = run_in(&dir, &["ask", "TOON", "--full-content"])?;
    assert!(
        with_body.status.success(),
        "ask --full-content falhou: {:?}",
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
    let out = run_in(
        &dir,
        &["--json", "write", "--summary", "algo", "--type", "task"],
    )?;
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
    let out = run_in(
        &dir,
        &[
            "--json",
            "task",
            "new",
            "--summary",
            "passo",
            "--scope",
            "task",
        ],
    )?;
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
fn ask_without_mode_returns_usage() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["ask"])?;
    assert_eq!(out.status.code(), Some(2), "ask vazio devia ser 2");
    let text = String::from_utf8(out.stderr)?;
    assert!(
        text.contains("Usage") && text.contains("--around"),
        "help do verbo ausente: {text}"
    );
    Ok(())
}

#[test]
fn removed_anchors_alias_is_rejected() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let cases: [&[&str]; 3] = [
        &["write", "--summary", "x", "--anchors", "src/a.rs"],
        &[
            "task",
            "new",
            "--summary",
            "x",
            "--scope",
            "task",
            "--anchors",
            "src/a.rs",
        ],
        &["task", "list", "--universe", "--anchors", "src/a.rs"],
    ];
    for args in cases {
        let out = run_in(&dir, args)?;
        assert_eq!(
            out.status.code(),
            Some(2),
            "alias removido deveria ser 2: {args:?}"
        );
    }
    Ok(())
}

#[test]
fn write_without_statement_is_invalid() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["--json", "write", "--type", "fact"])?;
    assert_eq!(out.status.code(), Some(2), "write vazio devia ser 2");
    let value = json(&out)?;
    assert_eq!(value.get("success"), Some(&serde_json::Value::Bool(false)));
    Ok(())
}

#[test]
fn task_new_without_statement_is_invalid() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["task", "new", "--summary", "", "--scope", "task"])?;
    assert_eq!(out.status.code(), Some(2), "tarefa vazia devia ser 2");
    Ok(())
}

#[test]
fn d140_write_positional_is_body() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    // Corpo inline pelo posicional.
    let out = run_in(
        &dir,
        &[
            "--json",
            "write",
            "--summary",
            "Resumo inline",
            "corpo inline",
        ],
    )?;
    assert!(out.status.success(), "write falhou: {:?}", out.stderr);
    let id = json(&out)?
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("write sem id")?
        .to_string();
    let text = std::fs::read_to_string(note_path(&dir, &id))?;
    assert!(text.contains("Resumo inline"), "sem summary: {text}");
    assert!(text.contains("corpo inline"), "sem corpo: {text}");

    // `-` lê stdin.
    let piped = run_stdin(
        &dir,
        &["write", "--summary", "Via pipe explícito", "-"],
        "corpo do pipe\n",
    )?;
    assert!(piped.status.success(), "pipe falhou: {:?}", piped.stderr);

    // Sem posicional + stdin não-TTY lê o corpo.
    let piped = run_stdin(
        &dir,
        &["write", "--summary", "Via pipe implícito"],
        "corpo implícito\n",
    )?;
    assert!(
        piped.status.success(),
        "pipe implícito falhou: {:?}",
        piped.stderr
    );

    // `--body` não existe mais.
    let bad = run_in(&dir, &["write", "--summary", "S", "--body", "x"])?;
    assert_eq!(bad.status.code(), Some(2), "`--body` deveria ser exit 2");
    Ok(())
}

#[test]
fn d140_task_new_positional_and_named_ids() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    // `task new`: posicional = corpo; `--summary` = afirmação.
    let task = task(&dir, "Tarefa", &["corpo da tarefa", "--scope", "task"])?;
    let show = run_in(&dir, &["task", "show", "--id", &task])?;
    let text = String::from_utf8(show.stdout)?;
    assert!(text.contains("corpo da tarefa"), "sem corpo: {text}");

    // `--body` não existe em `task new`.
    let bad = run_in(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "T",
            "--body",
            "x",
            "--scope",
            "task",
        ],
    )?;
    assert_eq!(bad.status.code(), Some(2), "task `--body` devia ser exit 2");

    // Posicional em verbos sem conteúdo → exit 2.
    for args in [
        vec!["task", "show", task.as_str()],
        vec!["forget", task.as_str()],
        vec!["sync", "foo"],
        vec!["prime", "foo"],
    ] {
        let out = run_in(&dir, &args)?;
        assert_eq!(out.status.code(), Some(2), "{args:?} devia ser exit 2");
    }
    Ok(())
}

#[test]
fn task_batch_creates_with_parent_and_edges() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let batch = concat!(
        r#"{"key":"e","statement":"Programa","scope":"epic"}"#,
        "\n",
        r#"{"key":"i","statement":"Story","scope":"issue","parent":"e"}"#,
        "\n",
        r#"{"key":"a","statement":"A","scope":"task","parent":"i","depends_on":["i"]}"#,
        "\n",
        r#"{"key":"b","statement":"B","scope":"task","parent":"i","depends_on":["a"]}"#,
        "\n",
    );
    let out = run_stdin(&dir, &["--json", "task", "new", "--batch", "-"], batch)?;
    assert!(out.status.success(), "batch falhou: {:?}", out.stderr);
    let value = json(&out)?;
    let data = value.get("data").ok_or("sem data")?;
    let keys = data
        .get("keys")
        .and_then(|keys| keys.as_object())
        .ok_or("sem keys")?;
    let epic = keys.get("e").and_then(|v| v.as_str()).ok_or("sem e")?;
    let a = keys.get("a").and_then(|v| v.as_str()).ok_or("sem a")?;
    let items = data
        .get("items")
        .and_then(|items| items.as_array())
        .ok_or("sem items")?;
    let b_item = items
        .iter()
        .find(|item| item.get("key").and_then(|k| k.as_str()) == Some("b"))
        .ok_or("sem b")?;
    let edges = b_item
        .get("edges")
        .and_then(|edges| edges.as_array())
        .ok_or("sem edges")?;
    assert!(
        edges
            .iter()
            .any(|edge| edge.get("to").and_then(|t| t.as_str()) == Some(a)
                && edge.get("kind").and_then(|k| k.as_str()) == Some("depends_on")),
        "depends_on não resolvido: {edges:?}"
    );
    let graph = run_in(&dir, &["task", "graph", "--root", epic])?;
    let text = String::from_utf8(graph.stdout)?;
    assert!(text.contains(a), "graph sem a: {text}");
    Ok(())
}

#[test]
fn task_batch_dry_run_and_warnings() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let batch = "{\"statement\":\"Nada\",\"scope\":\"task\"}\n{nao-json}";
    let out = run_stdin(
        &dir,
        &["--json", "task", "new", "--batch", "-", "--dry-run"],
        batch,
    )?;
    assert!(out.status.success(), "dry-run falhou: {:?}", out.stderr);
    let value = json(&out)?;
    assert!(
        value
            .get("warnings")
            .and_then(|warnings| warnings.as_array())
            .is_some_and(|warnings| !warnings.is_empty()),
        "linha inválida sem aviso"
    );
    let list = run_in(&dir, &["task", "list"])?;
    let text = String::from_utf8(list.stdout)?;
    assert!(!text.contains("Nada"), "dry-run gravou: {text}");
    Ok(())
}

#[test]
fn task_params_reparent_and_max() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    // `--params` cria um item.
    let params = r#"{"statement":"Único","scope":"task","body":"corpo"}"#;
    let out = run_in(&dir, &["--json", "task", "new", "--params", params])?;
    assert!(out.status.success(), "params falhou: {:?}", out.stderr);
    let value = json(&out)?;
    let item = value
        .get("data")
        .and_then(|data| data.get("items"))
        .and_then(|items| items.as_array())
        .and_then(|items| items.first())
        .ok_or("sem item")?;
    assert_eq!(item.get("action").and_then(|a| a.as_str()), Some("created"));

    // Re-parenta via `id`+`parent`.
    let epic = task(&dir, "Raiz", &["--scope", "epic"])?;
    let orphan = task(&dir, "Órfã", &["--scope", "issue"])?;
    let update = format!(r#"{{"id":"{orphan}","statement":"Órfã","parent":"{epic}"}}"#);
    let out = run_in(&dir, &["task", "new", "--params", &update])?;
    assert!(out.status.success(), "update falhou: {:?}", out.stderr);
    let show = run_in(&dir, &["task", "show", "--id", &orphan])?;
    let text = String::from_utf8(show.stdout)?;
    assert!(
        text.contains(&format!("pai: {epic}|Raiz")),
        "sem pai: {text}"
    );

    // Teto `task.batch_max`.
    let cap = config_set(&dir, "task.batch_max", "1")?;
    assert!(cap.status.success());
    let batch =
        "{\"statement\":\"A\",\"scope\":\"task\"}\n{\"statement\":\"B\",\"scope\":\"task\"}";
    let out = run_stdin(&dir, &["task", "new", "--batch", "-"], batch)?;
    assert_eq!(out.status.code(), Some(2), "acima do teto devia ser 2");
    Ok(())
}

#[test]
fn d147_params_universal() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    // `write --params` cria a nota.
    let params = r#"{"statement":"Cache LRU","body":"detalhe","type":"decision","tags":["cache"]}"#;
    let out = run_in(&dir, &["--json", "write", "--params", params])?;
    assert!(
        out.status.success(),
        "write params falhou: {:?}",
        out.stderr
    );
    let value = json(&out)?;
    let id = value
        .get("data")
        .and_then(|data| data.get("items"))
        .and_then(|items| items.as_array())
        .and_then(|items| items.first())
        .and_then(|item| item.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("sem id")?
        .to_string();

    // `ask --params` executa a consulta.
    let out = run_in(&dir, &["--json", "ask", "--params", r#"{"query":"cache"}"#])?;
    assert!(out.status.success(), "ask params falhou: {:?}", out.stderr);
    let value = json(&out)?;
    let hits = value
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .ok_or("sem hits")?;
    assert!(
        hits.iter()
            .any(|hit| hit.get("id").and_then(|i| i.as_str()) == Some(id.as_str())),
        "hit ausente: {hits:?}"
    );

    // `ask -` lê a consulta do stdin.
    let out = run_stdin(&dir, &["--json", "ask", "-"], "cache\n")?;
    assert!(out.status.success(), "ask - falhou: {:?}", out.stderr);

    // `write --params -` lê o objeto do stdin.
    let out = run_stdin(
        &dir,
        &["write", "--params", "-"],
        r#"{"statement":"Via stdin","body":"x","type":"fact"}"#,
    )?;
    assert!(
        out.status.success(),
        "write params - falhou: {:?}",
        out.stderr
    );
    Ok(())
}

#[test]
fn task_new_accepts_anchor_and_rejects_plural_alias() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(
        &dir,
        &[
            "--json",
            "task",
            "new",
            "--summary",
            "Ajustar backoff",
            "--scope",
            "task",
            "--anchor",
            "src/retry.ts",
        ],
    )?;
    assert!(
        out.status.success(),
        "task new --anchor falhou: {:?}",
        out.stderr
    );
    let id = json(&out)?
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("task sem id")?
        .to_string();
    let list = run_in(&dir, &["task", "list", "--anchor", "src/retry.ts"])?;
    let text = String::from_utf8(list.stdout)?;
    assert!(text.contains(&id), "âncora não filtrou {id}: {text}");
    // O plural é alias removido (D168): deve ser erro de uso (2).
    let plural = run_in(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "x",
            "--scope",
            "task",
            "--anchors",
            "src/a.rs",
        ],
    )?;
    assert_eq!(
        plural.status.code(),
        Some(2),
        "alias `--anchors` deveria ser 2"
    );
    Ok(())
}

#[test]
fn config_set_get_roundtrip() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let set = config_set(&dir, "recall.default_limit", "7")?;
    assert!(set.status.success(), "set falhou: {:?}", set.stderr);
    let get = run_in(
        &dir,
        &["--json", "config", "get", "--key", "recall.default_limit"],
    )?;
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
        &[
            "--json",
            "write",
            "--summary",
            "o cache usa body_hash",
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

    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(
        provider.status.success(),
        "config falhou: {:?}",
        provider.stderr
    );
    let drain = run_in(&dir, &["drain", "--digest"])?;
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
    let off = config_set(&dir, "recall.semantic", "false")?;
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
    let out = run_in(&dir, &["--json", "config", "get", "--key", "nao.existe"])?;
    assert_eq!(out.status.code(), Some(3));
    Ok(())
}

#[test]
fn task_plan_prompt_and_submit_from_file() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);

    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Programa", "--scope", "epic"],
    )?;

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
    assert!(tree.contains("|error|"), "sem espécie error: {tree}");
    assert!(tree.contains("|epic|"), "sem espécie epic: {tree}");
    Ok(())
}

#[test]
fn task_plan_invalid_from_writes_nothing() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Programa", "--scope", "epic"],
    )?;
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
fn task_plan_removed_flags_exit_two() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Programa", "--scope", "epic"],
    )?;
    for flag in ["--adopt", "--release", "--review", "--reorder"] {
        let out = run_in(&dir, &["task", "plan", &epic, flag])?;
        assert_eq!(out.status.code(), Some(2), "{flag} deveria ser exit 2");
    }
    Ok(())
}

#[test]
fn task_graph_reports_reduced_mode_without_owner() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let epic = task(&dir, "Épico", &["--scope", "epic"])?;
    let _first = task(&dir, "A", &["--scope", "issue", "--parent", &epic])?;
    let _second = task(&dir, "B", &["--scope", "issue", "--parent", &epic])?;

    let graph = run_in(&dir, &["task", "graph", "--root", &epic])?;
    assert!(graph.status.success());
    let tree = String::from_utf8(graph.stdout)?;
    // Texto enxuto: `id|kind|status|statement`, sem role/mode/owner.
    assert!(!tree.contains("owner"), "coluna de dono: {tree}");
    assert!(
        !tree.contains("supervisor") && !tree.contains("handoff"),
        "modo removido ainda aparece: {tree}"
    );
    let json_out = run_in(&dir, &["--json", "task", "graph", "--root", &epic])?;
    let value = json(&json_out)?;
    let nodes = value
        .get("data")
        .and_then(|data| data.get("nodes"))
        .and_then(|nodes| nodes.as_array())
        .ok_or("sem nodes")?;
    let root = nodes.first().ok_or("sem raiz")?;
    let mode = root
        .get("mode")
        .and_then(|mode| mode.as_str())
        .unwrap_or("");
    assert!(
        matches!(mode, "concurrent" | "magentic" | "sequential"),
        "modo fora do conjunto reduzido: {mode}"
    );
    assert!(
        root.get("role").and_then(|role| role.as_str()).is_some(),
        "role deveria seguir no json"
    );
    assert!(root.get("owner").is_none(), "owner não deveria existir");
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
        let out = run_in(
            &dir,
            &[
                "write",
                "--summary",
                statement,
                "--type",
                "fact",
                "--tag",
                tag,
            ],
        )?;
        assert!(out.status.success(), "write falhou: {:?}", out.stderr);
    }

    let out = run_in(&dir, &["knowledge", "tags"])?;
    assert!(
        out.status.success(),
        "knowledge tags falhou: {:?}",
        out.stderr
    );
    let text = String::from_utf8(out.stdout)?;
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("retry|2"));
    assert_eq!(lines.next(), Some("queue|1"));

    let json_out = run_in(&dir, &["--json", "knowledge", "tags"])?;
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

    let epic = task_new(
        &dir,
        &["task", "new", "--summary", "Épico", "--scope", "epic"],
    )?;
    let story = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Story",
            "--scope",
            "issue",
            "--parent",
            &epic,
        ],
    )?;
    let _ready = task_new(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "Pronta",
            "--scope",
            "task",
            "--parent",
            &story,
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
            "--summary",
            statement,
            "--type",
            note_type,
            "--anchor",
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
            "--summary",
            "implementar retry",
            "--scope",
            "task",
            "--anchor",
            "src/retry.ts",
        ],
    )?;
    let outcome = run_in(&dir, &["write", "--outcome", "success", "--id", &task])?;
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
            "--summary",
            "implementar retry",
            "--scope",
            "task",
            "--anchor",
            "src/retry.ts",
        ],
    )?;
    let outcome = run_in(&dir, &["write", "--outcome", "success", "--id", &task])?;
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

#[test]
fn idle_lazy_drains_queue_after_command() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(
        provider.status.success(),
        "config falhou: {:?}",
        provider.stderr
    );

    let write = run_in(
        &dir,
        &[
            "write",
            "--summary",
            "o auto-drain ocioso indexa sozinho",
            "--type",
            "fact",
        ],
    )?;
    assert!(write.status.success(), "write falhou: {:?}", write.stderr);

    // `drain` é verbo de topo (não drena): reflete o que o auto-drain fez antes de sair.
    let status = run_in(&dir, &["--json", "drain", "--status"])?;
    assert!(
        status.status.success(),
        "status falhou: {:?}",
        status.stderr
    );
    let value = json(&status)?;
    let pending = value
        .get("data")
        .and_then(|data| data.get("pending"))
        .and_then(serde_json::Value::as_u64);
    assert_eq!(pending, Some(0), "fila não drenou: {value}");
    Ok(())
}

#[test]
fn idle_manual_mode_leaves_queue_pending() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(
        provider.status.success(),
        "config falhou: {:?}",
        provider.stderr
    );
    let mode = config_set(&dir, "embeddings.mode", "manual")?;
    assert!(mode.status.success(), "config falhou: {:?}", mode.stderr);

    let write = run_in(
        &dir,
        &[
            "write",
            "--summary",
            "no modo manual nada drena sozinho",
            "--type",
            "fact",
        ],
    )?;
    assert!(write.status.success(), "write falhou: {:?}", write.stderr);

    let status = run_in(&dir, &["--json", "drain", "--status"])?;
    let value = json(&status)?;
    let pending = value
        .get("data")
        .and_then(|data| data.get("pending"))
        .and_then(serde_json::Value::as_u64);
    assert_eq!(pending, Some(1), "modo manual drenou: {value}");
    Ok(())
}

#[test]
fn idle_skips_prime_and_self_verbs() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(
        provider.status.success(),
        "config falhou: {:?}",
        provider.stderr
    );

    // Escreve com o auto-drain desligado: a fila fica `pending` de propósito.
    let write = run_env(
        &dir,
        &[
            "write",
            "--summary",
            "a fila fica pendente de propósito",
            "--type",
            "fact",
        ],
        &[("KNUDGE_NO_IDLE", "1")],
    )?;
    assert!(write.status.success(), "write falhou: {:?}", write.stderr);

    // `prime`/`self version` não drenam a fila (E15-T03/O1.5).
    let prime = run_in(&dir, &["prime"])?;
    assert!(prime.status.success(), "prime falhou: {:?}", prime.stderr);
    let version = run_in(&dir, &["self", "version"])?;
    assert!(
        version.status.success(),
        "self version falhou: {:?}",
        version.stderr
    );

    let status = run_in(&dir, &["--json", "drain", "--status"])?;
    let value = json(&status)?;
    let pending = value
        .get("data")
        .and_then(|data| data.get("pending"))
        .and_then(serde_json::Value::as_u64);
    assert_eq!(pending, Some(1), "prime/self drenaram: {value}");
    Ok(())
}

#[test]
fn drain_without_flags_shows_help_and_does_nothing() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["drain"])?;
    assert!(out.status.success(), "drain: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("Usage: kd drain"), "sem help: {text}");
    assert!(out.stderr.is_empty(), "help não deve ir ao stderr");
    assert!(
        !dir.join(".knudge")
            .join(".idx")
            .join("embeddings.jsonl")
            .exists(),
        "`kd drain` sem flags não pode digerir"
    );
    Ok(())
}

#[test]
fn drain_force_requires_digest() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let out = run_in(&dir, &["drain", "--force"])?;
    assert_eq!(
        out.status.code(),
        Some(2),
        "`--force` sem `--digest` devia dar exit 2"
    );
    Ok(())
}

#[test]
fn drain_status_reports_states_and_recommendation() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(provider.status.success(), "config: {:?}", provider.stderr);
    let write = run_in(
        &dir,
        &["write", "--summary", "nota para o drain", "--type", "fact"],
    )?;
    assert!(write.status.success(), "write: {:?}", write.stderr);
    let out = run_in(&dir, &["--json", "drain", "--status"])?;
    let value = json(&out)?;
    let data = value.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("provider").and_then(serde_json::Value::as_str),
        Some("lightweight")
    );
    for key in ["indexed", "pending", "stale"] {
        assert!(
            data.get(key).and_then(serde_json::Value::as_u64).is_some(),
            "faltou `{key}`: {data}"
        );
    }
    assert!(
        data.get("recommendation")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "sem recomendação: {data}"
    );
    Ok(())
}

#[test]
fn drain_status_explains_when_disabled() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let off = config_set(&dir, "embeddings.enabled", "false")?;
    assert!(off.status.success(), "config: {:?}", off.stderr);
    let out = run_in(&dir, &["--json", "drain", "--status"])?;
    let value = json(&out)?;
    let data = value.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("enabled").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    let recommendation = data
        .get("recommendation")
        .and_then(serde_json::Value::as_str)
        .ok_or("sem recomendação")?;
    assert!(recommendation.contains("desligados"), "{recommendation}");
    Ok(())
}

#[test]
fn drain_digest_force_rebuilds_derived() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let provider = config_set(&dir, "embeddings.provider", "lightweight")?;
    assert!(provider.status.success(), "config: {:?}", provider.stderr);
    let write = run_in(
        &dir,
        &[
            "write",
            "--summary",
            "reconstruir derivado",
            "--type",
            "fact",
        ],
    )?;
    assert!(write.status.success(), "write: {:?}", write.stderr);

    let out = run_in(&dir, &["--json", "drain", "--digest", "--force"])?;
    assert!(out.status.success(), "drain --force: {:?}", out.stderr);
    let value = json(&out)?;
    let data = value.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("rebuilt").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    let removed = data
        .get("removed")
        .and_then(serde_json::Value::as_array)
        .ok_or("sem removed")?;
    assert!(!removed.is_empty(), "nada removido: {value}");

    let out = run_in(&dir, &["--json", "drain", "--status"])?;
    let value = json(&out)?;
    let pending = value
        .get("data")
        .and_then(|data| data.get("pending"))
        .and_then(serde_json::Value::as_u64);
    assert_eq!(pending, Some(0), "fila não redigeriu: {value}");
    Ok(())
}

#[test]
fn eager_mode_is_rejected() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let out = config_set(&dir, "embeddings.mode", "eager")?;
    assert_eq!(
        out.status.code(),
        Some(7),
        "eager deveria ser config inválida"
    );
    Ok(())
}

#[test]
fn watch_service_dry_run_reports_plan() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let out = run_in(
        &dir,
        &[
            "--json",
            "maintenance",
            "watch-service",
            "--install",
            "--dry-run",
        ],
    )?;
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let data = json(&out)?;
    let data = data.get("data").ok_or("sem data")?;
    assert_eq!(
        data.get("dry_run").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        data.get("action").and_then(serde_json::Value::as_str),
        Some("install")
    );
    let reference = data
        .get("reference")
        .and_then(|v| v.as_str())
        .ok_or("sem reference")?;
    assert!(
        reference.contains("knudge-idle.sh"),
        "reference: {reference}"
    );
    let command = data
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("sem command")?;
    assert!(
        command.contains("install") && command.contains("--project"),
        "command: {command}"
    );
    Ok(())
}

#[test]
fn watch_service_defaults_to_status() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let out = run_in(
        &dir,
        &["--json", "maintenance", "watch-service", "--dry-run"],
    )?;
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let data = json(&out)?;
    assert_eq!(
        data.get("data")
            .and_then(|d| d.get("action"))
            .and_then(serde_json::Value::as_str),
        Some("status")
    );
    Ok(())
}

#[test]
fn watch_service_action_flags_are_exclusive() -> TestResult {
    let dir = temp_project();
    let out = run_in(
        &dir,
        &["maintenance", "watch-service", "--install", "--status"],
    )?;
    assert_eq!(
        out.status.code(),
        Some(2),
        "flags de ação conflitantes deveriam ser uso (2)"
    );
    Ok(())
}

#[test]
fn watch_service_declined_does_nothing() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let out = run_stdin(&dir, &["maintenance", "watch-service", "--install"], "n\n")?;
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let text = String::from_utf8(out.stdout)?;
    assert!(text.contains("cancelad"), "saída: {text}");
    Ok(())
}

#[test]
fn watch_service_subscribe_runs_local_script() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "init falhou: {:?}", init.stderr);
    let args_file = dir.join("watch-args.txt");
    let script = dir.join("fake-idle.sh");
    std::fs::write(
        &script,
        format!(
            "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{}'\n",
            args_file.display()
        ),
    )?;
    let script_arg = script.display().to_string();
    let out = run_in(
        &dir,
        &[
            "maintenance",
            "watch-service",
            "--subscribe",
            "--yes",
            "--script",
            &script_arg,
        ],
    )?;
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let recorded = std::fs::read_to_string(&args_file)?;
    assert!(recorded.contains("subscribe"), "args: {recorded}");
    assert!(recorded.contains("--project"), "args: {recorded}");
    Ok(())
}

/// Escreve uma nota com tag e classe e devolve o id.
fn write_tagged(
    dir: &Path,
    statement: &str,
    tag: &str,
    class: &str,
    anchor: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "write",
            "--summary",
            statement,
            "--type",
            "fact",
            "--tag",
            tag,
            "--class",
            class,
            "--anchor",
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

/// Contagem `docs` do `--json knowledge map`.
fn map_docs(value: &serde_json::Value) -> Option<u64> {
    value
        .get("data")
        .and_then(|data| data.get("docs"))
        .and_then(serde_json::Value::as_u64)
}

#[test]
fn d143_knowledge_map_scope_and_filters() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let cache = write_typed(&dir, "cache usa lru", "decision", "src/cache.rs")?;
    let _queue = write_tagged(
        &dir,
        "fila usa backoff",
        "retry",
        "foundational",
        "src/queue.rs",
    )?;

    let bare = run_in(&dir, &["knowledge", "map"])?;
    assert_eq!(bare.status.code(), Some(2), "map sem escopo devia ser 2");

    for filter in [
        vec!["--type", "decision"],
        vec!["--anchor", "src/queue.rs"],
        vec!["--tag", "retry"],
        vec!["--class", "foundational"],
    ] {
        let mut args = vec!["--json", "knowledge", "map"];
        args.extend(filter.iter().copied());
        let out = run_in(&dir, &args)?;
        assert!(out.status.success(), "map {filter:?}: {:?}", out.stderr);
        assert_eq!(
            map_docs(&json(&out)?),
            Some(1),
            "map {filter:?} não restringiu"
        );
    }

    let around = run_in(
        &dir,
        &["knowledge", "map", "--around", &cache, "--depth", "1"],
    )?;
    assert!(around.status.success(), "map --around: {:?}", around.stderr);

    let all = run_in(&dir, &["--json", "knowledge", "map", "--universe"])?;
    assert_eq!(map_docs(&json(&all)?), Some(2), "--universe devia ver tudo");
    Ok(())
}

#[test]
fn d143_rewind_filters_scope_handoff() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let retry = write_tagged(&dir, "usar jitter", "retry", "tactical", "src/a.rs")?;
    let queue = write_tagged(&dir, "usar fila", "queue", "tactical", "src/b.rs")?;

    let scoped = run_in(&dir, &["rewind", "--tag", "retry"])?;
    assert!(scoped.status.success(), "rewind --tag: {:?}", scoped.stderr);
    let text = String::from_utf8(scoped.stdout)?;
    assert!(text.contains(&retry), "nota com a tag ausente: {text}");
    assert!(!text.contains(&queue), "nota fora da tag entrou: {text}");

    let by_files = run_in(&dir, &["rewind", "--files", "src/a.rs", "--tag", "queue"])?;
    let text = String::from_utf8(by_files.stdout)?;
    assert!(
        !text.contains(&retry),
        "filtro não cortou o working set: {text}"
    );
    Ok(())
}

#[test]
fn d144_maintenance_requires_scope() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let _ignored = write_tagged(
        &dir,
        "fila usa backoff",
        "retry",
        "tactical",
        "src/queue.rs",
    )?;

    for cmd in ["learn", "compact", "prune"] {
        let bare = run_in(&dir, &["maintenance", cmd])?;
        assert_eq!(bare.status.code(), Some(2), "maintenance {cmd} sem escopo");
        let universe = run_in(&dir, &["maintenance", cmd, "--universe"])?;
        assert!(
            universe.status.success(),
            "{cmd} --universe: {:?}",
            universe.stderr
        );
    }

    let tagged = run_in(&dir, &["maintenance", "learn", "--tag", "retry"])?;
    assert!(tagged.status.success(), "learn --tag: {:?}", tagged.stderr);
    Ok(())
}

#[test]
fn d144_task_list_requires_scope() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let _ignored = task(&dir, "Alfa", &["--scope", "task"])?;
    let _ignored = task(&dir, "Beta", &["--scope", "task"])?;

    let bare = run_in(&dir, &["task", "list"])?;
    assert_eq!(bare.status.code(), Some(2), "task list sem escopo");
    let sorted = run_in(&dir, &["task", "list", "--sort", "impact"])?;
    assert_eq!(sorted.status.code(), Some(2), "--sort não é escopo");

    let all = run_in(&dir, &["task", "list", "--universe"])?;
    assert!(all.status.success(), "list --universe: {:?}", all.stderr);
    assert_eq!(String::from_utf8(all.stdout)?.lines().count(), 2);

    let by_scope = run_in(&dir, &["task", "list", "--scope", "task"])?;
    assert!(
        by_scope.status.success(),
        "list --scope: {:?}",
        by_scope.stderr
    );
    Ok(())
}

#[test]
fn d146_ask_knowledge_only_and_with_task() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());

    let knowledge = write_anchored(&dir, "cache usa lru", "src/cache.rs")?;
    let task = task(
        &dir,
        "cache invalidation",
        &["--scope", "task", "--anchor", "src/cache.rs"],
    )?;

    let out = run_in(&dir, &["--json", "ask", "cache"])?;
    let hits = json(&out)?
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .cloned()
        .ok_or("sem hits")?;
    let ids: Vec<&str> = hits
        .iter()
        .filter_map(|hit| hit.get("id").and_then(|id| id.as_str()))
        .collect();
    assert!(
        ids.contains(&knowledge.as_str()),
        "conhecimento ausente: {ids:?}"
    );
    assert!(
        !ids.contains(&task.as_str()),
        "tarefa entrou sem --with-task: {ids:?}"
    );

    let with_task = run_in(&dir, &["--json", "ask", "cache", "--with-task"])?;
    let hits = json(&with_task)?
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .cloned()
        .ok_or("sem hits")?;
    assert!(
        hits.iter()
            .any(|hit| hit.get("id").and_then(|id| id.as_str()) == Some(task.as_str())),
        "--with-task não incluiu a tarefa"
    );

    let removed = run_in(&dir, &["ask", "--rank"])?;
    assert_eq!(removed.status.code(), Some(2), "`ask --rank` devia sumir");
    Ok(())
}

#[test]
fn d146_knowledge_rank_and_tags() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let _ignored = write_tagged(
        &dir,
        "fila usa backoff",
        "retry",
        "tactical",
        "src/queue.rs",
    )?;

    let bare = run_in(&dir, &["knowledge", "rank"])?;
    assert_eq!(bare.status.code(), Some(2), "rank sem escopo devia ser 2");

    let ranked = run_in(&dir, &["--json", "knowledge", "rank", "--universe"])?;
    assert!(
        ranked.status.success(),
        "rank --universe: {:?}",
        ranked.stderr
    );
    assert!(
        json(&ranked)?
            .get("data")
            .and_then(|data| data.get("ranked"))
            .is_some(),
        "sem ranked"
    );

    let tags = run_in(&dir, &["knowledge", "tags"])?;
    assert!(tags.status.success(), "knowledge tags: {:?}", tags.stderr);
    assert!(String::from_utf8(tags.stdout)?.contains("retry|1"));
    Ok(())
}

#[test]
fn d151_ask_json_exposes_channel_contributions() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let id = write_anchored(&dir, "cache usa lru", "src/cache.rs")?;

    let out = run_in(&dir, &["--json", "ask", "cache"])?;
    assert!(out.status.success(), "ask: {:?}", out.stderr);
    let hit = json(&out)?
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .and_then(|hits| hits.first())
        .cloned()
        .ok_or("sem hit")?;
    assert_eq!(hit.get("id").and_then(|v| v.as_str()), Some(id.as_str()));
    let channels = hit.get("channels").ok_or("sem channels")?;
    for key in ["lexical", "anchor", "semantic", "recent", "stars"] {
        assert!(
            channels
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .is_some(),
            "canal ausente: {key}"
        );
    }

    // O pipe segue `id|statement|score|why` (sem canais) — D151.
    let pipe = run_in(&dir, &["ask", "cache"])?;
    assert!(pipe.status.success(), "pipe: {:?}", pipe.stderr);
    let text = String::from_utf8(pipe.stdout)?;
    let first = text.lines().next().unwrap_or_default();
    assert_eq!(first.split('|').count(), 4, "pipe mudou: {first}");
    Ok(())
}

#[test]
fn d152_empty_search_prints_no_results() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let _ignored = write_anchored(&dir, "cache usa lru", "src/cache.rs")?;

    let out = run_in(&dir, &["ask", "termo-que-nao-existe-zzz"])?;
    assert!(out.status.success(), "ask vazio: {:?}", out.stderr);
    assert_eq!(String::from_utf8(out.stdout)?.trim_end(), "[no_results]");

    let js = run_in(&dir, &["--json", "ask", "termo-que-nao-existe-zzz"])?;
    assert!(js.status.success(), "ask --json: {:?}", js.stderr);
    let hits = json(&js)?
        .get("data")
        .and_then(|data| data.get("hits"))
        .and_then(|hits| hits.as_array())
        .cloned()
        .ok_or("sem hits")?;
    assert!(hits.is_empty(), "hits devia ser vazio: {hits:?}");
    Ok(())
}

#[test]
fn d145_maintenance_eval_and_index_removed() -> TestResult {
    let dir = temp_project();
    let init = run_in(&dir, &["init", "--no-prompt"])?;
    assert!(init.status.success());
    let removed: [&[&str]; 2] = [&["maintenance", "eval"], &["maintenance", "index"]];
    for args in removed {
        let out = run_in(&dir, args)?;
        assert_eq!(out.status.code(), Some(2), "{args:?} devia sumir");
    }
    let digest = run_in(&dir, &["drain", "--status"])?;
    assert!(digest.status.success(), "drain: {:?}", digest.stderr);
    Ok(())
}
