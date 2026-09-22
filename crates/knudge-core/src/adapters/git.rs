//! Adaptador `git` real (chama o binário `git`, nunca um shell — R12).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Result;
use crate::ports::{Git, GitOutput};

/// Executa `git` em um diretório de trabalho.
#[derive(Debug, Clone)]
pub struct StdGit {
    /// Diretório onde os comandos rodam.
    cwd: PathBuf,
}

impl StdGit {
    /// Cria o adaptador apontando para o diretório atual.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cwd: PathBuf::from("."),
        }
    }

    /// Cria o adaptador apontando para `dir`.
    #[must_use]
    pub fn in_dir(dir: impl Into<PathBuf>) -> Self {
        Self { cwd: dir.into() }
    }

    /// Roda um `rev-parse` e devolve o caminho, se for bem-sucedido e não vazio.
    fn path(&self, args: &[&str]) -> Option<PathBuf> {
        let output = self.run(args).ok()?;
        if output.status != 0 {
            return None;
        }
        let text = output.stdout_text();
        if text.is_empty() {
            None
        } else {
            Some(PathBuf::from(text))
        }
    }
}

impl Default for StdGit {
    fn default() -> Self {
        Self::new()
    }
}

impl Git for StdGit {
    fn is_repo(&self) -> bool {
        self.run(&["rev-parse", "--is-inside-work-tree"])
            .is_ok_and(|output| output.status == 0 && output.stdout_text() == "true")
    }

    fn common_dir(&self) -> Option<PathBuf> {
        self.path(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
    }

    fn top_level(&self) -> Option<PathBuf> {
        self.path(&["rev-parse", "--show-toplevel"])
    }

    fn superproject_root(&self) -> Option<PathBuf> {
        self.path(&["rev-parse", "--show-superproject-working-tree"])
    }

    fn status_porcelain(&self) -> Result<Vec<String>> {
        let output = self.run(&["status", "--porcelain"])?;
        if output.status != 0 {
            return Err(crate::Error::internal(format!(
                "git status falhou (status {})",
                output.status
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    }

    fn run(&self, args: &[&str]) -> Result<GitOutput> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.cwd)
            .output()
            .map_err(|error| crate::Error::io(self.cwd.as_path(), error))?;
        Ok(GitOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Caminho passado ao `git -C` a partir de um `Path`.
#[must_use]
pub fn as_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
