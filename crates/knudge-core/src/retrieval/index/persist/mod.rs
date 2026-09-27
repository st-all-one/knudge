//! Persistência do índice derivado: serialização canônica e carga tolerante (E06-T01).

use std::path::{Path, PathBuf};

use crate::jsonl::{self, json};
use crate::ports::Fs;
use crate::store::Store;
use crate::{Error, Result};

use super::{INDEX_FILE, INDEX_FORMAT, INDEX_WARN_BYTES, Index, compute_stats};

mod codec;

use codec::{doc_from_value, doc_to_value};

impl Index {
    /// Caminho do arquivo do índice.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(".idx").join(INDEX_FILE)
    }

    /// Serializa o índice em JSONL canônico: cabeçalho de formato + uma linha por documento.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se algum float não for finito.
    pub fn serialize(&self) -> Result<String> {
        let mut out = String::new();
        out.push_str(INDEX_FORMAT);
        out.push('\n');
        for doc in &self.docs {
            out.push_str(&json::encode(&doc_to_value(doc))?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Lê o índice de UM texto JSONL e recomputa as estatísticas.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se o cabeçalho de formato for diferente (índice antigo,
    /// D172) e `ErrorKind::InvalidInput`/`Schema` para linha malformada.
    pub fn parse(text: &str) -> Result<Self> {
        let mut lines = jsonl::lines(text);
        if lines.next() != Some(INDEX_FORMAT) {
            return Err(Error::schema(
                "índice em formato antigo (pré-D172); reconstruir a partir de notas/",
            ));
        }
        let mut docs = Vec::new();
        for line in lines {
            docs.push(doc_from_value(&json::decode(line)?)?);
        }
        docs.sort_unstable_by(|a, b| a.meta.id.cmp(&b.meta.id));
        let stats = compute_stats(&docs);
        Ok(Self::from_parts(docs, stats))
    }

    /// Grava o índice atomicamente (`tmp + rename`), avisando se ultrapassar o teto.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, fs: &dyn Fs, root: &Path, warnings: &mut Vec<String>) -> Result<()> {
        let path = Self::path(root);
        if let Some(parent) = path.parent() {
            fs.create_dir_all(parent)?;
        }
        let data = self.serialize()?;
        if let Some(warning) = size_warning(data.len()) {
            warnings.push(warning);
        }
        fs.write_atomic(&path, data.as_bytes())
    }

    /// Carrega o índice, se existir e for legível.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`InvalidInput` se a leitura/parse falhar.
    pub fn load(fs: &dyn Fs, root: &Path, warnings: &mut Vec<String>) -> Result<Option<Self>> {
        let path = Self::path(root);
        if !fs.exists(&path) {
            return Ok(None);
        }
        let bytes = fs.read(&path)?;
        if let Some(warning) = size_warning(bytes.len()) {
            warnings.push(warning);
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_input(format!("{} não é UTF-8", path.display())))?;
        if text.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(Self::parse(text)?))
    }

    /// Abre o índice, reconstruindo e gravando se estiver ausente, **desatualizado** ou
    /// ilegível.
    ///
    /// A frescura é decidida por `mtime` (E15-T11/O1.6): o índice só é servido se for mais novo
    /// que todas as notas; caso contrário, reconstrói a partir de `notas/`.
    ///
    /// # Errors
    /// Propaga erros de leitura/parse/construção/escrita.
    pub fn open(fs: &dyn Fs, root: &Path, store: &Store<'_>) -> Result<(Self, Vec<String>)> {
        let mut warnings = Vec::new();
        let existed = fs.exists(&Self::path(root));
        let before = warnings.len();
        if let Some(index) = Self::load_if_fresh(fs, root, store, &mut warnings)? {
            return Ok((index, warnings));
        }
        let warned_illegible = warnings.len() > before;
        let index = Self::from_store(store)?;
        index.save(fs, root, &mut warnings)?;
        if !warned_illegible {
            warnings.push(if existed {
                "índice desatualizado: reconstruído a partir de notas/".to_string()
            } else {
                "índice ausente: reconstruído a partir de notas/".to_string()
            });
        }
        Ok((index, warnings))
    }

    /// Carrega o índice persistido **se fresco** (`mtime` ≥ o de todas as notas); `None` se
    /// ausente, desatualizado ou ilegível (nunca serve um índice parcial/obsoleto — E13-T03).
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura; erro de parse vira `None` + aviso.
    pub fn load_if_fresh(
        fs: &dyn Fs,
        root: &Path,
        store: &Store<'_>,
        warnings: &mut Vec<String>,
    ) -> Result<Option<Self>> {
        if !is_fresh(fs, root, store)? {
            return Ok(None);
        }
        match Self::load(fs, root, warnings) {
            Ok(index) => Ok(index),
            Err(error) => {
                warnings.push(format!(
                    "índice derivado ilegível; reconstruído a partir de notas/: {error}"
                ));
                Ok(None)
            }
        }
    }
}

/// `true` se o índice persistido é mais novo que todas as notas (invalidação por `mtime`).
///
/// Se o índice não existir, o `mtime` não estiver disponível (FS sem suporte) ou qualquer nota
/// não puder ser inspecionada, devolve `false` — reconstruir é sempre seguro (D15/D84).
fn is_fresh(fs: &dyn Fs, root: &Path, store: &Store<'_>) -> Result<bool> {
    let path = Index::path(root);
    if !fs.exists(&path) {
        return Ok(false);
    }
    let Some(index_mtime) = fs.modified_ms(&path)? else {
        return Ok(false);
    };
    let mut newest = i64::MIN;
    for id in store.list_ids()? {
        match fs.modified_ms(&store.resolve_path(&id)) {
            Ok(Some(note_mtime)) => newest = newest.max(note_mtime),
            // Nota removida entre a listagem e o `stat` (ou FS sem `mtime`): reconstrói.
            Ok(None) | Err(_) => return Ok(false),
        }
    }
    Ok(index_mtime >= newest)
}

/// Aviso quando o índice passa do teto de [`INDEX_WARN_BYTES`] (E06-T07).
#[must_use]
pub fn size_warning(len: usize) -> Option<String> {
    let limit = usize::try_from(INDEX_WARN_BYTES).unwrap_or(usize::MAX);
    (len > limit).then(|| {
        format!("índice de {len} bytes acima do teto de {INDEX_WARN_BYTES}; considere rebuild")
    })
}
