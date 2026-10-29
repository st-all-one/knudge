//! Fonte de notas do corpus: `notas/` (canónica) ou JSONL/NDJSON externo **read-only** (D218/D220).
//!
//! A `Source` é a **porta de entrada** para agentes que mantêm a memória num store próprio — um
//! ficheiro JSONL ou um **diretório de shards** (`<agent_id>.jsonl`), sempre com linhas
//! `{"frontmatter": {…}, "body": "…"}`, o codec de [`Note::to_jsonl`]. O knudge **lê** essa fonte e
//! deriva índice/grafo; **nunca** lhe escreve (uma só verdade: o store é do agente, o índice é
//! derivado). O contrato é o **documento** (o schema do knudge) — não o schema interno do agente.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::ports::Fs;
use crate::store::{Note, Store};

use super::Corpus;

mod jsonl;

pub use jsonl::{parse_jsonl, shard_file_name};

/// Notas lidas de uma fonte + avisos (linha malformada ignorada — D18).
pub type Loaded = (Vec<Note>, Vec<String>);

/// De onde o corpus é lido.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Source {
    /// `notas/<tipo>/<id>.md` — a verdade do knudge (default).
    #[default]
    NotesDir,
    /// JSONL de notas (`{frontmatter, body}` por linha), lido **read-only**.
    Jsonl(PathBuf),
    /// Diretório de shards JSONL (um por `agent_id`), lido **read-only** (D220).
    JsonlDir(PathBuf),
}

impl Source {
    /// Fonte JSONL num caminho de ficheiro.
    #[must_use]
    pub fn jsonl(path: impl Into<PathBuf>) -> Self {
        Self::Jsonl(path.into())
    }

    /// Fonte de shards num caminho de diretório.
    #[must_use]
    pub fn jsonl_dir(path: impl Into<PathBuf>) -> Self {
        Self::JsonlDir(path.into())
    }

    /// Escolhe entre ficheiro e diretório pela porta de FS (D220).
    ///
    /// Um caminho inexistente é tratado como ficheiro (o `--source` aceita ambos os tipos e uma
    /// fonte ausente continua a ser corpus vazio, sem erro).
    #[must_use]
    pub fn at(fs: &dyn Fs, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        if fs.is_dir(&path) {
            Self::JsonlDir(path)
        } else {
            Self::Jsonl(path)
        }
    }

    /// Caminho da fonte externa, se for dessa espécie.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::NotesDir => None,
            Self::Jsonl(path) | Self::JsonlDir(path) => Some(path),
        }
    }

    /// `true` quando a fonte é um diretório de shards.
    #[must_use]
    pub fn is_jsonl_dir(&self) -> bool {
        matches!(self, Self::JsonlDir(_))
    }

    /// `true` quando lê `notas/` (a fonte canónica do knudge).
    #[must_use]
    pub fn is_notes_dir(&self) -> bool {
        matches!(self, Self::NotesDir)
    }

    /// Lê as notas da fonte.
    ///
    /// JSONL (ficheiro/diretório): **dedup-on-read** por `id` (a última linha do shard vence —
    /// D26/D223) e linha malformada vira *skip + warning* (D18) — nunca derruba a leitura.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura da fonte `NotesDir` (o JSONL é tolerante por contrato).
    pub fn load_notes(&self, store: &Store<'_>) -> Result<Loaded> {
        match self {
            Self::NotesDir => Ok((Corpus::load_notes(store)?, Vec::new())),
            Self::Jsonl(path) => Ok(jsonl::read_file(store.fs(), path)),
            Self::JsonlDir(dir) => Ok(jsonl::read_dir(store.fs(), dir)),
        }
    }

    /// Notas + índice + grafo da fonte, numa só leitura.
    ///
    /// # Errors
    /// Propaga erros de leitura da fonte e de derivação do índice/grafo.
    pub fn load(&self, store: &Store<'_>) -> Result<(Corpus, Vec<String>)> {
        let (notes, warnings) = self.load_notes(store)?;
        Ok((Corpus::from_notes(notes)?, warnings))
    }
}
