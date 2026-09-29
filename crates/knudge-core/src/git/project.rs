//! Resolução do projeto: raiz do conhecimento no **worktree principal** e nome lógico (D29/D91).
//!
//! `notas/`, `eventos/` e `config.toml` vivem em `<raiz>/<layout>/`, onde `<raiz>` é o worktree
//! principal do repositório e `<layout>` é o diretório de conhecimento (default `.knudge/`).
//! Worktrees ligados compartilham o mesmo diretório; submódulos são resolvidos no próprio
//! worktree do submódulo (o superprojeto não conta — D29).
//!
//! O [`Project`] carrega o `layout` (caminho relativo) para que aplicações embutidas possam
//! trocar `.knudge` por outro diretório — inclusive aninhado (`.a/b`) — sem tocar no domínio.

use std::path::{Component, Path, PathBuf};

use crate::config::CONFIG_FILE;
use crate::ports::{Env, Git};
use crate::{Error, Result};

/// Nome default do diretório de conhecimento.
pub const KNUDGE_DIR: &str = ".knudge";

/// Projeto resolvido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: PathBuf,
    layout: PathBuf,
    name: String,
    in_repo: bool,
    common_dir: Option<PathBuf>,
}

impl Project {
    /// Resolve o projeto a partir das portas de Git/ambiente, com o layout default (`.knudge`).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a raiz ou o nome não puderem ser resolvidos.
    pub fn resolve(git: &dyn Git, env: &dyn Env) -> Result<Self> {
        Self::resolve_with(git, env, KNUDGE_DIR)
    }

    /// Resolve o projeto usando um diretório de conhecimento alternativo (relativo à raiz).
    ///
    /// O `layout` é um caminho relativo (ex.: `.knudge`, `.a/b`). O default `.knudge` mantém o
    /// comportamento histórico.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a raiz, o nome ou o layout forem inválidos.
    pub fn resolve_with(git: &dyn Git, env: &dyn Env, layout: impl Into<PathBuf>) -> Result<Self> {
        let layout = validate_layout(layout.into())?;
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
                layout,
                name,
                in_repo: true,
                common_dir: common,
            })
        } else {
            let root = env.current_dir()?;
            let name = logical_name(&root)?;
            Ok(Self {
                root,
                layout,
                name,
                in_repo: false,
                common_dir: None,
            })
        }
    }

    /// Constrói um projeto a partir de uma raiz explícita, **sem** consultar o Git.
    ///
    /// Útil para embutir o núcleo apontando para um diretório arbitrário (a persistência via
    /// `git` fica indisponível: `in_repo() == false`).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a raiz, o nome ou o layout forem inválidos.
    pub fn at(root: impl Into<PathBuf>, layout: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let layout = validate_layout(layout.into())?;
        let name = logical_name(&root)?;
        Ok(Self {
            root,
            layout,
            name,
            in_repo: false,
            common_dir: None,
        })
    }

    /// Deriva um novo projeto com outro diretório de conhecimento (default `.knudge`).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se o layout for inválido.
    pub fn with_layout(self, layout: impl Into<PathBuf>) -> Result<Self> {
        let layout = validate_layout(layout.into())?;
        Ok(Self { layout, ..self })
    }

    /// Raiz do worktree principal.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Diretório de conhecimento, relativo à raiz (ex.: `.knudge` ou `.a/b`).
    #[must_use]
    pub fn layout(&self) -> &Path {
        &self.layout
    }

    /// Layout como string com separador `/` (para padrões de `.git/info/exclude` e afins).
    #[must_use]
    pub fn layout_str(&self) -> String {
        self.layout.to_string_lossy().replace('\\', "/")
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

    /// Diretório de conhecimento completo (`<raiz>/<layout>`).
    #[must_use]
    pub fn knowledge_dir(&self) -> PathBuf {
        self.root.join(&self.layout)
    }

    /// Diretórios de topo ignorados numa varredura do projeto.
    ///
    /// O primeiro componente do layout (`.knudge` ou `.a`) é ignorado, além dos diretórios de
    /// build/SO. Ordem estável (determinismo).
    #[must_use]
    pub fn ignored_dirs(&self) -> Vec<String> {
        let top = self
            .layout
            .iter()
            .next()
            .and_then(|component| component.to_str())
            .unwrap_or(KNUDGE_DIR)
            .to_string();
        vec![
            ".git".to_string(),
            top,
            "target".to_string(),
            "node_modules".to_string(),
            ".hg".to_string(),
        ]
    }

    /// Caminho de `<raiz>/<layout>/config.toml`.
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

/// `true` se `layout` é um diretório relativo válido (sem `.`, `..` ou raiz — D91).
///
/// Aceita caminhos aninhados (`a/b`, `.a/b`); rejeita vazios, absolutos e travessias (`../x`).
#[must_use]
pub fn is_valid_layout(layout: &Path) -> bool {
    !layout.as_os_str().is_empty()
        && layout
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Valida e devolve o layout.
///
/// # Errors
/// Retorna `ErrorKind::Config` se o layout não for um caminho relativo simples.
fn validate_layout(layout: PathBuf) -> Result<PathBuf> {
    if is_valid_layout(&layout) {
        Ok(layout)
    } else {
        Err(Error::config(format!(
            "diretório de conhecimento inválido: `{}` (use caminho relativo sem `..`)",
            layout.display()
        )))
    }
}
