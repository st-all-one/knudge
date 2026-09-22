//! Testes de resolução de worktree e nome lógico (E04-T02).

use std::path::Path;

use super::*;
use crate::git::{Project, is_valid_name, logical_name};

#[test]
fn resolves_main_worktree_from_common_dir() -> Result<()> {
    let git = git_with(true, Some("/repo/.git"), Some("/repo/wt"), None);
    let project = Project::resolve(&git, &env_at("/repo/wt"))?;
    assert_eq!(project.root(), Path::new("/repo"));
    assert_eq!(project.name(), "repo");
    assert!(project.in_repo());
    Ok(())
}

#[test]
fn linked_worktree_shares_main_worktree() -> Result<()> {
    // Worktree ligado: `top` é outro diretório, mas `common` aponta para o principal.
    let git = git_with(true, Some("/repo/.git"), Some("/repo/wt-feature"), None);
    let project = Project::resolve(&git, &env_at("/repo/wt-feature"))?;
    assert_eq!(project.root(), Path::new("/repo"));
    Ok(())
}

#[test]
fn submodule_uses_its_own_worktree() -> Result<()> {
    let git = git_with(
        true,
        Some("/super/.git/modules/sub"),
        Some("/super/sub"),
        Some("/super"),
    );
    let project = Project::resolve(&git, &env_at("/super/sub"))?;
    assert_eq!(project.root(), Path::new("/super/sub"));
    assert_eq!(project.name(), "sub");
    Ok(())
}

#[test]
fn outside_repo_uses_cwd() -> Result<()> {
    let git = git_with(false, None, None, None);
    let project = Project::resolve(&git, &env_at("/work/proj"))?;
    assert_eq!(project.root(), Path::new("/work/proj"));
    assert_eq!(project.name(), "proj");
    assert!(!project.in_repo());
    assert_eq!(project.common_dir(), None);
    Ok(())
}

#[test]
fn name_validation_rejects_dots_and_paths() {
    assert!(is_valid_name("proj"));
    assert!(!is_valid_name(""));
    assert!(!is_valid_name("."));
    assert!(!is_valid_name(".."));
    assert!(!is_valid_name("a/b"));
    assert!(!is_valid_name("/abs"));
    assert!(logical_name(Path::new("/")).is_err());
}
