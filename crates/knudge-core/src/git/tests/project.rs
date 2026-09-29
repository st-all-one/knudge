//! Testes de resolução de worktree e nome lógico (E04-T02).

use std::path::Path;

use super::*;
use crate::git::{Project, is_valid_layout, is_valid_name, logical_name};

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

#[test]
fn custom_layout_is_used_for_knowledge_dir() -> Result<()> {
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    let project = Project::resolve_with(&git, &env_at("/repo"), ".a/b")?;
    assert_eq!(project.layout(), Path::new(".a/b"));
    assert_eq!(project.layout_str(), ".a/b");
    assert_eq!(project.knowledge_dir(), Path::new("/repo/.a/b"));
    assert_eq!(project.config_path(), Path::new("/repo/.a/b/config.toml"));
    Ok(())
}

#[test]
fn default_layout_keeps_knudge_dir() -> Result<()> {
    let git = git_with(false, None, None, None);
    let project = Project::resolve(&git, &env_at("/work/proj"))?;
    assert_eq!(project.layout(), Path::new(".knudge"));
    assert_eq!(project.knowledge_dir(), Path::new("/work/proj/.knudge"));
    Ok(())
}

#[test]
fn project_at_bypasses_git() -> Result<()> {
    let project = Project::at("/work/proj", "sub/mem")?;
    assert!(!project.in_repo());
    assert_eq!(project.common_dir(), None);
    assert_eq!(project.knowledge_dir(), Path::new("/work/proj/sub/mem"));
    Ok(())
}

#[test]
fn with_layout_revalidates() -> Result<()> {
    let git = git_with(false, None, None, None);
    let project = Project::resolve(&git, &env_at("/work/proj"))?;
    let project = project.with_layout(".mem/notes")?;
    assert_eq!(project.knowledge_dir(), Path::new("/work/proj/.mem/notes"));
    assert!(project.with_layout("../fora").is_err());
    Ok(())
}

#[test]
fn ignored_dirs_track_the_layout_top_component() -> Result<()> {
    let default = Project::at("/work/proj", ".knudge")?;
    assert!(default.ignored_dirs().iter().any(|dir| dir == ".knudge"));
    let custom = Project::at("/work/proj", ".a/b")?;
    assert!(custom.ignored_dirs().iter().any(|dir| dir == ".a"));
    assert!(!custom.ignored_dirs().iter().any(|dir| dir == ".a/b"));
    Ok(())
}

#[test]
fn layout_validation_rejects_dots_and_paths() {
    assert!(is_valid_layout(Path::new(".knudge")));
    assert!(is_valid_layout(Path::new(".a/b")));
    assert!(!is_valid_layout(Path::new("")));
    assert!(!is_valid_layout(Path::new("/abs")));
    assert!(!is_valid_layout(Path::new("../x")));
    assert!(!is_valid_layout(Path::new("a/../b")));
    assert!(!is_valid_layout(Path::new(".")));
}
