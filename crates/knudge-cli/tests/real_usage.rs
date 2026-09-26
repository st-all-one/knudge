//! Uso real de ponta a ponta (v0.3.0, D134–D153): superfície completa num corpus sintético
//! rico — conhecimento, tarefas, grafo, mapa, handoff, manutenção, `--json` e exit codes.

mod common;

use std::path::Path;

use common::{
    TestResult, data, expect_code, init, ok, ok_json, run_in, run_stdin, stderr, temp_project,
    write_note,
};

/// Ids semeados no corpus de conhecimento.
struct Knowledge {
    fact: String,
    decision: String,
    snippet: String,
}

/// Escreve um corpus rico de conhecimento (todas as espécies, tags/âncoras, corpo por stdin).
fn seed_knowledge(dir: &Path) -> Result<Knowledge, Box<dyn std::error::Error>> {
    let fact = write_note(
        dir,
        "O cache usa LRU",
        "fact",
        &[
            "--tag",
            "cache",
            "--anchor",
            "src/cache.rs",
            "--class",
            "tactical",
        ],
    )?;
    let decision = write_note(
        dir,
        "Adotamos RRF para fundir canais",
        "decision",
        &["--tag", "retrieval", "--anchor", "src/retrieval/mod.rs"],
    )?;
    for (summary, kind) in [
        ("Erro de parser com whitespace Unicode", "error"),
        ("Índice pode ficar grande", "risk"),
        ("RRF significa Reciprocal Rank Fusion", "def"),
        ("Precisamos de mais canais de ranking?", "question"),
        ("Documentação do RRF", "link"),
        ("Convenções do projeto", "meta"),
    ] {
        let _ignored = write_note(dir, summary, kind, &[])?;
    }

    let snippet = write_snippet(dir)?;
    write_params(dir)?;
    Ok(Knowledge {
        fact,
        decision,
        snippet,
    })
}

/// Corpo por stdin (D140/D147).
fn write_snippet(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let stdin_note = run_stdin(
        dir,
        &[
            "--json",
            "write",
            "--summary",
            "Snippet de retry",
            "--type",
            "snippet",
        ],
        "```\nretry(3)\n```\n",
    )?;
    assert!(
        stdin_note.status.success(),
        "write stdin: {}",
        stderr(&stdin_note)?
    );
    let snippet = data(&stdin_note)?
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("write stdin sem id")?
        .to_string();
    Ok(snippet)
}

/// `--params` (D147): objeto no schema do lote (`statement`, não `summary`).
fn write_params(dir: &Path) -> TestResult {
    let params = ok_json(
        dir,
        &[
            "write",
            "--params",
            r#"{"statement":"Cache expira em 30 dias","type":"fact","tags":["cache"]}"#,
        ],
    )?;
    assert!(
        params
            .get("items")
            .and_then(|v| v.as_array())
            .and_then(|items| items.first())
            .and_then(|item| item.get("id"))
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "--params não criou a nota"
    );
    Ok(())
}

/// `true` se algum hit tem o id dado.
fn has_hit(value: &serde_json::Value, id: &str) -> bool {
    value
        .get("hits")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .any(|hit| hit.get("id").and_then(serde_json::Value::as_str) == Some(id))
}

#[test]
fn knowledge_write_and_ask() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let seeded = seed_knowledge(&dir)?;

    // Busca: query, filtros, `--brief`, `--full-content`, `--id`.
    let hits = ok_json(&dir, &["ask", "cache"])?;
    assert!(has_hit(&hits, &seeded.fact), "fato ausente");
    assert!(!has_hit(&hits, &seeded.snippet), "sem match entrou");

    let _filtered = ok_json(&dir, &["ask", "cache", "--tag", "cache"])?;
    let brief = ok(&dir, &["ask", "cache", "--brief"])?;
    assert!(
        brief.lines().all(|line| line.split('|').count() == 2),
        "brief: {brief}"
    );
    let body = ok(&dir, &["ask", "--id", &seeded.fact, "--full-content"])?;
    assert!(body.contains(&seeded.fact));

    // Sem resultado → `[no_results]` (D152).
    let none = ok(&dir, &["ask", "termo-que-nao-existe-zzz"])?;
    assert_eq!(none.trim_end(), "[no_results]");

    // Aresta explícita + expand (D126).
    let link = ok(
        &dir,
        &[
            "write",
            "--link",
            &format!("{}:references:{}", seeded.decision, seeded.fact),
        ],
    )?;
    assert!(link.contains("references"), "link: {link}");
    let around = ok(&dir, &["ask", "--around", &seeded.decision, "--depth", "1"])?;
    assert!(around.contains(&seeded.fact), "around: {around}");
    Ok(())
}

