//! Testes do escopo `git` (E04).

mod agents;
mod attributes;
mod exclude;
mod onboard;
mod project;
mod sync;

use std::path::PathBuf;

use crate::Result;
use crate::ports::fakes::{FakeEnv, FakeGit, MemFs};

/// Cria um `FakeGit` com os campos de consulta preenchidos.
#[allow(
    clippy::fn_params_excessive_bools,
    reason = "helper de teste; `repo` é um flag de cenário"
)]
pub(super) fn git_with(
    repo: bool,
    common: Option<&str>,
    top: Option<&str>,
    superproject: Option<&str>,
) -> FakeGit {
    let mut git = FakeGit::new();
    git.repo = repo;
    git.common = common.map(PathBuf::from);
    git.top = top.map(PathBuf::from);
    git.superproject = superproject.map(PathBuf::from);
    git
}

/// Cria um `FakeEnv` com o diretório de trabalho dado.
pub(super) fn env_at(cwd: &str) -> FakeEnv {
    FakeEnv {
        cwd: PathBuf::from(cwd),
        ..FakeEnv::default()
    }
}
