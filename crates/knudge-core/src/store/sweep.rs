//! Varredura de resíduos na inicialização (R10).
//!
//! Remove `*.tmp`, `*.lock` e `*.stale` mais velhos que um limiar, com `warn`. Archivos frescos
//! de um processo vivo **não** são tocados (idade ≤ limiar).

#![allow(
    clippy::arithmetic_side_effects,
    reason = "contador de resíduos removidos com domínio limitado"
)]

use std::path::{Path, PathBuf};

use crate::Result;
use crate::ports::{Fs, Level, LogRecord, Logger};

/// Percorre `dir` recursivamente e remove resíduos abandonados; devolve quantos removeu.
///
/// # Errors
/// Retorna `ErrorKind::Io` se listagem ou remoção falharem.
pub fn sweep_residues(
    fs: &dyn Fs,
    dir: &Path,
    now_ms: i64,
    max_age_ms: i64,
    logger: &dyn Logger,
) -> Result<usize> {
    let mut removed = 0_usize;
    for path in walk(fs, dir)? {
        if !is_residue(&path) {
            continue;
        }
        let Ok(Some(modified)) = fs.modified_ms(&path) else {
            continue;
        };
        if now_ms.saturating_sub(modified) <= max_age_ms {
            continue;
        }
        fs.remove_file(&path)?;
        removed += 1;
        let display = path.display().to_string();
        logger.log(&LogRecord {
            level: Level::Warn,
            message: "resíduo removido",
            fields: &[("path", display.as_str())],
        });
    }
    Ok(removed)
}

fn walk(fs: &dyn Fs, dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if fs.exists(dir) {
        collect(fs, dir, &mut out)?;
    }
    Ok(out)
}

fn collect(fs: &dyn Fs, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for path in fs.list_dir(dir)? {
        if fs.is_dir(&path) {
            collect(fs, &path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn is_residue(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    [".tmp", ".lock", ".stale"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}
