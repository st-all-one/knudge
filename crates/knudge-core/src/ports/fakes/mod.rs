//! Fakes determinísticos das portas, para testes sem tocar o sistema operacional (D65).

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::Result;
use crate::error::lock_or_recover;
use crate::logging::Redactor;
use crate::time::Timestamp;

pub mod fs;

pub use fs::{FaultyFs, MemFs};

use super::logger::{Level, LogRecord, Logger};
use super::{Clock, Env, Git, GitOutput, HookOutput, HookRunner, Rng};

/// Relógio fixo, ajustável pelo teste.
#[derive(Debug)]
pub struct FixedClock {
    /// Instante devolvido por [`Clock::now`].
    now: Mutex<Timestamp>,
}

impl FixedClock {
    /// Cria um relógio fixo.
    #[must_use]
    pub fn new(now: Timestamp) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    /// Ajusta o instante devolvido.
    pub fn set(&self, now: Timestamp) {
        *lock_or_recover(&self.now) = now;
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::new(Timestamp::EPOCH)
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        *lock_or_recover(&self.now)
    }
}

/// RNG determinístico (xorshift64) para testes.
#[derive(Debug, Clone)]
pub struct SeqRng {
    /// Estado interno.
    state: u64,
}

impl SeqRng {
    /// Cria com a semente dada (zero é normalizado).
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }
}

impl Default for SeqRng {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Rng for SeqRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// Ambiente falso.
#[derive(Debug, Clone, Default)]
pub struct FakeEnv {
    /// Variáveis disponíveis.
    pub vars: BTreeMap<String, String>,
    /// Argumentos disponíveis.
    pub args: Vec<String>,
    /// Diretório atual.
    pub cwd: PathBuf,
}

impl Env for FakeEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }

    fn args(&self) -> Vec<String> {
        self.args.clone()
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(self.cwd.clone())
    }
}

/// Git falso: campos de consulta + registro de comandos e resultados enfileirados.
#[derive(Debug)]
pub struct FakeGit {
    /// Se está em um repositório.
    pub repo: bool,
    /// `git-common-dir` simulado.
    pub common: Option<PathBuf>,
    /// `show-toplevel` simulado.
    pub top: Option<PathBuf>,
    /// Raiz do superprojeto simulada (submódulo).
    pub superproject: Option<PathBuf>,
    /// Linhas de `status --porcelain`.
    pub status: Vec<String>,
    /// Comandos registrados (sem o prefixo `git`).
    commands: Mutex<Vec<Vec<String>>>,
    /// Resultados a devolver em `run`, em ordem.
    outputs: Mutex<VecDeque<GitOutput>>,
}

impl FakeGit {
    /// Cria um fake vazio (fora de repositório).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enfileira o resultado do próximo `run`.
    pub fn push_output(&self, output: GitOutput) {
        lock_or_recover(&self.outputs).push_back(output);
    }

    /// Comandos executados até agora.
    #[must_use]
    pub fn commands(&self) -> Vec<Vec<String>> {
        lock_or_recover(&self.commands).clone()
    }
}

impl Default for FakeGit {
    fn default() -> Self {
        Self {
            repo: false,
            common: None,
            top: None,
            superproject: None,
            status: Vec::new(),
            commands: Mutex::new(Vec::new()),
            outputs: Mutex::new(VecDeque::new()),
        }
    }
}

impl Git for FakeGit {
    fn is_repo(&self) -> bool {
        self.repo
    }

    fn common_dir(&self) -> Option<PathBuf> {
        self.common.clone()
    }

    fn top_level(&self) -> Option<PathBuf> {
        self.top.clone()
    }

    fn superproject_root(&self) -> Option<PathBuf> {
        self.superproject.clone()
    }

    fn status_porcelain(&self) -> Result<Vec<String>> {
        Ok(if self.repo {
            self.status.clone()
        } else {
            Vec::new()
        })
    }

    fn run(&self, args: &[&str]) -> Result<GitOutput> {
        lock_or_recover(&self.commands).push(args.iter().map(|a| (*a).to_string()).collect());
        Ok(lock_or_recover(&self.outputs)
            .pop_front()
            .unwrap_or(GitOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }))
    }
}

/// Hook runner que não faz nada.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopHookRunner;

impl HookRunner for NoopHookRunner {
    fn run(&self, _hook: &str, _input: &[u8]) -> Result<HookOutput> {
        Ok(HookOutput {
            status: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }
}

/// Registro capturado por [`RecordingLogger`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Nível.
    pub level: Level,
    /// Mensagem já redigida.
    pub message: String,
    /// Campos já redigidos.
    pub fields: Vec<(String, String)>,
}

/// Logger que guarda os registros (já redigidos) para inspeção.
#[derive(Debug, Default)]
pub struct RecordingLogger {
    /// Redator aplicado a cada registro.
    redactor: Redactor,
    /// Registros capturados.
    records: Mutex<Vec<Record>>,
}

impl RecordingLogger {
    /// Cria com um redator.
    #[must_use]
    pub fn new(redactor: Redactor) -> Self {
        Self {
            redactor,
            records: Mutex::new(Vec::new()),
        }
    }

    /// Devolve uma cópia dos registros.
    #[must_use]
    pub fn records(&self) -> Vec<Record> {
        lock_or_recover(&self.records).clone()
    }

    /// `true` se algum registro contém o texto (útil em testes de varredura).
    #[must_use]
    pub fn contains(&self, needle: &str) -> bool {
        lock_or_recover(&self.records)
            .iter()
            .any(|r| r.message.contains(needle))
    }
}

impl Logger for RecordingLogger {
    fn log(&self, record: &LogRecord<'_>) {
        let fields = record
            .fields
            .iter()
            .map(|(k, v)| ((*k).to_string(), self.redactor.redact(v).into_owned()))
            .collect();
        lock_or_recover(&self.records).push(Record {
            level: record.level,
            message: self.redactor.redact(record.message).into_owned(),
            fields,
        });
    }
}

#[cfg(test)]
mod tests;
