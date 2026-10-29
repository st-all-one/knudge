//! Testes de integração de `kd ai` e dos filtros de proveniência do `ask` (D218).
//!
//! O contrato: a `Source` JSONL é **read-only**, derivada e determinística; o agente possui o
//! store, o knudge só o indexa. Uma linha malformada é *skip + warning* (D18) — nunca derruba.

mod common;

use std::fs;
use std::path::Path;

use common::{TestResult, init, ok, ok_json, run_in, stderr, temp_project, write_note};
use knudge_core::schema::{AgentRole, Provenance, Value, id, provenance};
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

/// Escreve um shard NDJSON sob `.knudge/ai/<area>/<shard>.jsonl` (D220).
fn seed_source(
    dir: &Path,
    area: &str,
    shard: &str,
    lines: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let memory = dir.join(".knudge").join("ai").join(area);
    fs::create_dir_all(&memory)?;
    let mut text = String::new();
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    fs::write(memory.join(format!("{shard}.jsonl")), text)?;
    Ok(())
}

/// Notas do envelope `kd ai --json`.
fn notes_of(
    data: &serde_json::Value,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    data.get("data")
        .and_then(|envelope| envelope.get("notes"))
        .and_then(|notes| notes.as_array())
        .cloned()
        .ok_or_else(|| "notes ausente".into())
}

/// Hits do envelope `kd ai --json`.
fn hits_of(data: &serde_json::Value) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    data.get("data")
        .and_then(|envelope| envelope.get("hits"))
        .and_then(|hits| hits.as_array())
        .cloned()
        .ok_or_else(|| "hits ausente".into())
}

