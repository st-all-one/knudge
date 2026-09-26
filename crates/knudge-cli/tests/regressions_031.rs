//! Regressões da v0.3.1: âncoras ausentes/diretório, índice de épico, `--update --params`,
//! `--clear-anchors`, purga forçada, `--scope` por id e `program-anchor` advisório.

mod common;

use std::path::Path;

use common::{
    TestResult, expect_code, init, ok_json, run_in, stderr, stdout, temp_project, write_note,
};
use serde_json::Value;

/// Id da nota + caminho do arquivo em `notas/`.
type Anchored = Result<(String, std::path::PathBuf), Box<dyn std::error::Error>>;

/// Cria a nota e o arquivo/âncora; devolve `(id, caminho_da_nota)`.
fn anchored_note(dir: &Path, anchor: &str) -> Anchored {
    let id = write_note(dir, "nota ancorada", "fact", &["--anchor", anchor])?;
    let path = dir
        .join(".knudge")
        .join("notas")
        .join("fact")
        .join(format!("{id}.md"));
    Ok((id, path))
}

/// Lê do `report` o campo booleano `key` do check `id`.
fn check_bool(report: &Value, id: &str, key: &str) -> Option<bool> {
    report
        .get("checks")
        .and_then(Value::as_array)
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check.get("id").and_then(Value::as_str) == Some(id))
        })
        .and_then(|check| check.get(key))
        .and_then(Value::as_bool)
}

#[test]
fn missing_anchor_does_not_abort_doctor() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src").join("a.txt"), "x")?;
    let (_id, path) = anchored_note(&dir, "src/a.txt")?;
    let _fixed = ok_json(&dir, &["doctor", "--fix"])?;

    std::fs::remove_file(dir.join("src").join("a.txt"))?;

    let report = ok_json(&dir, &["doctor"])?;
    let anchors_ok = check_bool(&report, "anchors", "ok");
    assert_eq!(
        anchors_ok,
        Some(false),
        "âncora ausente deve virar `anchors: fail`, não erro de I/O"
    );
    // A nota continua legível.
    let body = std::fs::read_to_string(&path)?;
    assert!(body.contains("nota ancorada"));
    Ok(())
}

#[test]
fn directory_anchor_is_dropped_by_fix() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    std::fs::create_dir_all(dir.join("src").join("dir"))?;
    let (_id, path) = anchored_note(&dir, "src/dir")?;
    let _fixed = ok_json(&dir, &["doctor", "--fix"])?;
    let body = std::fs::read_to_string(&path)?;
    assert!(
        !body.contains("anchors:"),
        "âncora de diretório deve sair: {body}"
    );
    Ok(())
}

#[test]
fn epic_derived_index_is_coherent() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _epic = common::task_new(&dir, "Épico E", &["--scope", "epic"])?;
    let _fixed = ok_json(&dir, &["doctor", "--fix"])?;
    let report = ok_json(&dir, &["doctor"])?;
    assert_eq!(report.get("healthy").and_then(Value::as_bool), Some(true));
    let derived_ok = check_bool(&report, "derived", "ok");
    assert_eq!(derived_ok, Some(true));
    Ok(())
}

#[test]
fn update_params_body_revises_in_place() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = write_note(&dir, "original", "fact", &[])?;
    let data = ok_json(
        &dir,
        &[
            "write",
            "--update",
            &id,
            "--params",
            "{\"body\":\"novo corpo\"}",
        ],
    )?;
    assert_eq!(
        data.get("action").and_then(|value| value.as_str()),
        Some("updated")
    );
    assert_eq!(
        data.get("id").and_then(|value| value.as_str()),
        Some(id.as_str())
    );
    Ok(())
}

#[test]
fn clear_anchors_removes_them() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src").join("a.rs"), "x")?;
    let (id, path) = anchored_note(&dir, "src/a.rs")?;
    let _data = ok_json(&dir, &["write", "--update", &id, "--clear-anchors"])?;
    let body = std::fs::read_to_string(&path)?;
    assert!(!body.contains("anchors:"), "âncoras deveriam sumir: {body}");
    Ok(())
}

#[test]
fn empty_anchor_is_rejected() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    expect_code(
        &dir,
        &[
            "write",
            "--summary",
            "vazia",
            "--type",
            "fact",
            "--anchor",
            "",
        ],
        2,
    )
}

