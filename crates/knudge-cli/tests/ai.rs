//! Testes de integração de `kd ai` e dos filtros de proveniência do `ask` (D218).
//!
//! O contrato: a `Source` JSONL é **read-only**, derivada e determinística; o agente possui o
//! store, o knudge só o indexa. Uma linha malformada é *skip + warning* (D18) — nunca derruba.

mod common;

use std::fs;
use std::path::Path;

use common::{TestResult, init, ok, ok_json, run_in, stderr, temp_project, write_note};
use knudge_core::schema::id;
use knudge_core::store::Note;

/// Converte uma nota do store (`notas/<tipo>/<id>.md`) na linha JSONL do knudge.
fn jsonl_line(dir: &Path, note_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = dir
        .join(".knudge")
        .join("notas")
        .join(id::type_dir(note_id))
        .join(format!("{note_id}.md"));
    let note = Note::parse(&fs::read(&path)?)?;
    Ok(note.to_jsonl()?)
}

/// Escreve uma fonte JSONL sob `.agents/memory/<nome>`.
fn seed_source(dir: &Path, name: &str, lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let memory = dir.join(".agents").join("memory");
    fs::create_dir_all(&memory)?;
    let mut text = String::new();
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    fs::write(memory.join(name), text)?;
    Ok(())
}

#[test]
fn ai_lists_and_recalls_a_jsonl_source() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let ktu = write_note(
        &dir,
        "a memoria do agente vive num jsonl",
        "fact",
        &["--agent", "ktu", "--activity", "write"],
    )?;
    let goose = write_note(
        &dir,
        "adotamos BTreeMap para determinismo",
        "decision",
        &["--agent", "goose", "--activity", "learn"],
    )?;
    seed_source(
        &dir,
        "notes.jsonl",
        &[jsonl_line(&dir, &ktu)?, jsonl_line(&dir, &goose)?],
    )?;

    // Sem query: lista a fonte (`id|tipo|statement`), na ordem determinística.
    let listed = ok(&dir, &["ai", "memory"])?;
    assert!(listed.contains(&ktu), "{listed}");
    assert!(listed.contains(&goose), "{listed}");
    assert!(listed.contains("|fact|"), "{listed}");

    // Com query: recall sobre a fonte.
    let hits = ok(&dir, &["ai", "memory", "jsonl"])?;
    assert!(hits.contains(&ktu), "{hits}");
    assert!(!hits.contains(&goose), "{hits}");

    // Envelope `--json` válido e com a proveniência da área.
    let data = ok_json(&dir, &["ai", "memory", "jsonl"])?;
    assert_eq!(data.get("area").and_then(|v| v.as_str()), Some("memory"));
    assert!(
        data.get("source")
            .and_then(|v| v.as_str())
            .is_some_and(|path| path.ends_with(".agents/memory/notes.jsonl")),
        "{data}"
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_session_log_area_reads_its_own_convention() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let note = write_note(
        &dir,
        "projetamos o log da sessao em notas derivadas",
        "meta",
        &["--agent", "ktu", "--activity", "derive"],
    )?;
    seed_source(&dir, "session-log.jsonl", &[jsonl_line(&dir, &note)?])?;

    let listed = ok(&dir, &["ai", "session-log"])?;
    assert!(listed.contains(&note), "{listed}");
    // A área `memory` (default) não vê o session-log.
    let memory = ok(&dir, &["ai"])?;
    assert!(!memory.contains(&note), "{memory}");
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_source_flag_overrides_the_convention() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let note = write_note(&dir, "fonte explicita fora da convencao", "fact", &[])?;
    let custom = dir.join("outra.jsonl");
    fs::write(&custom, format!("{}\n", jsonl_line(&dir, &note)?))?;

    let listed = ok(
        &dir,
        &["ai", "memory", "--source", custom.to_str().unwrap_or("")],
    )?;
    assert!(listed.contains(&note), "{listed}");
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_tolerates_malformed_lines_with_warning() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let note = write_note(&dir, "linha boa sobrevive a linha ma", "fact", &[])?;
    seed_source(
        &dir,
        "notes.jsonl",
        &["isto nao e json".to_string(), jsonl_line(&dir, &note)?],
    )?;

    let out = run_in(&dir, &["ai", "memory"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    let listed = String::from_utf8(out.stdout.clone())?;
    assert!(listed.contains(&note), "{listed}");
    assert!(
        stderr(&out)?.contains("aviso: linha 1"),
        "{}",
        stderr(&out)?
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_absent_source_is_an_empty_success() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["ai", "memory"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    assert!(
        String::from_utf8(out.stdout)?.trim().is_empty(),
        "a lista vazia nao imprime notas"
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_rejects_an_unknown_area() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["ai", "memoria"])?;
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out)?);
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ask_filters_by_agent_and_activity() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let ktu = write_note(
        &dir,
        "a memoria do agente vive num jsonl",
        "fact",
        &["--agent", "ktu", "--activity", "write"],
    )?;
    let goose = write_note(
        &dir,
        "o formato jsonl tolera linhas malformadas",
        "fact",
        &["--agent", "goose", "--activity", "learn"],
    )?;
    assert_ne!(ktu, goose, "o dedup nao pode fundir estas duas notas");

    let only_ktu = ok(&dir, &["ask", "--agent", "ktu", "jsonl"])?;
    assert!(only_ktu.contains(&ktu), "{only_ktu}");
    assert!(!only_ktu.contains(&goose), "{only_ktu}");

    let only_learn = ok(&dir, &["ask", "--activity", "learn", "jsonl"])?;
    assert!(only_learn.contains(&goose), "{only_learn}");
    assert!(!only_learn.contains(&ktu), "{only_learn}");

    // Vírgula e repetição são equivalentes (D210); `--params` transporta os mesmos filtros.
    let comma = ok(&dir, &["ask", "--agent", "ktu,goose", "jsonl"])?;
    assert!(comma.contains(&ktu) && comma.contains(&goose), "{comma}");
    let params = ok(
        &dir,
        &[
            "ask",
            "--params",
            "{\"query\":\"jsonl\",\"agent\":[\"ktu\"]}",
        ],
    )?;
    assert!(
        params.contains(&ktu) && !params.contains(&goose),
        "{params}"
    );

    // `--rank` aceita os mesmos filtros de proveniência.
    let ranked = ok(
        &dir,
        &[
            "ask",
            "--rank",
            "--universe",
            "--agent",
            "goose",
            "--limit",
            "5",
        ],
    )?;
    assert!(
        ranked.contains(&goose) && !ranked.contains(&ktu),
        "{ranked}"
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}