/// Ajusta identidade/`created_at` de uma linha JSONL de nota (para testar D221/D222).
fn identify_line(
    line: &str,
    agent_id: &str,
    role: AgentRole,
    parent: Option<&str>,
    created_ms: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut note = Note::from_jsonl(line)?;
    note.frontmatter.set("created_at", Value::Int(created_ms))?;
    let prov = Provenance {
        agent_id: Some(agent_id.to_string()),
        agent_role: Some(role),
        parent_agent_id: parent.map(str::to_string),
        ..provenance(&note.frontmatter)?
    };
    note.frontmatter.set("provenance", prov.to_value())?;
    Ok(note.to_jsonl()?)
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
        "memory",
        "notes",
        &[jsonl_line(&dir, &ktu)?, jsonl_line(&dir, &goose)?],
    )?;

    // Sem query: lista a fonte (`id|tipo|statement`), na ordem determinística.
    let listed = ok(&dir, &["ai", "--memory"])?;
    assert!(listed.contains(&ktu), "{listed}");
    assert!(listed.contains(&goose), "{listed}");
    assert!(listed.contains("|fact|"), "{listed}");

    // Com query: recall sobre a fonte.
    let hits = ok(&dir, &["ai", "--memory", "jsonl"])?;
    assert!(hits.contains(&ktu), "{hits}");
    assert!(!hits.contains(&goose), "{hits}");

    // Envelope `--json` válido e com a proveniência da área.
    let data = ok_json(&dir, &["ai", "--memory", "jsonl"])?;
    assert_eq!(data.get("area").and_then(|v| v.as_str()), Some("memory"));
    assert!(
        data.get("source")
            .and_then(|v| v.as_str())
            .is_some_and(|path| path.ends_with(".knudge/ai/memory")),
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
    seed_source(&dir, "session-log", "log", &[jsonl_line(&dir, &note)?])?;

    let listed = ok(&dir, &["ai", "--session"])?;
    assert!(listed.contains(&note), "{listed}");
    // A área `memory` (default) não vê o session-log.
    seed_source(
        &dir,
        "memory",
        "main",
        &[jsonl_line(
            &dir,
            &write_note(&dir, "a memoria nao ve o log", "fact", &[])?,
        )?],
    )?;
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
        &["ai", "--memory", "--source", custom.to_str().unwrap_or("")],
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
        "memory",
        "notes",
        &["isto nao e json".to_string(), jsonl_line(&dir, &note)?],
    )?;

    let out = run_in(&dir, &["ai", "--memory"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    let listed = String::from_utf8(out.stdout.clone())?;
    assert!(listed.contains(&note), "{listed}");
    assert!(
        stderr(&out)?.contains("linha 1 ignorada"),
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
    let out = run_in(&dir, &["ai", "--memory"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    assert!(
        String::from_utf8(out.stdout)?.trim().is_empty(),
        "a lista vazia nao imprime notas"
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

// (removido: área desconhecida agora exibe help com exit 0)

#[test]
fn ai_exposes_identity_and_sorts_by_time() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let early = write_note(&dir, "primeira nota temporal", "fact", &[])?;
    let late = write_note(&dir, "segunda nota temporal", "fact", &[])?;
    let early_line = identify_line(
        &jsonl_line(&dir, &early)?,
        "omp:sub",
        AgentRole::Subagent,
        Some("omp:main"),
        1_000,
    )?;
    let late_line = identify_line(
        &jsonl_line(&dir, &late)?,
        "omp:main",
        AgentRole::Main,
        None,
        2_000,
    )?;
    seed_source(&dir, "memory", "a-sub", &[early_line])?;
    seed_source(&dir, "memory", "b-main", &[late_line])?;

    let data = ok_json(&dir, &["ai", "--memory"])?;
    let notes = notes_of(&data)?;
    assert_eq!(notes.len(), 2, "{data}");
    let sub = notes
        .iter()
        .find(|note| note.get("id").and_then(serde_json::Value::as_str) == Some(early.as_str()))
        .ok_or("nota early ausente")?;
    assert_eq!(
        sub.get("agent_id").and_then(serde_json::Value::as_str),
        Some("omp:sub")
    );
    assert_eq!(
        sub.get("agent_role").and_then(serde_json::Value::as_str),
        Some("subagent")
    );
    assert_eq!(
        sub.get("parent_agent_id")
            .and_then(serde_json::Value::as_str),
        Some("omp:main")
    );
    assert_eq!(
        sub.get("created_at").and_then(serde_json::Value::as_i64),
        Some(1_000)
    );

    let timed = ok_json(&dir, &["ai", "--memory", "--sort", "time"])?;
    let timed_notes = notes_of(&timed)?;
    let ids: Vec<&str> = timed_notes
        .iter()
        .filter_map(|note| note.get("id").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(ids, vec![early.as_str(), late.as_str()], "{timed}");
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_filters_identity_and_reads_params() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let early = write_note(&dir, "primeira nota filtrada", "fact", &[])?;
    let late = write_note(&dir, "segunda nota filtrada", "fact", &[])?;
    let early_line = identify_line(
        &jsonl_line(&dir, &early)?,
        "omp:sub",
        AgentRole::Subagent,
        Some("omp:main"),
        1_000,
    )?;
    let late_line = identify_line(
        &jsonl_line(&dir, &late)?,
        "omp:main",
        AgentRole::Main,
        None,
        2_000,
    )?;
    seed_source(&dir, "memory", "a-sub", &[early_line])?;
    seed_source(&dir, "memory", "b-main", &[late_line])?;

    let only_sub = ok(&dir, &["ai", "--memory", "--agent-id", "omp:sub"])?;
    assert!(
        only_sub.contains(&early) && !only_sub.contains(&late),
        "{only_sub}"
    );
    let only_main = ok(&dir, &["ai", "--memory", "--agent-role", "main"])?;
    assert!(
        only_main.contains(&late) && !only_main.contains(&early),
        "{only_main}"
    );
    let params = ok(
        &dir,
        &[
            "ai",
            "--memory",
            "--params",
            "{\"agent_id\":[\"omp:sub\"],\"sort\":\"time\"}",
        ],
    )?;
    assert!(
        params.contains(&early) && !params.contains(&late),
        "{params}"
    );
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_recall_json_includes_created_at_and_identity() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let note = write_note(&dir, "o recall expõe a identidade", "fact", &[])?;
    let line = identify_line(
        &jsonl_line(&dir, &note)?,
        "omp:main",
        AgentRole::Main,
        None,
        4_242,
    )?;
    seed_source(&dir, "memory", "main", &[line])?;

    let data = ok_json(&dir, &["ai", "--memory", "identidade"])?;
    let hits = hits_of(&data)?;
    let hit = hits.first().ok_or("hit ausente")?;
    assert_eq!(
        hit.get("id").and_then(serde_json::Value::as_str),
        Some(note.as_str())
    );
    assert_eq!(
        hit.get("created_at").and_then(serde_json::Value::as_i64),
        Some(4_242)
    );
    assert_eq!(
        hit.get("agent_id").and_then(serde_json::Value::as_str),
        Some("omp:main")
    );
    assert_eq!(
        hit.get("agent_role").and_then(serde_json::Value::as_str),
        Some("main")
    );
    assert_eq!(hit.get("parent_agent_id"), Some(&serde_json::Value::Null));
    let _ignored = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn ai_multiple_shards_merge_deterministically() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let first = write_note(&dir, "nota do shard a", "fact", &[])?;
    let second = write_note(&dir, "nota do shard b", "fact", &[])?;
    seed_source(&dir, "memory", "a", &[jsonl_line(&dir, &first)?])?;
    seed_source(&dir, "memory", "b", &[jsonl_line(&dir, &second)?])?;

    let listed = ok(&dir, &["ai", "--memory"])?;
    assert!(
        listed.contains(&first) && listed.contains(&second),
        "{listed}"
    );
    let lines: Vec<&str> = listed.lines().collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted, "ordem determinística por id: {listed}");
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
