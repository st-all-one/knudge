//! Testes de `sync()` (E04-T05).

use super::*;
use crate::git::{Persistence, Project, sync};
use crate::ports::GitOutput;
use crate::store::{Event, EventLog};

fn success() -> GitOutput {
    GitOutput {
        status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    }
}

#[test]
fn rejects_outside_repo() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(false, None, None, None);
    let project = Project::resolve(&git, &env_at("/work/proj"))?;
    assert!(sync(&fs, &git, &project, Persistence::Versioned, None).is_err());
    Ok(())
}

#[test]
fn not_persisting_is_noop() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    let project = Project::resolve(&git, &env_at("/repo"))?;
    let report = sync(&fs, &git, &project, Persistence::LocalOnly, None)?;
    assert!(!report.committed);
    assert!(git.commands().is_empty(), "não deve chamar git");
    Ok(())
}

#[test]
fn clean_tree_is_noop() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    git.push_output(success());
    let project = Project::resolve(&git, &env_at("/repo"))?;
    let report = sync(&fs, &git, &project, Persistence::Versioned, None)?;
    assert!(!report.committed);
    assert_eq!(report.message, "sem mudanças");
    Ok(())
}

#[test]
fn commits_with_generated_message_in_main_worktree() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    let status = GitOutput {
        status: 0,
        stdout: b" M .knudge/notas/fact_abc.md\n".to_vec(),
        stderr: Vec::new(),
    };
    git.push_output(status);
    git.push_output(success()); // add
    git.push_output(success()); // commit

    let project = Project::resolve(&git, &env_at("/repo"))?;
    let log = EventLog::new(&fs, project.knowledge_dir(), EventLog::DEFAULT_MAX_BYTES);
    let event = Event::new("write", 1)
        .with_note_id("fact_abc")
        .with_actor("cli");
    log.append(&event)?;

    let report = sync(&fs, &git, &project, Persistence::Versioned, None)?;
    assert!(report.committed);
    assert_eq!(report.message, "knudge: write fact_abc");
    assert_eq!(report.files.len(), 1);

    let commands = git.commands();
    assert!(
        commands.iter().all(|args| {
            args.first().map(String::as_str) == Some("-C")
                && args.get(1).map(String::as_str) == Some("/repo")
        }),
        "todo comando deve mirar o worktree principal via -C: {commands:?}"
    );
    assert!(commands.iter().any(|args| {
        args.contains(&"commit".to_string()) && args.contains(&"knudge: write fact_abc".to_string())
    }));
    Ok(())
}

#[test]
fn explicit_message_wins() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    git.push_output(GitOutput {
        status: 0,
        stdout: b" M .knudge/eventos/events.jsonl\n".to_vec(),
        stderr: Vec::new(),
    });
    git.push_output(success());
    git.push_output(success());
    let project = Project::resolve(&git, &env_at("/repo"))?;
    let report = sync(
        &fs,
        &git,
        &project,
        Persistence::Versioned,
        Some("mensagem fixa"),
    )?;
    assert_eq!(report.message, "mensagem fixa");
    Ok(())
}

#[test]
fn git_failure_is_reported() -> Result<()> {
    let fs = MemFs::new();
    let git = git_with(true, Some("/repo/.git"), Some("/repo"), None);
    git.push_output(GitOutput {
        status: 1,
        stdout: Vec::new(),
        stderr: b"boom".to_vec(),
    });
    let project = Project::resolve(&git, &env_at("/repo"))?;
    assert!(sync(&fs, &git, &project, Persistence::Versioned, None).is_err());
    Ok(())
}
