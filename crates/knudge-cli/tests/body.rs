//! Revelação progressiva do corpo no `ask` (D161): top-1 completo, 2–5 truncado, resto padrão.

mod common;

use common::{TestResult, expect_code, init, ok, ok_json, temp_project, write_note};

#[test]
fn ask_shows_body_of_top_hit_and_reports_match() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    write_note(
        &dir,
        "cache usa LRU",
        "decision",
        &["Por quê: reduz latência.", "--anchor", "src/cache.rs"],
    )?;

    let text = ok(&dir, &["ask", "lat"])?;
    assert!(text.contains("Por quê"), "corpo do top hit ausente: {text}");

    let data = ok_json(&dir, &["ask", "lat"])?;
    let hit = data
        .get("hits")
        .and_then(|hits| hits.as_array())
        .and_then(|hits| hits.first());
    let Some(hit) = hit else {
        return Err("sem hits no envelope".into());
    };
    assert_eq!(
        hit.get("body_match").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        hit.get("body_snippet")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|snippet| snippet.contains("lat")),
        "snippet ausente: {hit}"
    );
    assert!(
        hit.get("channels")
            .and_then(|channels| channels.get("body"))
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|share| share > 0.0),
        "parcela do body ausente: {hit}"
    );
    Ok(())
}

#[test]
fn ask_truncates_body_after_first_hit() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "recall.preview_chars",
            "--value",
            "5",
        ],
    )?;
    write_note(
        &dir,
        "cache usa LRU",
        "fact",
        &["corpo bem longo para o cache usa LRU"],
    )?;
    write_note(
        &dir,
        "fila usa redis",
        "fact",
        &["corpo bem longo para a fila usa redis"],
    )?;

    let text = ok(&dir, &["ask", "usa"])?;
    assert!(text.contains('…'), "corpo parcial sem marcador: {text}");
    Ok(())
}

#[cfg(unix)]
fn install_body_gate(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let script = dir.join(".knudge").join("gate_body.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\ninput=$(cat)\n\
         echo \"$input\" | grep -q '\"type\":\"decision\"' || { echo '{\"passed\":true,\"score_before\":0,\"score_after\":1}'; exit 0; }\n\
         echo \"$input\" | grep -q '\"body\":\"\"' && { echo '{\"passed\":false,\"score_before\":0,\"score_after\":0}'; exit 0; }\n\
         echo '{\"passed\":true,\"score_before\":0,\"score_after\":1}'\n",
    )?;
    let mut perms = std::fs::metadata(&script)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms)?;

    let catalog = dir.join(".knudge").join("validators.toml");
    std::fs::write(
        &catalog,
        format!("[body]\ncmd = \"{}\"\nkind = \"gate\"\n", script.display()),
    )?;
    ok(
        dir,
        &[
            "config",
            "set",
            "--key",
            "proposals.gate",
            "--value",
            "body",
        ],
    )?;
    ok(
        dir,
        &[
            "config",
            "set",
            "--key",
            "proposals.enforce",
            "--value",
            "true",
        ],
    )?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn body_gate_blocks_decision_without_body() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    install_body_gate(&dir)?;

    expect_code(
        &dir,
        &[
            "write",
            "--summary",
            "decisão sem corpo",
            "--type",
            "decision",
        ],
        4,
    )?;
    ok(
        &dir,
        &[
            "write",
            "--summary",
            "decisão com corpo",
            "--type",
            "decision",
            "Por quê: x.",
        ],
    )?;
    Ok(())
}
