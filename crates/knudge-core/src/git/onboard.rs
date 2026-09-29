//! `onboard()` idempotente: cria/atualiza `.knudge/`, clona o config e aplica exclusões
//! (E04-T01/T03/T04).

use std::path::{Path, PathBuf};

use crate::Result;
use crate::config::{Config, ConfigValue, global_config_path};
use crate::ports::{Env, Fs, Git};
use crate::schema::NoteType;

use super::agent_md;
use super::attributes;
use super::exclude;
use super::persistence::Persistence;
use super::project::{KNUDGE_DIR, Project};
use super::skill;

/// Subdiretórios criados dentro de `.knudge/`.
pub const LAYOUT_DIRS: &[&str] = &["notas", "eventos", ".idx", "cache"];

/// Opções de `onboard`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnboardOptions {
    /// Sobrescreve um `config.toml` de projeto existente.
    pub force: bool,
    /// Força o modo de persistência (D34), sobrepondo o config clonado ou existente.
    pub persistence: Option<Persistence>,
}

/// Relatório do `onboard` (idempotente e auditável).
#[allow(
    clippy::struct_excessive_bools,
    reason = "relatório com flags independentes de mudança; bool é o tipo natural"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardReport {
    /// Raiz do worktree principal.
    pub root: PathBuf,
    /// Diretório `.knudge/`.
    pub knowledge_dir: PathBuf,
    /// Nome lógico do projeto.
    pub name: String,
    /// `true` se dentro de um repositório git.
    pub in_repo: bool,
    /// `true` se o `config.toml` do projeto foi (re)escrito.
    pub config_written: bool,
    /// `true` se o `.git/info/exclude` mudou.
    pub exclude_changed: bool,
    /// `true` se o `.gitattributes` mudou.
    pub attributes_changed: bool,
    /// `true` se o bloco do `AGENTS.md` foi criado/atualizado.
    pub agents_changed: bool,
    /// `true` se a skill `.agents/skill/kd/SKILL.md` foi criada/atualizada (D162).
    pub skill_changed: bool,
}

/// Executa `onboard` com o layout default (`.knudge`).
///
/// # Errors
/// Propaga erros de resolução de projeto, I/O e configuração.
pub fn onboard(
    fs: &dyn Fs,
    git: &dyn Git,
    env: &dyn Env,
    options: OnboardOptions,
) -> Result<OnboardReport> {
    onboard_with_layout(fs, git, env, options, KNUDGE_DIR)
}

/// Executa `onboard` para um diretório de conhecimento alternativo.
///
/// Reexecutar é seguro: nada é duplicado nem sobrescrito sem `force`.
///
/// # Errors
/// Propaga erros de resolução de projeto, I/O e configuração.
pub fn onboard_with_layout(
    fs: &dyn Fs,
    git: &dyn Git,
    env: &dyn Env,
    options: OnboardOptions,
    layout: impl Into<PathBuf>,
) -> Result<OnboardReport> {
    let project = Project::resolve_with(git, env, layout)?;
    let knowledge_dir = project.knowledge_dir();
    fs.create_dir_all(&knowledge_dir)?;
    for dir in LAYOUT_DIRS {
        fs.create_dir_all(&knowledge_dir.join(dir))?;
    }
    let notes_dir = knowledge_dir.join("notas");
    for note_type in NoteType::ALL {
        fs.create_dir_all(&notes_dir.join(note_type.as_str()))?;
    }
    fs.create_dir_all(&notes_dir.join(NoteType::Epic.as_str()))?;

    let config_path = project.config_path();
    let (config, config_written) = ensure_config(fs, env, &config_path, options)?;
    let persistence = if config
        .get_bool("knowledge.persist_in_project")
        .unwrap_or(true)
    {
        Persistence::Versioned
    } else {
        Persistence::LocalOnly
    };

    let layout = project.layout_str();
    let exclude_changed = if project.in_repo() {
        exclude::apply_with_layout(fs, project.common_dir(), persistence, &layout)?
    } else {
        false
    };
    let attributes_changed = if project.in_repo() {
        attributes::apply_with_layout(fs, project.root(), persistence, &layout)?
    } else {
        false
    };
    let agents_changed = agent_md::apply_with_layout(fs, project.root(), &layout)?;
    let skill_changed = skill::apply(fs, project.root())?;

    Ok(OnboardReport {
        root: project.root().to_path_buf(),
        knowledge_dir,
        name: project.name().to_string(),
        in_repo: project.in_repo(),
        config_written,
        exclude_changed,
        attributes_changed,
        agents_changed,
        skill_changed,
    })
}

/// Garante o `config.toml` do projeto e devolve `(config, escrito)`.
///
/// `force` recopia o global; `options.persistence` sobrepõe `persist_in_project` (D213) —
/// inclusive sobre um clone do global — e marca o config como escrito.
///
/// # Errors
/// Propaga erros de I/O e de validação do config.
fn ensure_config(
    fs: &dyn Fs,
    env: &dyn Env,
    config_path: &Path,
    options: OnboardOptions,
) -> Result<(Config, bool)> {
    let mut written = if fs.exists(config_path) && !options.force {
        false
    } else {
        let bytes = match global_config_path(env) {
            Ok(global_path) if fs.exists(&global_path) => fs.read(&global_path)?,
            _ => Config::defaults().render().into_bytes(),
        };
        fs.write_atomic(config_path, &bytes)?;
        true
    };

    let mut config = Config::load(fs, config_path)?.unwrap_or_default();
    if let Some(desired) = options.persistence {
        let versioned = desired.is_versioned();
        if config.get_bool("knowledge.persist_in_project") != Some(versioned) {
            config.set_value("knowledge.persist_in_project", ConfigValue::Bool(versioned))?;
            config.save(fs, config_path)?;
            written = true;
        }
    }
    Ok((config, written))
}
