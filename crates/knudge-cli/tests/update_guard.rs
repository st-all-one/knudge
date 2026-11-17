//! Guarda de retry em `kd write --update` (D228): flag, chave JSON e exit codes.

use std::path::Path;

mod common;

use common::{TestResult, init, run_in, stderr, temp_project, write_note};

/// Lê o `body_hash` do frontmatter da nota gravada.
fn note_body_hash(dir: &Path, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = dir
        .join(".knudge")
        .join("notas")
        .join("fact")
        .join(format!("{id}.md"));
    let text = std::fs::read_to_string(&path)?;
    let hash = text
        .lines()
        .find_map(|line| line.strip_prefix("body_hash: "))
        .ok_or("nota sem body_hash")?
        .to_string();
    Ok(hash)
}

#[test]
fn expect_body_hash_sem_update_eh_invalid_input() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(
        &dir,
        &[
            "write",
            "--summary",
            "nota isolada",
            "--expect-body-hash",
            "00000000",
        ],
    )?;
    assert_eq!(
        out.status.code(),
        Some(2),
        "flag sem --update devia ser invalid_input: {}",
        stderr(&out)?
    );
    Ok(())
}

#[test]
fn expect_body_hash_divergente_sai_com_conflito_e_dois_hashes() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = write_note(&dir, "nota da guarda", "fact", &[])?;
    let atual = note_body_hash(&dir, &id)?;
    let esperado = "00000000";

    let out = run_in(
        &dir,
        &[
            "write",
            "--update",
            &id,
            "--expect-body-hash",
            esperado,
            "--tag",
            "alterada",
        ],
    )?;
    assert_eq!(
        out.status.code(),
        Some(4),
        "divergência devia ser conflict: {}",
        stderr(&out)?
    );
    let message = stderr(&out)?;
    assert!(
        message.contains(esperado),
        "mensagem sem o hash esperado: {message}"
    );
    assert!(
        message.contains(&atual),
        "mensagem sem o hash atual: {message}"
    );
    assert_eq!(
        note_body_hash(&dir, &id)?,
        atual,
        "guarda divergente alterou a nota"
    );
    Ok(())
}

#[test]
fn expect_body_hash_igual_aplica_update() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = write_note(&dir, "nota da guarda", "fact", &[])?;
    let atual = note_body_hash(&dir, &id)?;

    let out = run_in(
        &dir,
        &[
            "write",
            "--update",
            &id,
            "--expect-body-hash",
            &atual,
            "--tag",
            "alterada",
        ],
    )?;
    assert!(
        out.status.success(),
        "hash igual devia aplicar: {}",
        stderr(&out)?
    );
    Ok(())
}

#[test]
fn expect_body_hash_por_params_json_key_tambem_guarda() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let id = write_note(&dir, "nota da guarda", "fact", &[])?;
    let atual = note_body_hash(&dir, &id)?;
    let params = format!("{{\"expect_body_hash\":\"{atual}\",\"tags\":[\"alterada\"]}}");

    let out = run_in(&dir, &["write", "--update", &id, "--params", &params])?;
    assert!(
        out.status.success(),
        "chave JSON com hash igual devia aplicar: {}",
        stderr(&out)?
    );

    let divergente = "{\"expect_body_hash\":\"00000000\",\"tags\":[\"outra\"]}";
    let out = run_in(&dir, &["write", "--update", &id, "--params", divergente])?;
    assert_eq!(
        out.status.code(),
        Some(4),
        "chave JSON divergente devia ser conflict: {}",
        stderr(&out)?
    );
    let message = stderr(&out)?;
    assert!(message.contains("00000000"), "sem esperado: {message}");
    assert!(message.contains(&atual), "sem atual: {message}");
    Ok(())
}