#[test]
fn knowledge_map_rank_tags_digest() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _seeded = seed_knowledge(&dir)?;

    let map = ok_json(&dir, &["knowledge", "map", "--universe", "--axis", "type"])?;
    assert!(
        map.get("docs")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|n| n >= 9)
    );
    let _members = ok(
        &dir,
        &[
            "knowledge",
            "map",
            "--universe",
            "--axis",
            "anchor",
            "--members",
        ],
    )?;
    let _write = ok(&dir, &["knowledge", "map", "--universe", "--write"])?;
    assert!(dir.join(".knudge").join("notas").join("MAP.md").exists());

    let ranked = ok_json(&dir, &["knowledge", "rank", "--universe", "--limit", "5"])?;
    assert!(ranked.get("ranked").and_then(|v| v.as_array()).is_some());
    let tags = ok(&dir, &["knowledge", "tags"])?;
    assert!(tags.contains("cache|"), "tags: {tags}");
    let drain = ok(&dir, &["drain", "--status"])?;
    assert!(drain.contains("pending="), "drain status: {drain}");

    // Escopo obrigatório (D143/D144).
    expect_code(&dir, &["knowledge", "map", "--axis", "type"], 2)?;
    expect_code(&dir, &["knowledge", "rank"], 2)?;
    Ok(())
}

#[test]
fn knowledge_forget_restore_purge() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let seeded = seed_knowledge(&dir)?;

    let _forget = ok(&dir, &["forget", "--id", &seeded.fact])?;
    let active = ok_json(&dir, &["ask", "cache"])?;
    assert!(!has_hit(&active, &seeded.fact), "forgotten ainda aparece");

    let _restore = ok(&dir, &["forget", "--id", &seeded.fact, "--restore"])?;
    let restored = ok_json(&dir, &["ask", "cache"])?;
    assert!(
        has_hit(&restored, &seeded.fact),
        "restore não trouxe a nota"
    );

    let _forget = ok(&dir, &["forget", "--id", &seeded.fact])?;
    let _retention = ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "retention.retired_days",
            "--value",
            "0",
        ],
    )?;
    let _purge = ok(&dir, &["forget", "--id", &seeded.fact, "--purge"])?;
    Ok(())
}

