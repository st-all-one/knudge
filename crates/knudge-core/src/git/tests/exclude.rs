//! Testes de exclusão via `.git/info/exclude` (E04-T03).

use std::path::Path;

use super::*;
use crate::git::Persistence;
use crate::git::exclude::{self, exclude_path};
use crate::ports::Fs;

fn read(fs: &MemFs, path: &Path) -> String {
    String::from_utf8(fs.read(path).unwrap_or_default()).unwrap_or_default()
}

#[test]
fn persists_derived_lines_and_is_idempotent() -> Result<()> {
    let fs = MemFs::new();
    let common = Path::new("/repo/.git");
    assert!(exclude::apply(&fs, Some(common), Persistence::Versioned)?);
    let text = read(&fs, &exclude_path(common));
    assert!(text.contains("/.knudge/.idx/"));
    assert!(text.contains("/.knudge/cache/"));
    assert!(
        !text.contains("\n/.knudge/\n"),
        "não exclui o diretório inteiro"
    );
    // Segunda aplicação não muda nada.
    assert!(!exclude::apply(&fs, Some(common), Persistence::Versioned)?);
    Ok(())
}

#[test]
fn local_only_excludes_whole_dir() -> Result<()> {
    let fs = MemFs::new();
    let common = Path::new("/repo/.git");
    exclude::apply(&fs, Some(common), Persistence::LocalOnly)?;
    let text = read(&fs, &exclude_path(common));
    assert!(text.contains("/.knudge/"));
    assert!(!text.contains("/.knudge/.idx/"));
    Ok(())
}

#[test]
fn flipping_modes_reverts_previous_lines() -> Result<()> {
    let fs = MemFs::new();
    let common = Path::new("/repo/.git");
    exclude::apply(&fs, Some(common), Persistence::LocalOnly)?;
    exclude::apply(&fs, Some(common), Persistence::Versioned)?;
    let text = read(&fs, &exclude_path(common));
    assert!(text.contains("/.knudge/.idx/"));
    assert!(!text.lines().any(|l| l.trim() == "/.knudge/"));
    Ok(())
}

#[test]
fn preserves_unrelated_lines() -> Result<()> {
    let fs = MemFs::new();
    let common = Path::new("/repo/.git");
    let path = exclude_path(common);
    fs.create_dir_all(Path::new("/repo/.git/info"))?;
    fs.write_atomic(&path, b"# meu\n*.local\n")?;
    exclude::apply(&fs, Some(common), Persistence::Versioned)?;
    let text = read(&fs, &path);
    assert!(text.contains("# meu"));
    assert!(text.contains("*.local"));
    assert!(text.contains("/.knudge/.idx/"));
    Ok(())
}

#[test]
fn outside_repo_is_noop() -> Result<()> {
    let fs = MemFs::new();
    assert!(!exclude::apply(&fs, None, Persistence::LocalOnly)?);
    assert!(!exclude::apply(&fs, None, Persistence::Versioned)?);
    Ok(())
}
