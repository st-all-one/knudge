//! Executor de hooks externos com timeout e kill do grupo de processos (E12-T04, D59).
//!
//! Sem shell (R12): o comando é dividido em programa + argumentos e executado diretamente. O
//! hook recebe o payload JSON em **stdin**; stdout é a resposta. Um hook que excede o timeout é
//! morto **junto com seus filhos** (`process_group(0)` + `kill -- -PGID`).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::ports::{HookOutput, HookRunner};
use crate::{Error, Result};

/// Timeout default de um hook (ms).
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Executa hooks via `std::process::Command` (sem shell).
#[derive(Debug, Clone)]
pub struct ProcessHookRunner {
    cwd: PathBuf,
    timeout: Duration,
}

impl ProcessHookRunner {
    /// Cria o executor com diretório de trabalho e timeout.
    #[must_use]
    pub fn new(cwd: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            cwd: cwd.into(),
            timeout,
        }
    }

    /// Executor com timeout default.
    #[must_use]
    pub fn with_default_timeout(cwd: impl Into<PathBuf>) -> Self {
        Self::new(cwd, Duration::from_millis(DEFAULT_TIMEOUT_MS))
    }

    /// Diretório de trabalho.
    #[must_use]
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
}

impl HookRunner for ProcessHookRunner {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador real: mede o timeout do processo externo (R01/R43)"
    )]
    fn run(&self, hook: &str, input: &[u8]) -> Result<HookOutput> {
        let mut parts = hook.split_whitespace();
        let program = parts.next().ok_or_else(|| Error::config("hook vazio"))?;
        let mut command = Command::new(program);
        command
            .args(parts)
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command.spawn().map_err(|error| Error::io(hook, error))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input)
                .map_err(|error| Error::io(hook, error))?;
        }
        let stdout = child.stdout.take().map(spawn_reader);
        let stderr = child.stderr.take().map(spawn_reader);
        let deadline = Instant::now()
            .checked_add(self.timeout)
            .unwrap_or_else(Instant::now);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        kill_group(&mut child);
                        return Err(Error::timeout(format!(
                            "hook excedeu {}ms: {hook}",
                            self.timeout.as_millis()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(error) => return Err(Error::io(hook, error)),
            }
        };
        Ok(HookOutput {
            status: status.code().unwrap_or(-1),
            stdout: join_reader(stdout),
            stderr: join_reader(stderr),
        })
    }
}

type Reader = std::thread::JoinHandle<std::io::Result<Vec<u8>>>;

fn spawn_reader<R: Read + Send + 'static>(mut reader: R) -> Reader {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    })
}

fn join_reader(reader: Option<Reader>) -> Vec<u8> {
    match reader {
        Some(handle) => handle
            .join()
            .ok()
            .and_then(std::result::Result::ok)
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

#[allow(
    clippy::disallowed_methods,
    reason = "adaptador real: mata o grupo de processos do hook no timeout (R01/R43)"
)]
fn kill_group(child: &mut Child) {
    let pid = child.id();
    let _ignored = Command::new("kill")
        .arg("-TERM")
        .arg(format!("-{pid}"))
        .status();
    let _ignored = child.kill();
    let _ignored = child.wait();
}

#[cfg(test)]
mod tests;
