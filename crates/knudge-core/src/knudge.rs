//! Fachada de incorporação: monta os adaptadores reais, resolve o projeto e carrega a config.
//!
//! É a mesma sequência que `knudge-cli`/`knudge-mcp` executam, exposta como biblioteca para que
//! outro projeto use o núcleo como dependência. O diretório de conhecimento é configurável
//! (default `.knudge`) e aceita caminhos aninhados (ex.: `.a/b`).
//!
//! ```no_run
//! use knudge_core::Knudge;
//!
//! # fn main() -> knudge_core::Result<()> {
//! let kd = Knudge::builder()
//!     .knowledge_dir(".a/b")
//!     .open()?;
//! let index = kd.index()?;
//! # let _ = index;
//! # Ok(())
//! # }
//! ```
//!
//! O domínio continua puro: esta fachada só existe para conveniência. Quem precisa injetar
//! portas próprias pode usar [`Project`], [`Store`], [`Index`] etc. diretamente.

use std::path::{Path, PathBuf};

use crate::adapters::{StdEnv, StdFs, StdGit, SystemClock};
use crate::config::{Config, global_config_path};
use crate::corpus::Corpus;
use crate::error::Result;
use crate::git::{KNUDGE_DIR, Project};
use crate::graph::Graph;
use crate::ports::{Clock, Fs, Logger};
use crate::retrieval::Index;
use crate::store::{EventLog, LockPolicy, Note, Store, sweep_residues};
use crate::write::{DedupThresholds, WriteContext, thresholds_from_config};

/// Construtor de [`Knudge`].
#[derive(Debug, Clone, Default)]
pub struct KnudgeBuilder {
    root: Option<PathBuf>,
    knowledge_dir: Option<PathBuf>,
}

/// Contexto de uso do núcleo por uma aplicação embutida.
///
/// Expõe store, eventos, índice, corpus e grafo já ancorados no [`Project`] resolvido.
pub struct Knudge {
    fs: StdFs,
    git: StdGit,
    env: StdEnv,
    project: Project,
    config: Config,
    now_ms: i64,
}

impl Knudge {
    /// Construtor com opções.
    #[must_use]
    pub fn builder() -> KnudgeBuilder {
        KnudgeBuilder::default()
    }

    /// Abre o projeto do diretório atual com o layout default (`.knudge`).
    ///
    /// # Errors
    /// Propaga erros de resolução de projeto/config e leitura de ambiente.
    pub fn open() -> Result<Self> {
        KnudgeBuilder::default().open()
    }

