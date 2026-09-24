//! Varredura de resíduos na inicialização (R10 / D160): `*.tmp`/`*.stale` antigos saem,
//! frescos e locks ficam.

mod common;

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use common::{TestResult, init, run_in, stderr, temp_project, write_note};

/// Cria um arquivo e recua o `mtime` para 1970 (resíduo comprovadamente antigo).
fn write_aged(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, bytes)?;
    let file = fs::OpenOptions::new().write(true).open(path)?;
    let aged = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1))
        .ok_or_else(|| std::io::Error::other("mtime fora de faixa"))?;
    file.set_modified(aged)?;
    Ok(())
}

#[test]
fn startup_sweep_removes_aged_tmp_but_keeps_fresh_and_locks() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    write_note(&dir, "nota alfa", "fact", &[])?;

    let knudge = dir.join(".knudge");
    fs::create_dir_all(knudge.join(".locks"))?;
    let aged = knudge.join("notas").join("orfa.md.tmp");
    let fresh = knudge.join("notas").join("fresco.md.tmp");
    let lock = knudge.join(".locks").join("velho.lock");
    write_aged(&aged, b"orfa")?;
    fs::write(&fresh, b"fresco")?;
    write_aged(&lock, b"lock")?;

    let out = run_in(&dir, &["ask", "nota"])?;
    assert!(out.status.success(), "ask falhou: {}", stderr(&out)?);

    assert!(!aged.exists(), "resíduo antigo não foi removido");
    assert!(fresh.exists(), "resíduo fresco foi removido");
    assert!(lock.exists(), "lock não pode ser varrido (D160)");
    assert!(
        stderr(&out)?.contains("resíduo"),
        "remoção não reportada: {}",
        stderr(&out)?
    );
    Ok(())
}
