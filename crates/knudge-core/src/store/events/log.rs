//! Log de eventos em disco: append, rotação por tamanho, leitura tolerante e checkpoint.

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices/números de linha com domínio limitado ao tamanho do log"
)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::schema::Value;
use crate::{Error, Result};

use super::Event;

/// Resultado de leitura do log: eventos (deduplicados) + warnings tolerados.
pub type EventRead = (Vec<Event>, Vec<String>);

/// Log append-only de eventos com rotação por tamanho.
pub struct EventLog<'a> {
    fs: &'a dyn Fs,
    root: PathBuf,
    max_bytes: u64,
}

impl<'a> EventLog<'a> {
    /// Tamanho padrão do segmento ativo antes de rotacionar (1 MiB).
    pub const DEFAULT_MAX_BYTES: u64 = 1 << 20;

    /// Cria o log com raiz em `.knudge/`.
    #[must_use]
    pub fn new(fs: &'a dyn Fs, root: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self {
            fs,
            root: root.into(),
            max_bytes,
        }
    }

    /// Diretório dos eventos.
    #[must_use]
    pub fn dir(&self) -> PathBuf {
        self.root.join("eventos")
    }

    /// Segmento ativo (`eventos/events.jsonl`).
    #[must_use]
    pub fn active_path(&self) -> PathBuf {
        self.dir().join("events.jsonl")
    }

    /// Caminho do checkpoint (derivado, em `.idx/`).
    #[must_use]
    pub fn checkpoint_path(&self) -> PathBuf {
        self.root.join(".idx").join("events.checkpoint")
    }

    /// Acrescenta um evento, rotacionando o segmento ativo se necessário.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se a escrita falhar.
    pub fn append(&self, event: &Event) -> Result<()> {
        self.fs.create_dir_all(&self.dir())?;
        self.rotate_if_needed()?;
        self.fs
            .append(&self.active_path(), event.to_line()?.as_bytes())
    }

    /// Segmentos em ordem de leitura: rotacionados (crescente) e o ativo por último.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a listagem falhar.
    pub fn segments(&self) -> Result<Vec<PathBuf>> {
        let dir = self.dir();
        if !self.fs.exists(&dir) {
            return Ok(Vec::new());
        }
        let active = self.active_path();
        let mut rotated = Vec::new();
        let mut has_active = false;
        for path in self.fs.list_dir(&dir)? {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name == "events.jsonl" {
                has_active = true;
            } else if name.starts_with("events-")
                && path.extension().and_then(|e| e.to_str()) == Some("jsonl")
            {
                rotated.push(path);
            }
        }
        rotated.sort();
        if has_active {
            rotated.push(active);
        }
        Ok(rotated)
    }

    /// Percorre os eventos em ordem, deduplicados por `id`, sem acumular o log inteiro.
    ///
    /// Retorna os warnings (linhas malformadas/ignoradas).
    ///
    /// # Errors
    /// Retorna erro apenas de I/O catastrófico (listagem); linhas ruins viram warning.
    pub fn for_each<F>(&self, mut visit: F) -> Result<Vec<String>>
    where
        F: FnMut(&Event),
    {
        let mut warnings = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for segment in self.segments()? {
            let bytes = match self.fs.read(&segment) {
                Ok(bytes) => bytes,
                Err(error) => {
                    warnings.push(format!("segmento ilegível {}: {error}", segment.display()));
                    continue;
                }
            };
            let Ok(text) = std::str::from_utf8(&bytes) else {
                warnings.push(format!("segmento não é UTF-8: {}", segment.display()));
                continue;
            };
            for (index, line) in jsonl::lines(text).enumerate() {
                let number = index + 1;
                match json::decode(line) {
                    Ok(value) => match Event::from_value(&value) {
                        Ok(event) => {
                            let id = match Event::stored_id(&value) {
                                Some(id) => id.to_string(),
                                None => event.id()?,
                            };
                            if seen.insert(id) {
                                visit(&event);
                            }
                        }
                        Err(error) => warnings.push(format!(
                            "evento inválido {}:{number}: {error}",
                            segment.display()
                        )),
                    },
                    Err(error) => warnings.push(format!(
                        "linha inválida {}:{number}: {error}",
                        segment.display()
                    )),
                }
            }
        }
        Ok(warnings)
    }

    /// Lê todos os eventos (deduplicados) e os warnings.
    ///
    /// # Errors
    /// Propaga erro de I/O catastrófico.
    pub fn read_all(&self) -> Result<EventRead> {
        let mut events = Vec::new();
        let warnings = self.for_each(|event| events.push(event.clone()))?;
        Ok((events, warnings))
    }

    /// Eventos de uma nota específica (histórico auditável — E03-T05).
    ///
    /// # Errors
    /// Propaga erro de I/O catastrófico.
    pub fn history(&self, note_id: &str) -> Result<Vec<Event>> {
        let (events, _warnings) = self.read_all()?;
        Ok(events
            .into_iter()
            .filter(|event| event.note_id.as_deref() == Some(note_id))
            .collect())
    }

    /// Marca o último evento já processado por consumidores (checkpoint derivado).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se a escrita falhar.
    pub fn mark_checkpoint(&self, event_id: &str) -> Result<()> {
        let path = self.checkpoint_path();
        if let Some(parent) = path.parent() {
            self.fs.create_dir_all(parent)?;
        }
        let value = Value::map([("id".to_string(), Value::Str(event_id.to_string()))]);
        self.fs
            .write_atomic(&path, json::encode(&value)?.as_bytes())
    }

    /// Eventos após o checkpoint (ou todos, se o checkpoint não estiver no log).
    ///
    /// # Errors
    /// Propaga erro de I/O catastrófico.
    pub fn read_since_checkpoint(&self) -> Result<EventRead> {
        let last = self.read_checkpoint_id()?;
        let Some(last) = last else {
            return self.read_all();
        };
        let mut events = Vec::new();
        let mut started = false;
        let warnings = self.for_each(|event| {
            if started {
                events.push(event.clone());
            } else if event.id().is_ok_and(|id| id == last) {
                started = true;
            }
        })?;
        if started {
            Ok((events, warnings))
        } else {
            // Checkpoint ausente (segmento rotacionado/compactado): fail-safe, devolve tudo.
            self.read_all()
        }
    }

    fn read_checkpoint_id(&self) -> Result<Option<String>> {
        let path = self.checkpoint_path();
        if !self.fs.exists(&path) {
            return Ok(None);
        }
        let bytes = self.fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input("checkpoint não é UTF-8"))?;
        let value = json::decode(text)?;
        Ok(Event::stored_id(&value).map(str::to_string))
    }

    fn rotate_if_needed(&self) -> Result<()> {
        let active = self.active_path();
        if !self.fs.exists(&active) {
            return Ok(());
        }
        let size = u64::try_from(self.fs.read(&active)?.len()).unwrap_or(u64::MAX);
        if size < self.max_bytes {
            return Ok(());
        }
        let index = self.next_segment_index()?;
        let target = self.dir().join(format!("events-{index:04}.jsonl"));
        self.fs.rename(&active, &target)
    }

    fn next_segment_index(&self) -> Result<u32> {
        let mut max = 0_u32;
        for path in self.fs.list_dir(&self.dir())? {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(rest) = name.strip_prefix("events-") else {
                continue;
            };
            let Some(digits) = rest.strip_suffix(".jsonl") else {
                continue;
            };
            if let Ok(index) = digits.parse::<u32>() {
                max = max.max(index);
            }
        }
        Ok(max.saturating_add(1))
    }
}
