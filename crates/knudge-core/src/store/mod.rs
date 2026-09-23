//! Escopo `store`: persistência crash-safe e concorrente-segura (E03).
//!
//! `notas/` é a verdade; eventos são auditoria; `.idx/` é derivado e **nunca** fonte da
//! verdade. Toda escrita canônica é atômica (tmp + rename — D20) e vem **antes** do evento
//! correspondente (D21). O lock advisory (D23–D25) e a limpeza RAII (R05/R10) protegem dois
//! processos (CLI + MCP).

pub mod commit;
pub mod events;
pub mod lock;
pub mod note;
pub mod purge;
pub mod rebuild;
pub mod sweep;

mod detach;

#[cfg(test)]
mod tests;

pub use commit::commit;
pub use events::{Event, EventLog};
pub use lock::{LockGuard, LockPolicy, acquire};
pub use note::Note;
pub use purge::purge_derived;
pub use rebuild::Staging;
pub use sweep::sweep_residues;

use std::path::{Path, PathBuf};

use crate::Error;
use crate::Result;
use crate::ports::Fs;

/// Store de notas sobre uma porta [`Fs`].
pub struct Store<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> Store<'a> {
    /// Cria um store com raiz em `.knudge/`.
    #[must_use]
    pub fn new(fs: &'a dyn Fs, root: impl Into<PathBuf>) -> Self {
        Self {
            fs,
            root: root.into(),
        }
    }

    /// Raiz do `.knudge/`.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Porta de FS usada pelo store.
    #[must_use]
    pub fn fs(&self) -> &dyn Fs {
        self.fs
    }

    /// Diretório de notas.
    #[must_use]
    pub fn notes_dir(&self) -> PathBuf {
        self.root.join("notas")
    }

    /// Caminho canônico de uma nota.
    #[must_use]
    pub fn note_path(&self, id: &str) -> PathBuf {
        self.notes_dir().join(format!("{id}.md"))
    }

    /// Garante a existência do diretório de notas.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a criação falhar.
    pub fn ensure_dirs(&self) -> Result<()> {
        self.fs.create_dir_all(&self.notes_dir())
    }

    /// `true` se a nota existe.
    #[must_use]
    pub fn exists(&self, id: &str) -> bool {
        self.fs.exists(&self.note_path(id))
    }

    /// Lê e valida uma nota.
    ///
    /// # Errors
    /// Retorna `ErrorKind::NotFound` se a nota não existir, e `ErrorKind::Io`/`Schema`
    /// conforme o caso.
    pub fn read(&self, id: &str) -> Result<Note> {
        let path = self.note_path(id);
        if !self.fs.exists(&path) {
            return Err(Error::not_found(format!("nota ausente: {id}")));
        }
        let bytes = self.fs.read(&path)?;
        Note::parse(&bytes)
    }

    /// Grava uma nota atomicamente (sem `fsync`; ver [`Store::sync`]).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`Schema`.
    pub fn write(&self, note: &Note) -> Result<()> {
        self.ensure_dirs()?;
        let path = self.note_path(note.id()?);
        self.fs.write_atomic(&path, note.render().as_bytes())
    }

    /// Força a nota (e o diretório) para o disco — o `fsync` em batch (D22).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se o `fsync` falhar.
    pub fn sync(&self, id: &str) -> Result<()> {
        self.fs.sync(&self.note_path(id))?;
        let dir = self.notes_dir();
        if self.fs.is_dir(&dir) {
            self.fs.sync(&dir)?;
        }
        Ok(())
    }

    /// Lista os ids das notas presentes (ordenados).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a listagem falhar.
    pub fn list_ids(&self) -> Result<Vec<String>> {
        let dir = self.notes_dir();
        if !self.fs.exists(&dir) {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for path in self.fs.list_dir(&dir)? {
            let is_markdown = path.extension().and_then(|e| e.to_str()) == Some("md");
            if is_markdown && let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                ids.push(stem.to_string());
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// Aplica uma mutação, incrementa `revision`, recalcula `body_hash` e grava.
    ///
    /// # Errors
    /// Propaga erros de leitura, da mutação e da escrita.
    pub fn update<F>(&self, id: &str, mutate: F) -> Result<u32>
    where
        F: FnOnce(&mut Note) -> Result<()>,
    {
        let mut note = self.read(id)?;
        mutate(&mut note)?;
        let revision = note.revision().saturating_add(1);
        note.set_revision(revision)?;
        note.refresh_body_hash()?;
        self.write(&note)?;
        Ok(revision)
    }

    /// Remove a nota (canônico **primeiro**) e purga o derivado (D84).
    ///
    /// Antes de apagar o arquivo, remove as referências de entrada (arestas explícitas e
    /// `superseded_by`) para não deixar arestas penduradas (D46).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se remoção/purga falharem.
    pub fn remove(&self, id: &str) -> Result<()> {
        detach::detach_referrers(self.fs, &self.root, id)?;
        self.fs.remove_file(&self.note_path(id))?;
        purge_derived(self.fs, &self.root, id)
    }
}
