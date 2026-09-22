//! `context_id` endereçável e handoff 1:1 (D88).
//!
//! O contexto é **derivado**: seu id é o hash do texto renderizado. Guardar o texto em
//! `.idx/contexts/<id>` (derivado, reconstruível) permite que `rewind --resume <id>` devolva
//! **bytes idênticos** sem re-busca.

use std::path::PathBuf;

use crate::ports::Fs;
use crate::schema::hash;
use crate::{Error, Result};

/// Prefixo do `context_id`.
pub const CONTEXT_PREFIX: &str = "ctx";

/// Deriva o `context_id` do texto (determinístico).
#[must_use]
pub fn derive_id(text: &str) -> String {
    format!(
        "{CONTEXT_PREFIX}_{}",
        hash::base36_8(hash::short_hash(text.as_bytes()))
    )
}

/// `true` se o texto é um `context_id` bem formado.
#[must_use]
pub fn is_valid_context_id(value: &str) -> bool {
    let Some((prefix, suffix)) = value.split_once('_') else {
        return false;
    };
    prefix == CONTEXT_PREFIX
        && suffix.len() == 8
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
}

/// Store derivado de contextos renderizados.
pub struct ContextStore<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
}

impl<'a> ContextStore<'a> {
    /// Cria o store com raiz em `.knudge/`.
    #[must_use]
    pub fn new(fs: &'a dyn Fs, root: impl Into<PathBuf>) -> Self {
        Self {
            fs,
            root: root.into(),
        }
    }

    /// Diretório dos contextos.
    #[must_use]
    pub fn dir(&self) -> PathBuf {
        self.root.join(".idx").join("contexts")
    }

    /// Caminho de um contexto.
    #[must_use]
    pub fn path(&self, context_id: &str) -> PathBuf {
        self.dir().join(format!("{context_id}.txt"))
    }

    /// Grava o contexto derivado.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, context_id: &str, text: &str) -> Result<()> {
        self.fs.create_dir_all(&self.dir())?;
        self.fs
            .write_atomic(&self.path(context_id), text.as_bytes())
    }

    /// Lê o contexto, se existir.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se o arquivo existir e não for legível/UTF-8.
    pub fn load(&self, context_id: &str) -> Result<Option<String>> {
        let path = self.path(context_id);
        if !self.fs.exists(&path) {
            return Ok(None);
        }
        let bytes = self.fs.read(&path)?;
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| Error::invalid_input(format!("contexto não é UTF-8: {}", path.display())))
    }
}