    /// Projeto resolvido.
    #[must_use]
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Config efetiva (defaults + global + projeto).
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Raiz do worktree principal.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        self.project.root()
    }

    /// Diretório de conhecimento completo (`<raiz>/<layout>`).
    #[must_use]
    pub fn knowledge_dir(&self) -> PathBuf {
        self.project.knowledge_dir()
    }

    /// Instante da sessão (ms desde a época).
    #[must_use]
    pub const fn now_ms(&self) -> i64 {
        self.now_ms
    }

    /// Store de notas.
    #[must_use]
    pub fn store(&self) -> Store<'_> {
        Store::new(&self.fs, self.knowledge_dir())
    }

    /// Log de eventos.
    #[must_use]
    pub fn events(&self) -> EventLog<'_> {
        EventLog::new(&self.fs, self.knowledge_dir(), EventLog::DEFAULT_MAX_BYTES)
    }

    /// Índice de retrieval (reconstruído do store).
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse das notas.
    pub fn index(&self) -> Result<Index> {
        Index::from_store(&self.store())
    }

    /// Corpus de leitura única: notas + índice + grafo.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse das notas.
    pub fn corpus(&self) -> Result<Corpus> {
        Corpus::load(&self.store())
    }

    /// Notas do corpus numa única passada, sem índice/grafo.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura das notas.
    pub fn notes(&self) -> Result<Vec<Note>> {
        Corpus::load_notes(&self.store())
    }

    /// Grafo de arestas (reconstruído do store).
    ///
    /// # Errors
    /// Propaga erros de leitura das notas.
    pub fn graph(&self) -> Result<Graph> {
        Graph::build(&self.store())
    }

    /// Limiares de dedup da config efetiva.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para limiares inconsistentes.
    pub fn thresholds(&self) -> Result<DedupThresholds> {
        thresholds_from_config(&self.config)
    }

    /// Contexto de escrita (store + eventos + índice + relógio).
    ///
    /// # Errors
    /// Propaga erros de leitura do índice.
    pub fn write_context(&self) -> Result<WriteContext<'_>> {
        Ok(WriteContext::new(
            self.store(),
            self.events(),
            self.index()?,
            self.now_ms,
        ))
    }

    /// Porta de FS real.
    #[must_use]
    pub const fn fs(&self) -> StdFs {
        self.fs
    }

    /// Porta de FS como trait (para o domínio que recebe `&dyn Fs`).
    #[must_use]
    pub fn fs_dyn(&self) -> &dyn Fs {
        &self.fs
    }

    /// Porta de Git real.
    #[must_use]
    pub fn git(&self) -> &StdGit {
        &self.git
    }

    /// Porta de ambiente real.
    #[must_use]
    pub fn env(&self) -> &StdEnv {
        &self.env
    }

    /// Varre resíduos (`*.tmp`/`*.stale`) das áreas de dados (R10/D160).
    ///
    /// Best-effort (R33): nunca toca `*.lock` nem `.locks/`. Falhas viram avisos.
    #[must_use]
    pub fn sweep_residues(&self, logger: &dyn Logger) -> Vec<String> {
        let root = self.knowledge_dir();
        let threshold = LockPolicy::default().stale_ms;
        let mut warnings = Vec::new();
        for area in ["notas", ".idx", "cache", "eventos"] {
            let dir = root.join(area);
            if !self.fs.exists(&dir) {
                continue;
            }
            let swept = sweep_residues(&self.fs, &dir, self.now_ms, threshold, logger);
            if let Err(error) = swept {
                warnings.push(format!("varredura de resíduos em {area}/: {error}"));
            }
        }
        warnings
    }
}

impl KnudgeBuilder {
    /// Define a raiz do projeto, sem consultar o Git (`in_repo() == false`).
    #[must_use]
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Define o diretório de conhecimento relativo à raiz (default `.knudge`).
    ///
    /// Aceita caminhos aninhados (`.a/b`); rejeita absolutos e travessias (`..`).
    #[must_use]
    pub fn knowledge_dir(mut self, layout: impl Into<PathBuf>) -> Self {
        self.knowledge_dir = Some(layout.into());
        self
    }

    /// Resolve o projeto e carrega a config efetiva.
    ///
    /// # Errors
    /// Propaga erros de resolução de projeto/config e leitura de ambiente.
    pub fn open(self) -> Result<Knudge> {
        let fs = StdFs::new();
        let git = StdGit::new();
        let env = StdEnv::new();
        let clock = SystemClock::new();
        let layout = self
            .knowledge_dir
            .unwrap_or_else(|| PathBuf::from(KNUDGE_DIR));
        let project = match self.root {
            Some(root) => Project::at(root, layout)?,
            None => Project::resolve_with(&git, &env, layout)?,
        };
        let config = load_config(fs, env, &project)?;
        Ok(Knudge {
            fs,
            git,
            env,
            project,
            config,
            now_ms: clock.now().as_millis(),
        })
    }
}

fn load_config(fs: StdFs, env: StdEnv, project: &Project) -> Result<Config> {
    let global = match global_config_path(&env) {
        Ok(path) => Config::load(&fs, &path)?,
        Err(_) => None,
    };
    let local = Config::load(&fs, &project.config_path())?;
    Ok(Config::effective(global.as_ref(), local.as_ref()))
}