#[test]
fn task_create_and_list() -> TestResult {
    let dir = temp_project();
    init(&dir)?;

    let epic = common::task_new(
        &dir,
        "Programa V2",
        &["--scope", "epic", "--anchor", "plan/v2.md"],
    )?;
    let issue = common::task_new(
        &dir,
        "Story do cache",
        &["--scope", "issue", "--parent", &epic],
    )?;
    let base = common::task_new(
        &dir,
        "Base do cache",
        &[
            "--scope", "task", "--parent", &issue, "--checks", "test", "--tag", "cache",
        ],
    )?;
    let dependent = common::task_new(
        &dir,
        "Depende da base",
        &["--scope", "task", "--parent", &issue],
    )?;
    let _edge = ok(
        &dir,
        &["write", "--link", &format!("{dependent}:depends_on:{base}")],
    )?;

    // `--batch` com `key` local para pai/filho (D141) — chaves canônicas (`statement`).
    let batch = "{\"key\":\"p\",\"statement\":\"Pai em lote\",\"scope\":\"issue\"}\n{\"key\":\"c\",\"statement\":\"Filho em lote\",\"scope\":\"task\",\"parent\":\"p\"}\n";
    let out = run_stdin(&dir, &["--json", "task", "new", "--batch", "-"], batch)?;
    assert!(out.status.success(), "batch: {}", stderr(&out)?);
    let items = data(&out)?;
    assert_eq!(
        items.get("items").and_then(|v| v.as_array()).map(Vec::len),
        Some(2)
    );

    let list = ok(&dir, &["task", "list", "--universe"])?;
    assert!(list.contains(&base) && list.contains(&dependent));
    let ready = ok(&dir, &["task", "list", "--ready"])?;
    assert!(ready.contains(&base), "ready: {ready}");
    let blocked = ok(&dir, &["task", "list", "--blocked", "--explain"])?;
    assert!(
        blocked.contains(&dependent) && blocked.contains("blocked_by="),
        "blocked: {blocked}"
    );
    let by_tag = ok(&dir, &["task", "list", "--tag", "cache"])?;
    assert!(by_tag.contains(&base) && !by_tag.contains(&dependent));
    let full = ok(&dir, &["task", "list", "--ready", "--full-content"])?;
    assert!(full.contains("checks:"), "full-content: {full}");

    let show = ok(&dir, &["task", "show", "--id", &issue])?;
    assert!(show.contains(&epic) && show.contains(&base), "show: {show}");
    let graph = ok(&dir, &["task", "graph", "--root", &epic])?;
    assert!(
        graph.contains(&base) && graph.contains("(0/"),
        "graph: {graph}"
    );
    let program = ok(&dir, &["task", "graph", "--program", "plan/v2.md"])?;
    assert!(program.contains(&epic), "program: {program}");

    // Escopo obrigatório (D144).
    expect_code(&dir, &["task", "list"], 2)?;
    Ok(())
}

#[test]
fn task_plan_update_close() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let epic = common::task_new(&dir, "Programa do plano", &["--scope", "epic"])?;
    let task = common::task_new(&dir, "Tarefa do plano", &["--scope", "task"])?;

    // `task new --params` (D147): schema de D141 (`statement`).
    let params = ok_json(
        &dir,
        &[
            "task",
            "new",
            "--params",
            r#"{"statement":"Via params","scope":"task"}"#,
        ],
    )?;
    assert!(params.get("items").and_then(|v| v.as_array()).is_some());

    // Plano: prompt + submit (TOON do template) — D105.
    let prompt = ok(&dir, &["task", "plan", &epic, "--prompt"])?;
    assert!(prompt.contains("template:"), "prompt: {prompt}");
    let plan = "template: feature\nsections:\n  context: ctx\n  approach: app\n  steps:\n    - title: Passo um\n    - title: Passo dois\n  acceptance:\n    - teste passa\n";
    let submitted = run_stdin(
        &dir,
        &["task", "plan", &epic, "--submit", "--from", "-"],
        plan,
    )?;
    assert!(
        submitted.status.success(),
        "plan submit: {}",
        stderr(&submitted)?
    );
    assert_eq!(
        submitted
            .stdout
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .count(),
        2
    );

    // update/close com evidência.
    let updated = ok(
        &dir,
        &["task", "update", "--id", &task, "--status", "in_progress"],
    )?;
    assert!(updated.contains(&task));
    let closed = ok(
        &dir,
        &[
            "task",
            "close",
            "--id",
            &task,
            "--outcome",
            "success",
            "--note",
            "feito",
        ],
    )?;
    assert!(closed.contains(&task));
    Ok(())
}

