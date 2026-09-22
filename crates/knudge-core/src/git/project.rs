//! Resolução do projeto: raiz do conhecimento no **worktree principal** e nome lógico (D29/D91).
//!
//! `notas/`, `eventos/` e `config.toml` vivem em `<raiz>/.knudge/`, onde `<raiz>` é o worktree
//! principal do repositório. Worktrees ligados compartilham o mesmo `.knudge/`; submódulos são
//! resolvidos no próprio worktree do submódulo (o superprojeto não conta — D29).

use std::path::{Path, PathBuf};

use crate::config::CONFIG_FILE;
use crate::ports::{Env, Git};
use crate::{Error, Result};

/// Nome do diretório de conhecimento.
pub const KNUDGE_DIR: &str = ".knudge";

/// Projeto resolvido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: PathBuf,
    name: String,
    in_repo: bool,
    common_dir: Option<PathBuf>,
}

impl Project {
    /// Resolve o projeto a partir das portas de Git/ambiente.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a raiz ou o nome não puderem ser resolvidos.
    pub fn resolve(git: &dyn Git, env: &dyn Env) -> Result<Self> {
        if git.is_repo() {
            let common = git.common_dir();
            let root = if git.superproject_root().is_some() {
                git.top_level()
                    .ok_or_else(|| Error::config("submódulo sem `--show-toplevel`"))?
            } else {
                main_worktree(common.as_deref(), git)?
            };
            let name = logical_name(&root)?;
            Ok(Self {
                root,
                name,
                in_repo: true,
                common_dir: common,
            })
        } else {
            let root = env.current_dir()?;
            let name = logical_name(&root)?;
            Ok(Self {
                root,
                name,
                in_repo: false,
                common_dir: None,
            })
        }
    }

    /// Raiz do worktree principal.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Nome lógico do projeto (basename da raiz, nunca um caminho — D91).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `true` se o projeto está dentro de um repositório git.
    #[must_use]
    pub const fn in_repo(&self) -> bool {
        self.in_repo
    }

    /// Diretório comum do git (`<raiz>/.git` no caso usual).
    #[must_use]
    pub fn common_dir(&self) -> Option<&Path> {
        self.common_dir.as_deref()
    }

    /// Diretório `.knudge/`.
    #[must_use]
    pub fn knowledge_dir(&self) -> PathBuf {
        self.root.join(KNUDGE_DIR)
    }

    /// Caminho de `.knudge/config.toml`.
    #[must_use]
    pub fn config_path(&self) -> PathBuf {
        self.knowledge_dir().join(CONFIG_FILE)
    }
}

/// Deriva a raiz do worktree principal a partir do `git-common-dir`.
fn main_worktree(common: Option<&Path>, git: &dyn Git) -> Result<PathBuf> {
    if let Some(common) = common {
        let is_git_dir = common.file_name().and_then(|n| n.to_str()) == Some(".git");
        if is_git_dir
            && let Some(parent) = common.parent()
            && !parent.as_os_str().is_empty()
        {
            return Ok(parent.to_path_buf());
        }
        if let Some(top) = git.top_level() {
            return Ok(top);
        }
        if let Some(parent) = common.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    git.top_level()
        .ok_or_else(|| Error::config("não foi possível resolver a raiz do projeto"))
}

/// Extrai e valida o nome lógico a partir da raiz.
///
/// # Errors
/// Retorna `ErrorKind::Config` se a raiz não tiver um basename válido.
pub fn logical_name(root: &Path) -> Result<String> {
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::config(format!("raiz sem nome: {}", root.display())))?
        .to_string();
    if is_valid_name(&name) {
        Ok(name)
    } else {
        Err(Error::config(format!("nome de projeto inválido: `{name}`")))
    }
}

/// `true` se `name` é um nome lógico válido (rejeita `.`, `..` e caminhos — D91).
#[must_use]
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\'])
        && !Path::new(name).is_absolute()
}
