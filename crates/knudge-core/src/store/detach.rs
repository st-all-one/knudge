//! Limpeza referencial na remoção de uma nota (D46/D84).
//!
//! `notas/` é a verdade: ao remover um id, nenhuma outra nota pode continuar apontando para
//! ele. A limpeza reescreve as referrers (arestas explícitas e `superseded_by`) **antes** de o
//! arquivo sair, para que a integridade do grafo (D46) permaneça limpa.

use std::path::Path;

use crate::Result;
use crate::ports::Fs;
use crate::schema::{EdgeKind, Frontmatter, Value};

use super::Store;

/// Remove `removed` de todas as arestas e de `superseded_by` das demais notas.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Schema` para falha de listagem/leitura/escrita/validação. Notas
/// ilegíveis são puladas — o `doctor` as reporta no check de schema.
pub(super) fn detach_referrers(fs: &dyn Fs, root: &Path, removed: &str) -> Result<()> {
    let store = Store::new(fs, root);
    for id in store.list_ids()? {
        if id == removed {
            continue;
        }
        let Ok(mut note) = store.read(&id) else {
            continue;
        };
        if !strip(&mut note.frontmatter, removed)? {
            continue;
        }
        let revision = note.revision().saturating_add(1);
        note.set_revision(revision)?;
        note.refresh_body_hash()?;
        note.frontmatter.validate()?;
        store.write(&note)?;
    }
    Ok(())
}

/// Retira `removed` das arestas de `frontmatter`; retorna `true` se algo mudou.
fn strip(frontmatter: &mut Frontmatter, removed: &str) -> Result<bool> {
    let mut changed = false;
    for kind in EdgeKind::ALL {
        let key = kind.key();
        let targets: Vec<String> = frontmatter
            .string_list(key)?
            .into_iter()
            .map(str::to_string)
            .collect();
        if !targets.iter().any(|target| target == removed) {
            continue;
        }
        let kept: Vec<Value> = targets
            .into_iter()
            .filter(|target| target != removed)
            .map(Value::Str)
            .collect();
        if kept.is_empty() {
            frontmatter.remove(key);
        } else {
            frontmatter.set(key, Value::List(kept))?;
        }
        changed = true;
    }
    if frontmatter.get("superseded_by").and_then(Value::as_str) == Some(removed) {
        frontmatter.remove("superseded_by");
        changed = true;
    }
    Ok(changed)
}