#[test]
fn maintenance_and_handoff() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let note = write_note(
        &dir,
        "O gateway faz retry",
        "fact",
        &["--anchor", "src/gateway.rs"],
    )?;
    let task = common::task_new(&dir, "Corrigir gateway", &["--scope", "task"])?;

    let doctor = ok(&dir, &["doctor"])?;
    assert!(doctor.contains("schema"));
    let audit = ok(&dir, &["doctor"])?;
    assert!(audit.contains("audit"));

    // Propostas read-only exigem escopo (D144).
    expect_code(&dir, &["maintenance", "learn"], 2)?;
    let _learn = ok(&dir, &["maintenance", "learn", "--universe"])?;
    let _compact = ok(&dir, &["maintenance", "compact", "--universe"])?;
    let _prune = ok(&dir, &["maintenance", "prune", "--universe"])?;

    // Handoff com filtros de corpus (D143).
    let manifest = ok(&dir, &["rewind", "--budget", "800"])?;
    assert!(manifest.contains("notes="), "rewind: {manifest}");
    let files = ok(&dir, &["rewind", "--files", "src/gateway.rs"])?;
    assert!(files.contains(&note), "rewind files: {files}");
    let around = ok(&dir, &["rewind", "--around", &task, "--depth", "1"])?;
    assert!(around.contains("notes="), "rewind around: {around}");

    // Config em dois níveis.
    let _set = ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "recall.default_limit",
            "--value",
            "3",
        ],
    )?;
    let get = ok(&dir, &["config", "get", "--key", "recall.default_limit"])?;
    assert!(get.contains('3'), "config get: {get}");
    let list = ok(&dir, &["config", "list"])?;
    assert!(list.contains("recall.default_limit"), "config list: {list}");
    let _unset = ok(&dir, &["config", "unset", "--key", "recall.default_limit"])?;

    let version = ok_json(&dir, &["self", "version"])?;
    assert!(
        version
            .get("version")
            .and_then(serde_json::Value::as_str)
            .is_some()
    );
    Ok(())
}

#[test]
fn json_envelope_and_exit_codes() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _note = write_note(&dir, "Uma nota qualquer", "fact", &[])?;

    for args in [
        vec!["ask", "nota"],
        vec!["ask", "--id", "fact_zzzzzzzz"],
        vec!["knowledge", "map", "--universe", "--axis", "type"],
        vec!["knowledge", "tags"],
        vec!["task", "list", "--universe"],
        vec!["rewind"],
        vec!["doctor"],
        vec!["config", "list"],
        vec!["prime"],
    ] {
        let mut full = vec!["--json"];
        full.extend_from_slice(&args);
        let out = run_in(&dir, &full)?;
        assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
        let _parsed: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    }

    expect_code(&dir, &["ask", "--around", "fact_zzzzzzzz"], 3)?; // not_found
    expect_code(&dir, &["ask"], 2)?; // uso (invalid_input)
    expect_code(&dir, &["task", "list"], 2)?; // sem escopo
    expect_code(&dir, &["write", "--summary", "x", "--type", "alien"], 8)?; // schema
    expect_code(&dir, &["maintenance", "eval"], 2)?; // removido (D145)
    expect_code(&dir, &["maintenance", "index"], 2)?; // removido (D145)
    Ok(())
}

#[test]
fn universal_input_and_params() -> TestResult {
    let dir = temp_project();
    init(&dir)?;

    // Corpo por heredoc (stdin) em `write` e `task new` (D140/D147).
    let write = run_stdin(
        &dir,
        &[
            "--json",
            "write",
            "--summary",
            "Nota por heredoc",
            "--type",
            "fact",
        ],
        "corpo\ncom duas linhas\n",
    )?;
    assert!(write.status.success(), "write heredoc: {}", stderr(&write)?);
    let id = data(&write)?
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("sem id")?
        .to_string();

    let task = run_stdin(
        &dir,
        &[
            "--json",
            "task",
            "new",
            "--summary",
            "Tarefa por heredoc",
            "--scope",
            "task",
        ],
        "detalhes da tarefa\n",
    )?;
    assert!(task.status.success(), "task heredoc: {}", stderr(&task)?);

    // `ask --params` e `ask -` (query via stdin).
    let hits = ok_json(&dir, &["ask", "--params", r#"{"query":"heredoc"}"#])?;
    assert!(hits.get("hits").and_then(|v| v.as_array()).is_some());
    let piped = run_stdin(&dir, &["--json", "ask", "-"], "heredoc\n")?;
    assert!(piped.status.success(), "ask -: {}", stderr(&piped)?);

    let body = ok(&dir, &["ask", "--id", &id, "--full-content"])?;
    assert!(body.contains("duas linhas"), "corpo ausente: {body}");
    Ok(())
}
