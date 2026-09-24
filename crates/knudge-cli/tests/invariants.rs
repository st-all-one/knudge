//! Invariantes transversais R45 (D154–D159) exercitados de ponta a ponta no binário `kd`.
//!
//! Não testam uma feature nova, e sim que as propriedades valem para os verbos de **leitura**
//! já existentes: leitura não escreve nota nem derivado por default, renovação por uso só
//! estende, e supersessão vence uso.

mod common;

use std::path::{Path, PathBuf};

use common::{TestResult, init, ok, temp_project, write_note};

/// Impressão digital estável do conteúdo de `.knudge/notas/` (caminho relativo + bytes).
fn notas_fingerprint(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let root = dir.join(".knudge").join("notas");
    let mut stack = vec![root.clone()];
    let mut files: Vec<(String, String)> = Vec::new();
    while let Some(path) = stack.pop() {
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            for entry in std::fs::read_dir(&path)? {
                stack.push(entry?.path());
            }
            continue;
        }
        let rel = path.strip_prefix(&root)?.to_string_lossy().to_string();
        let content = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
        files.push((rel, content));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut out = String::new();
    for (rel, content) in files {
        out.push_str(&rel);
        out.push('\u{1f}');
        out.push_str(&content);
        out.push('\n');
    }
    Ok(out)
}

/// Caminho do derivado de uso (`.idx/usage.jsonl`).
fn usage_path(dir: &Path) -> PathBuf {
    dir.join(".knudge").join(".idx").join("usage.jsonl")
}

/// Com config default, os verbos de leitura não tocam `notas/` **nem** criam `.idx/usage.jsonl`.
#[test]
fn default_reads_leave_notes_and_derived_untouched() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    write_note(&dir, "nota alfa", "fact", &[])?;
    write_note(&dir, "nota beta", "decision", &[])?;
    let before = notas_fingerprint(&dir)?;

    ok(&dir, &["ask", "nota"])?;
    ok(&dir, &["rewind"])?;
    ok(&dir, &["knowledge", "rank", "--universe"])?;
    ok(&dir, &["knowledge", "tags"])?;
    ok(&dir, &["knowledge", "suggest"])?;

    assert_eq!(notas_fingerprint(&dir)?, before, "leitura alterou notas/");
    assert!(
        !usage_path(&dir).exists(),
        "uso derivado criado com config default (R45-1)"
    );
    Ok(())
}

/// Com `renew_on_use=true` o uso é creditado, mas a leitura continua sem escrever nota (R45-7).
#[test]
fn renewal_credits_usage_but_never_writes_notes() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "retention.renew_on_use",
            "--value",
            "true",
        ],
    )?;
    write_note(&dir, "nota gama", "fact", &[])?;
    let before = notas_fingerprint(&dir)?;

    ok(&dir, &["ask", "nota"])?;

    assert!(
        usage_path(&dir).exists(),
        "uso não creditado com renew_on_use (D154)"
    );
    assert_eq!(
        notas_fingerprint(&dir)?,
        before,
        "leitura alterou notas/ (R45-7)"
    );
    Ok(())
}

/// Nota superseded com uso alto não volta no `ask` por default (R45-3).
#[test]
fn supersession_beats_usage() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    ok(
        &dir,
        &[
            "config",
            "set",
            "--key",
            "retention.renew_on_use",
            "--value",
            "true",
        ],
    )?;
    let old = write_note(&dir, "o banco é postgres", "fact", &[])?;
    for _ in 0..3 {
        ok(&dir, &["ask", "postgres"])?;
    }
    let new = write_note(
        &dir,
        "o banco é sqlite",
        "fact",
        &["--update", old.as_str()],
    )?;
    assert_ne!(old, new, "--update com statement novo deve criar id novo");

    let lineage = ok(&dir, &["ask", "postgres", "--status", "superseded"])?;
    assert!(
        lineage.contains(&old),
        "nota antiga não ficou superseded: {lineage}"
    );

    let hits = ok(&dir, &["ask", "postgres", "--limit", "10"])?;
    assert!(
        !hits.contains(&old),
        "superseded voltou pelo uso (R45-3): {hits}"
    );
    Ok(())
}
