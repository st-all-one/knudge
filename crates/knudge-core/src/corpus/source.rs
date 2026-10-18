//! Fonte de notas do corpus: `notas/` (canónica) ou um JSONL externo **read-only** (D218).
//!
//! A `Source` é a **porta de entrada** para agentes que mantêm a memória num store próprio — um
//! JSONL de linhas `{"frontmatter": {…}, "body": "…"}`, o codec de [`Note::to_jsonl`]. O knudge
//! **lê** essa fonte e deriva índice/grafo; **nunca** lhe escreve (uma só verdade: o store é do
//! agente, o índice é derivado). O contrato é o **documento** (o schema do knudge) — não o schema
//! interno do agente, que fica do lado dele.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::jsonl;
use crate::ports::Fs;
use crate::store::{Note, Store};

use super::Corpus;

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
}

impl Source {
    /// Fonte JSONL num caminho.
    #[must_use]
    pub fn jsonl(path: impl Into<PathBuf>) -> Self {
        Self::Jsonl(path.into())
    }

    /// Caminho da fonte JSONL, se for dessa espécie.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::NotesDir => None,
            Self::Jsonl(path) => Some(path),
        }
    }

    /// `true` quando lê `notas/` (a fonte canónica do knudge).
    #[must_use]
    pub fn is_notes_dir(&self) -> bool {
        matches!(self, Self::NotesDir)
    }

    /// Lê as notas da fonte: **dedup-on-read** por `id` (a última linha vence, D26) e linha
    /// malformada vira *skip + warning* (D18) — nunca derruba a leitura.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura da fonte `NotesDir` (o JSONL é tolerante por contrato).
    pub fn load_notes(&self, store: &Store<'_>) -> Result<Loaded> {
        match self {
            Self::NotesDir => Ok((Corpus::load_notes(store)?, Vec::new())),
            Self::Jsonl(path) => Ok(read_jsonl(store.fs(), path)),
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

/// Lê notas de um **texto** JSONL: dedup-on-read por `id` (a última linha vence — D26) e linha
/// malformada = *skip + warning* (D18), em ordem determinística por `id`.
///
/// É a parte **pura** de [`Source::load_notes`]: quem já tem a sua própria porta de FS partilha as
/// mesmas regras sem reimplementar o codec nem a tolerância.
#[must_use]
pub fn parse_jsonl(text: &str) -> Loaded {
    let mut notes: Vec<Note> = Vec::new();
    let mut warnings = Vec::new();
    for (index, line) in jsonl::lines(text).enumerate() {
        match Note::from_jsonl(line) {
            Ok(note) => upsert(&mut notes, note),
            Err(error) => warnings.push(format!(
                "linha {} ignorada: {error}",
                index.saturating_add(1)
            )),
        }
    }
    // Ordem determinística (`Index::build` reordena por `id`; isto mantém a leitura estável).
    notes.sort_by(|a, b| a.id().unwrap_or("").cmp(b.id().unwrap_or("")));
    (notes, warnings)
}

/// Lê o JSONL de notas. Ficheiro ausente ⇒ corpus vazio (sem erro nem aviso).
fn read_jsonl(fs: &dyn Fs, path: &Path) -> Loaded {
    let mut warnings = Vec::new();
    if !fs.exists(path) {
        return (Vec::new(), warnings);
    }
    let Ok(bytes) = fs.read(path) else {
        warnings.push(format!("fonte ilegível: {}", path.display()));
        return (Vec::new(), warnings);
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        warnings.push(format!("fonte não é UTF-8: {}", path.display()));
        return (Vec::new(), warnings);
    };
    parse_jsonl(text)
}

/// Insere a nota substituindo a de mesmo `id` (a última versão vence — D26).
fn upsert(notes: &mut Vec<Note>, note: Note) {
    let id = note.id().unwrap_or_default().to_string();
    if let Some(slot) = notes
        .iter_mut()
        .find(|item| item.id().unwrap_or_default() == id)
    {
        *slot = note;
        return;
    }
    notes.push(note);
}
