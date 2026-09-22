//! Lock advisory por arquivo-alvo (D23–D25) com liberação RAII (R05).
//!
//! Regras:
//! - Aquisição por criação **exclusiva** (`O_CREAT|O_EXCL`) do arquivo de lock.
//! - Conteúdo `{"at": <ms>}` permite detectar **stale** (padrão 30 s).
//! - Reclaim **nunca apaga lock alheio**: renomeia para um sidecar (claim atômico) e só então
//!   remove.
//! - Retry com jitter limitado; sem jitter (`0`), a segunda tentativa já desiste.
//! - Ordem documentada: **externo = container, interno = nota** (evita ABBA — D25).

#![allow(
    clippy::arithmetic_side_effects,
    reason = "cálculo de idade/jitter com domínio de tempo limitado"
)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::jsonl::json;
use crate::ports::{Clock, Fs, Rng};
use crate::schema::Value;
use crate::{Error, Result};

/// Política de aquisição do lock.
#[derive(Debug, Clone, Copy)]
pub struct LockPolicy {
    /// Idade (ms) a partir da qual um lock é considerado abandonado.
    pub stale_ms: i64,
    /// Número de tentativas antes de desistir.
    pub retries: u32,
    /// Jitter máximo (ms) entre tentativas; `0` desativa o `sleep`.
    pub jitter_ms: u64,
}

impl Default for LockPolicy {
    fn default() -> Self {
        Self {
            stale_ms: 30_000,
            retries: 50,
            jitter_ms: 20,
        }
    }
}

/// Guarda RAII: remove o arquivo de lock ao ser descartado.
pub struct LockGuard<'a> {
    fs: &'a dyn Fs,
    path: PathBuf,
}

impl LockGuard<'_> {
    /// Caminho do arquivo de lock.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard<'_> {
    fn drop(&mut self) {
        let _ignored = self.fs.remove_file(&self.path);
    }
}

/// Adquire o lock em `lock_path`, reclamando locks stale.
///
/// # Errors
/// Retorna `ErrorKind::Conflict` se o lock seguir ocupado após as tentativas.
pub fn acquire<'a>(
    fs: &'a dyn Fs,
    clock: &dyn Clock,
    rng: &mut dyn Rng,
    lock_path: &Path,
    policy: LockPolicy,
) -> Result<LockGuard<'a>> {
    if let Some(parent) = lock_path.parent() {
        fs.create_dir_all(parent)?;
    }
    let mut attempt = 0_u32;
    loop {
        let now = clock.now().as_millis();
        let content = json::encode(&Value::map([("at".to_string(), Value::Int(now))]))?;
        match fs.create_exclusive(lock_path, content.as_bytes()) {
            Ok(()) => {
                return Ok(LockGuard {
                    fs,
                    path: lock_path.to_path_buf(),
                });
            }
            Err(Error::Conflict(_)) => {
                if is_stale(fs, lock_path, now, policy.stale_ms) {
                    reclaim(fs, lock_path)?;
                } else if attempt >= policy.retries {
                    return Err(Error::conflict(format!(
                        "lock ocupado: {}",
                        lock_path.display()
                    )));
                } else {
                    sleep_jitter(rng, policy.jitter_ms);
                }
            }
            Err(error) => return Err(error),
        }
        attempt = attempt.saturating_add(1);
    }
}

/// `true` se o lock pode ser reclamado (idade > `stale_ms`, ou sem conteúdo legível).
///
/// Um lock recém-criado pode ainda não ter conteúdo visível (o `create_exclusive` não é
/// atômico com a escrita do `at`); nesse caso o `mtime` decide. Sem `mtime`, é **conservador**
/// e não reclama — melhor esperar do que roubar um lock vivo (E13-T03).
fn is_stale(fs: &dyn Fs, path: &Path, now: i64, stale_ms: i64) -> bool {
    match read_lock_at(fs, path) {
        Ok(Some(at)) => now.saturating_sub(at) > stale_ms,
        _ => match fs.modified_ms(path) {
            Ok(Some(modified)) => now.saturating_sub(modified) > stale_ms,
            _ => false,
        },
    }
}

/// Reclama o lock via rename para sidecar (claim atômico) e remoção.
fn reclaim(fs: &dyn Fs, path: &Path) -> Result<()> {
    let sidecar = sidecar_path(path);
    match fs.rename(path, &sidecar) {
        Ok(()) => fs.remove_file(&sidecar),
        // Outro processo já reclamou: nada a fazer.
        Err(Error::NotFound(_)) => Ok(()),
        Err(error) => Err(error),
    }
}

fn read_lock_at(fs: &dyn Fs, path: &Path) -> Result<Option<i64>> {
    let bytes = fs.read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::invalid_input("lock não é UTF-8"))?;
    let value = json::decode(text)?;
    Ok(value
        .as_map()
        .and_then(|map| map.get("at"))
        .and_then(Value::as_int))
}

fn sidecar_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsStr::to_os_string)
        .unwrap_or_default();
    name.push(".stale");
    path.with_file_name(name)
}

fn sleep_jitter(rng: &mut dyn Rng, jitter_ms: u64) {
    if jitter_ms == 0 {
        return;
    }
    let delay = rng.next_u64() % jitter_ms;
    std::thread::sleep(std::time::Duration::from_millis(delay));
}
