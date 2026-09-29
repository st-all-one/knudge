//! Exclusão idempotente via `.git/info/exclude` (D30/D34).
//!
//! Nunca usamos `.gitignore` (versionado). Fora de um repositório, a operação é no-op.
//! `persist_in_project = true` versiona `notas/`/`eventos/` e exclui só o **derivado**;
//! `false` exclui o diretório de conhecimento inteiro. Alternar entre os modos **reverte** as
//! linhas do modo anterior sem duplicar.
//!
//! Os padrões derivam do `layout` do [`Project`](super::Project) — default `.knudge` — para que
//! aplicações embutidas possam trocar o diretório de conhecimento.

use std::path::{Path, PathBuf};

use crate::ports::Fs;
use crate::{Error, Result};

use super::persistence::Persistence;
use super::project::KNUDGE_DIR;

/// Exclusão do diretório inteiro (modo local-only), no layout default.
pub const KNUDGE_PATTERN: &str = "/.knudge/";

/// Exclusões do derivado quando o conhecimento é versionado (layout default).
pub const DERIVED_PATTERNS: &[&str] = &["/.knudge/.idx/", "/.knudge/cache/", "/.knudge/.locks/"];

/// Padrão que exclui o diretório de conhecimento inteiro para `layout`.
#[must_use]
pub fn pattern_for(layout: &str) -> String {
    format!("/{layout}/")
}

/// Padrões que excluem apenas o derivado para `layout`.
#[must_use]
pub fn derived_patterns_for(layout: &str) -> Vec<String> {
    vec![
        format!("/{layout}/.idx/"),
        format!("/{layout}/cache/"),
        format!("/{layout}/.locks/"),
    ]
}

/// Caminho de `.git/info/exclude` dentro do diretório comum.
#[must_use]
pub fn exclude_path(common_dir: &Path) -> PathBuf {
    common_dir.join("info").join("exclude")
}

/// Aplica a política de exclusão com o layout default (`.knudge`). Devolve `true` se mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` em falha de leitura/escrita ou UTF-8 inválido.
pub fn apply(fs: &dyn Fs, common_dir: Option<&Path>, persistence: Persistence) -> Result<bool> {
    apply_with_layout(fs, common_dir, persistence, KNUDGE_DIR)
}

/// Aplica a política de exclusão para um `layout` arbitrário. Devolve `true` se mudou.
///
/// As linhas gerenciadas do layout default também são removidas, de modo que trocar
/// `.knudge` por outro diretório reverte as exclusões antigas.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` em falha de leitura/escrita ou UTF-8 inválido.
pub fn apply_with_layout(
    fs: &dyn Fs,
    common_dir: Option<&Path>,
    persistence: Persistence,
    layout: &str,
) -> Result<bool> {
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
    lines.retain(|line| !is_managed(line) && !is_managed_for(line, layout));

    let desired: Vec<String> = if persistence.is_versioned() {
        derived_patterns_for(layout)
    } else {
        vec![pattern_for(layout)]
    };
    for pattern in desired {
        if !lines.iter().any(|line| line.trim() == pattern) {
            lines.push(pattern);
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

/// `true` se a linha pertence ao conjunto gerenciado pelo knudge (layout default).
#[must_use]
pub fn is_managed(line: &str) -> bool {
    is_managed_for(line, KNUDGE_DIR)
}

/// `true` se a linha pertence ao conjunto gerenciado para `layout`.
#[must_use]
pub fn is_managed_for(line: &str, layout: &str) -> bool {
    let trimmed = line.trim();
    trimmed == pattern_for(layout)
        || derived_patterns_for(layout)
            .iter()
            .any(|pattern| pattern == trimmed)
}
