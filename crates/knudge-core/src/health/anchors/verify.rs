//! Refresh e verify-on-hit das âncoras (D86, E09-T06).

use std::collections::BTreeSet;
use std::path::Path;

use crate::ports::Fs;
use crate::schema::hash;
use crate::store::{Note, Store};
use crate::{Error, Result};

use super::store::{AnchorRecord, AnchorRole, AnchorStore, StaleAnchor, StaleReason};
use crate::health::tolerant::read_tolerant;

/// `true` se a âncora contém metacaracteres de glob.
#[must_use]
pub fn is_glob(anchor: &str) -> bool {
    anchor.contains(['*', '?', '['])
}

/// Papel derivado da âncora: `cited` se o caminho aparece no corpo; senão `context`.
#[must_use]
pub fn anchor_role(note: &Note, anchor: &str) -> AnchorRole {
    if is_glob(anchor) {
        return AnchorRole::Context;
    }
    let basename = anchor.rsplit('/').next().unwrap_or(anchor);
    if note.body.contains(anchor) || (!basename.is_empty() && note.body.contains(basename)) {
        AnchorRole::Cited
    } else {
        AnchorRole::Context
    }
}

/// Hash `hex8` do conteúdo de um arquivo ancorado (`None` se ausente).
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de leitura que não seja ausência.
pub fn hash_file(fs: &dyn Fs, path: &Path) -> Result<Option<String>> {
    match fs.read(path) {
        Ok(bytes) => Ok(Some(hash::hex8(&bytes))),
        // Ausência ou caminho de diretório não têm conteúdo para hashear (D86).
        Err(error) if is_absent(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

/// `true` se o erro significa "sem arquivo legível" (ausente ou diretório).
fn is_absent(error: &Error) -> bool {
    match error {
        Error::NotFound(_) => true,
        Error::Io { source, .. } => matches!(
            source.kind(),
            std::io::ErrorKind::NotFound | std::io::ErrorKind::IsADirectory
        ),
        _ => false,
    }
}

/// Recalcula e persiste os hashes das âncoras literais de todas as notas.
///
/// Devolve quantas notas tiveram os registros **alterados** (idempotente).
///
/// # Errors
/// Propaga erros de I/O de listagem/leitura/escrita.
pub fn refresh(
    fs: &dyn Fs,
    project_root: &Path,
    store: &Store<'_>,
    anchor_store: &AnchorStore<'_>,
) -> Result<usize> {
    let read = read_tolerant(store)?;
    let mut changed = 0_usize;
    for note in &read.notes {
        let id = note.id()?;
        let anchors = note.frontmatter.string_list("anchors")?;
        let mut records = Vec::new();
        for anchor in anchors {
            if is_glob(anchor) {
                continue;
            }
            let Some(content_hash) = hash_file(fs, &project_root.join(anchor))? else {
                continue;
            };
            records.push(AnchorRecord {
                id: id.to_string(),
                path: anchor.to_string(),
                content_hash,
                role: anchor_role(note, anchor),
            });
        }
        if anchor_store.replace(id, &records)? {
            changed = changed.saturating_add(1);
        }
    }
    Ok(changed)
}

/// Verifica os hashes vigentes, devolvendo as âncoras stale.
///
/// # Errors
/// Propaga erros de I/O que não sejam ausência de arquivo.
pub fn verify(
    fs: &dyn Fs,
    project_root: &Path,
    anchor_store: &AnchorStore<'_>,
) -> Result<Vec<StaleAnchor>> {
    let mut stale = Vec::new();
    for record in anchor_store.list()? {
        match hash_file(fs, &project_root.join(&record.path))? {
            None => stale.push(StaleAnchor {
                id: record.id,
                path: record.path,
                role: record.role,
                reason: StaleReason::Missing,
            }),
            Some(current) if current != record.content_hash => stale.push(StaleAnchor {
                id: record.id,
                path: record.path,
                role: record.role,
                reason: StaleReason::ContentChanged,
            }),
            Some(_) => {}
        }
    }
    Ok(stale)
}

/// Ids de notas **inválidadas** por âncora stale (papel `cited` — D86).
#[must_use]
pub fn invalidated_notes(stale: &[StaleAnchor]) -> Vec<String> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for anchor in stale.iter().filter(|anchor| anchor.invalidates()) {
        ids.insert(anchor.id.clone());
    }
    ids.into_iter().collect()
}
