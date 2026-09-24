//! Integração v0.3.2: `ask --as-of`, `knowledge suggest`/`promote` e o portão de evidência
//! (D154–D159).

mod common;

use std::path::Path;

use common::{
    TestResult, expect_code, init, ok, ok_json, run_in, stdout, temp_project, write_note,
};

/// Lê o `created_at` do arquivo da nota `id` (frontmatter TOON).
fn created_at(dir: &Path, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = dir
        .join(".knudge")
        .join("notas")
        .join("fact")
        .join(format!("{id}.md"));
    let text = std::fs::read_to_string(&path)?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("created_at:") {
            return Ok(rest.trim().to_string());
        }
    }
    Err(format!("created_at ausente em {}", path.display()).into())
}

#[test]
fn ask_as_of_finds_note_at_creation_and_rejects_future() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = write_note(&dir, "o cache usa body_hash", "fact", &[])?;
    let at = created_at(&dir, &id)?;

    let at_creation = run_in(&dir, &["ask", "cache", "--as-of", at.as_str()])?;
    assert!(
        at_creation.status.success(),
        "as-of falhou: {:?}",
        at_creation.stderr
    );
    let text = stdout(&at_creation)?;
    assert!(text.contains(&id), "as-of não recuperou {id}: {text}");
    assert!(text.contains("as_of="), "banner ausente: {text}");

    let epoch = run_in(
        &dir,
        &["ask", "cache", "--as-of", "1970-01-01T00:00:00.000Z"],
    )?;
    assert!(
        epoch.status.success(),
        "as-of época falhou: {:?}",
        epoch.stderr
    );
    assert!(stdout(&epoch)?.contains("[no_results]"));

    expect_code(
        &dir,
        &["ask", "cache", "--as-of", "2999-01-01T00:00:00.000Z"],
        2,
    )?;
    Ok(())
}

#[test]
fn knowledge_suggest_without_embeddings_is_no_results() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["knowledge", "suggest"])?;
    assert!(out.status.success(), "suggest falhou: {:?}", out.stderr);
    assert_eq!(stdout(&out)?.trim(), "[no_results]");
    Ok(())
}

#[test]
fn knowledge_promote_is_disabled_by_default() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    expect_code(&dir, &["knowledge", "promote", "recommend"], 2)?;
    Ok(())
}

/// Habilita a promoção e cria uma nota `meta`/`foundational` amplamente ancorada.
fn promoted_candidate(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    ok(
        dir,
        &["config", "set", "--key", "rules.enabled", "--value", "true"],
    )?;
    ok(
        dir,
        &[
            "config",
            "set",
            "--key",
            "rules.min_confidence",
            "--value",
            "0.5",
        ],
    )?;
    write_note(
        dir,
        "sempre rodar make check antes do commit",
        "meta",
        &[
            "--class",
            "foundational",
            "--anchor",
            "src/lib.rs",
            "--anchor",
            "src/main.rs",
            "--anchor",
            "Cargo.toml",
        ],
    )
}

#[test]
fn knowledge_promote_approve_writes_governed_block() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = promoted_candidate(&dir)?;

    let recommend = run_in(&dir, &["knowledge", "promote", "recommend", "--universe"])?;
    assert!(
        recommend.status.success(),
        "recommend falhou: {:?}",
        recommend.stderr
    );
    assert!(stdout(&recommend)?.contains(&id));

    let approve = run_in(
        &dir,
        &["knowledge", "promote", "approve", id.as_str(), "--universe"],
    )?;
    assert!(
        approve.status.success(),
        "approve falhou: {:?}",
        approve.stderr
    );
    let agents = std::fs::read_to_string(dir.join("AGENTS.md"))?;
    assert!(agents.contains("knudge:rules:start"), "bloco ausente");
    assert!(agents.contains(&id), "proveniência ausente");

    let list = ok_json(&dir, &["knowledge", "promote", "list"])?;
    assert_eq!(
        list.get("count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let remove = run_in(
        &dir,
        &["knowledge", "promote", "remove", id.as_str(), "--universe"],
    )?;
    assert!(
        remove.status.success(),
        "remove falhou: {:?}",
        remove.stderr
    );
    let agents = std::fs::read_to_string(dir.join("AGENTS.md"))?;
    assert!(!agents.contains(&id), "linha não removida");
    Ok(())
}

#[test]
fn write_gate_enforce_blocks_creation() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let catalog = dir.join(".knudge").join("validators.toml");
    std::fs::write(&catalog, "[deny]\ncmd = \"false\"\nkind = \"gate\"\n")?;
    ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "proposals.gate",
            "--value",
            "deny",
        ],
    )?;
    ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "proposals.enforce",
            "--value",
            "true",
        ],
    )?;
    expect_code(
        &dir,
        &["write", "--summary", "nota bloqueada", "--type", "fact"],
        4,
    )?;
    Ok(())
}

#[test]
fn learn_verify_is_read_only() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let catalog = dir.join(".knudge").join("validators.toml");
    std::fs::write(&catalog, "[ok]\ncmd = \"true\"\nkind = \"gate\"\n")?;
    ok(
        &dir,
        &["config", "set", "--key", "proposals.gate", "--value", "ok"],
    )?;
    let verify = run_in(&dir, &["maintenance", "learn", "--universe", "--verify"])?;
    assert!(
        verify.status.success(),
        "verify falhou: {:?}",
        verify.stderr
    );
    Ok(())
}
