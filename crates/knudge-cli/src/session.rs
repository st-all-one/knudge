//! Sessão de CLI: resolve o projeto, carrega a config efetiva e monta o domínio (E12-T01).
//!
//! É a **única** camada que toca adaptadores reais; o domínio recebe portas. Cada comando pede
//! store/eventos/índice/grafo sob demanda — nunca há estado global mutável.

use std::path::{Path, PathBuf};

use knudge_core::Result;
use knudge_core::adapters::{StdEnv, StdFs, StdGit, SystemClock};
use knudge_core::config::{Config, global_config_path};
use knudge_core::corpus::Corpus;
use knudge_core::git::Project;
use knudge_core::graph::Graph;
use knudge_core::logging::Redactor;
use knudge_core::ports::{Clock, Fs, Git};
use knudge_core::retrieval::Index;
use knudge_core::store::{EventLog, LockPolicy, Note, Store, sweep_residues};
use knudge_core::write::{DedupThresholds, WriteContext, thresholds_from_config};

use crate::logging::TracingLogger;

/// Contexto resolvido de uma execução do `kd`.
pub struct Session {
    fs: StdFs,
    git: StdGit,
    env: StdEnv,
    project: Project,
    config: Config,
    now_ms: i64,
}

impl Session {
    /// Resolve o projeto e carrega a config efetiva (defaults + global + projeto).
    ///
    /// # Errors
    /// Propaga erros de resolução de projeto/config e leitura de ambiente.
    pub fn open() -> Result<Self> {
        let fs = StdFs::new();
        let git = StdGit::new();
        let env = StdEnv::new();
        let clock = SystemClock::new();
        let project = Project::resolve(&git, &env)?;
        let config = load_config(fs, env, &project)?;
        Ok(Self {
            fs,
            git,
            env,
            project,
            config,
            now_ms: clock.now().as_millis(),
        })
    }

    /// Projeto resolvido.
    #[must_use]
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Config efetiva.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Raiz do worktree principal.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        self.project.root()
    }

    /// Diretório `.knudge/`.
    #[must_use]
    pub fn knowledge_dir(&self) -> PathBuf {
        self.project.knowledge_dir()
    }

    /// Instante da sessão (ms desde a época).
    #[must_use]
    pub const fn now_ms(&self) -> i64 {
        self.now_ms
    }

    /// Porta de FS real.
    #[must_use]
    pub const fn fs(&self) -> StdFs {
        self.fs
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

    /// Corpus de leitura única: notas + índice + grafo (E15-T02/O1).
    ///
    /// Prefira este método a chamar [`Session::index`]/[`Session::graph`] (ou a reler as notas)
    /// separadamente: todos derivam do **mesmo** vetor de notas, numa só passada pelo store.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse das notas.
    pub fn corpus(&self) -> Result<Corpus> {
        Corpus::load(&self.store())
    }

    /// Notas do corpus numa única passada, sem índice/grafo (E15-T20/O8.1).
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

    /// Porta de Git real.
    #[must_use]
    pub fn git(&self) -> &StdGit {
        &self.git
    }

    /// Arquivos modificados no worktree (vazio fora de repositório).
    ///
    /// # Errors
    /// Propaga erro do `git status`.
    pub fn changed_paths(&self) -> Result<Vec<String>> {
        if !self.project.in_repo() {
            return Ok(Vec::new());
        }
        let lines = self.git.status_porcelain()?;
        Ok(lines
            .iter()
            .filter_map(|line| line.get(3..).map(str::trim))
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect())
    }

    /// Porta de ambiente real.
    #[must_use]
    pub fn env(&self) -> &StdEnv {
        &self.env
    }
}

impl Session {
    /// Referência à porta de FS (para domínio que recebe `&dyn Fs`).
    #[must_use]
    pub fn fs_dyn(&self) -> &dyn Fs {
        &self.fs
    }

    /// Varre resíduos (`*.tmp`/`*.stale`) das áreas de dados no início da sessão (R10/D160).
    ///
    /// Nunca toca `*.lock` nem `.locks/` — o reclaim de lock é do `lock.rs`/`doctor --fix`.
    /// Best-effort (R33): falha vira aviso; cada remoção é reportada em stderr pelo logger.
    #[must_use]
    pub fn sweep_residues(&self) -> Vec<String> {
        let logger = TracingLogger::new(Redactor::empty());
        let root = self.knowledge_dir();
        let threshold = LockPolicy::default().stale_ms;
        let mut warnings = Vec::new();
        for area in ["notas", ".idx", "cache", "eventos"] {
            let dir = root.join(area);
            if !self.fs.exists(&dir) {
                continue;
            }
            let swept = sweep_residues(&self.fs, &dir, self.now_ms, threshold, &logger);
            if let Err(error) = swept {
                warnings.push(format!("varredura de resíduos em {area}/: {error}"));
            }
        }
        warnings
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
