//! Exclusão idempotente via `.git/info/exclude` (D30/D34).
//!
//! Nunca usamos `.gitignore` (versionado). Fora de um repositório, a operação é no-op.
//! `persist_in_project = true` versiona `notas/`/`eventos/` e exclui só o **derivado**;
//! `false` exclui o `.knudge/` inteiro. Alternar entre os modos **reverte** as linhas do modo
//! anterior sem duplicar.

use std::path::{Path, PathBuf};

use crate::ports::Fs;
use crate::{Error, Result};

use super::persistence::Persistence;

/// Exclusão do diretório inteiro (modo local-only).
pub const KNUDGE_PATTERN: &str = "/.knudge/";

/// Exclusões do derivado quando o conhecimento é versionado.
pub const DERIVED_PATTERNS: &[&str] = &["/.knudge/.idx/", "/.knudge/cache/", "/.knudge/.locks/"];

/// Todas as linhas gerenciadas pelo knudge (em qualquer modo).
const MANAGED: &[&str] = &[
    "/.knudge/",
    "/.knudge/.idx/",
    "/.knudge/cache/",
    "/.knudge/.locks/",
];

/// Caminho de `.git/info/exclude` dentro do diretório comum.
#[must_use]
pub fn exclude_path(common_dir: &Path) -> PathBuf {
    common_dir.join("info").join("exclude")
}

/// Aplica a política de exclusão. Devolve `true` se o arquivo mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` em falha de leitura/escrita ou UTF-8 inválido.
pub fn apply(fs: &dyn Fs, common_dir: Option<&Path>, persistence: Persistence) -> Result<bool> {
    let Some(common_dir) = common_dir else {
        return Ok(false);
    };
    let path = exclude_path(common_dir);
    let original = if fs.exists(&path) {
        String::from_utf8(fs.read(&path)?)
            .map_err(|_| Error::config(format!("exclude não é UTF-8: {}", path.display())))?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    lines.retain(|line| !is_managed(line));

    let desired: Vec<&str> = if persistence.is_versioned() {
        DERIVED_PATTERNS.to_vec()
    } else {
        vec![KNUDGE_PATTERN]
    };
    for pattern in desired {
        if !lines.iter().any(|line| line.trim() == pattern) {
            lines.push(pattern.to_string());
        }
    }

    let mut updated = lines.join("\n");
    if !updated.is_empty() {
        updated.push('\n');
    }
    if updated == original {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.write_atomic(&path, updated.as_bytes())?;
    Ok(true)
}

/// `true` se a linha pertence ao conjunto gerenciado pelo knudge.
#[must_use]
pub fn is_managed(line: &str) -> bool {
    let trimmed = line.trim();
    MANAGED.contains(&trimmed)
}