#[test]
fn purge_force_releases_superseded_tombstone() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let old = write_note(&dir, "original", "fact", &[])?;
    let new = ok_json(
        &dir,
        &[
            "write",
            "--update",
            &old,
            "--params",
            "{\"statement\":\"renomeada\"}",
        ],
    )?
    .get("id")
    .and_then(|value| value.as_str())
    .ok_or("supersede sem id novo")?
    .to_string();
    assert_ne!(old, new);

    expect_code(&dir, &["forget", "--purge", "--id", &old], 2)?;
    let purged = ok_json(&dir, &["forget", "--purge", "--force", "--id", &old])?;
    assert_eq!(
        purged.get("action").and_then(|value| value.as_str()),
        Some("purge")
    );
    assert!(
        !dir.join(".knudge")
            .join("notas")
            .join("fact")
            .join(format!("{old}.md"))
            .exists()
    );
    Ok(())
}

#[test]
fn task_list_scope_accepts_container_id() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let epic = common::task_new(&dir, "Épico", &["--scope", "epic"])?;
    let issue = common::task_new(&dir, "Issue", &["--scope", "issue", "--parent", &epic])?;
    let leaf = common::task_new(&dir, "Folha", &["--scope", "task", "--parent", &issue])?;

    let by_level = stdout(&run_in(&dir, &["task", "list", "--scope", "epic"])?)?;
    assert!(by_level.contains(&epic), "nível epic: {by_level}");

    let by_id = stdout(&run_in(&dir, &["task", "list", "--scope", &epic])?)?;
    assert!(
        by_id.contains(&issue),
        "container id deve trazer filhos: {by_id}"
    );
    assert!(
        by_id.contains(&leaf),
        "container id deve trazer netos: {by_id}"
    );
    Ok(())
}

#[test]
fn task_update_replaces_and_clears_anchors() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src").join("a.rs"), "x")?;
    std::fs::write(dir.join("src").join("b.rs"), "x")?;
    let id = common::task_new(
        &dir,
        "Tarefa ancorada",
        &[
            "--scope", "task", "--anchor", "src/a.rs", "--anchor", "src/b.rs",
        ],
    )?;
    let path = dir
        .join(".knudge")
        .join("notas")
        .join("task")
        .join(format!("{id}.md"));

    let _replaced = ok_json(
        &dir,
        &["task", "update", "--id", &id, "--anchor", "src/a.rs"],
    )?;
    let body = std::fs::read_to_string(&path)?;
    assert!(
        body.contains("src/a.rs") && !body.contains("src/b.rs"),
        "replace: {body}"
    );

    let _cleared = ok_json(&dir, &["task", "update", "--id", &id, "--clear-anchors"])?;
    let body = std::fs::read_to_string(&path)?;
    assert!(!body.contains("anchors:"), "clear: {body}");

    // Âncora vazia em `task new` também é rejeitada (mesmo `validate_anchors`).
    expect_code(
        &dir,
        &[
            "task",
            "new",
            "--summary",
            "vazia",
            "--scope",
            "task",
            "--anchor",
            "",
        ],
        2,
    )?;
    Ok(())
}

#[test]
fn program_anchor_warn_keeps_corpus_healthy() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _epic = common::task_new(&dir, "Épico sem plano", &["--scope", "epic"])?;
    let _fixed = ok_json(&dir, &["doctor", "--fix"])?;
    let out = run_in(&dir, &["--json", "doctor"])?;
    assert!(out.status.success(), "doctor: {}", stderr(&out)?);
    let report = common::data(&out)?;
    assert_eq!(report.get("healthy").and_then(Value::as_bool), Some(true));
    let program_warn = check_bool(&report, "program-anchor", "warn");
    assert_eq!(program_warn, Some(true));
    Ok(())
}

#[test]
fn audit_json_exposes_duplicate_pairs() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _note = write_note(&dir, "alpha beta gamma", "fact", &[])?;
    let report = ok_json(&dir, &["doctor"])?;
    let audit = report.get("audit").ok_or("doctor sem bloco audit")?;
    assert!(
        audit.get("duplicate_pairs").is_some(),
        "audit --json precisa expor os pares: {report}"
    );
    assert!(audit.get("broken_anchor_details").is_some());
    Ok(())
}
