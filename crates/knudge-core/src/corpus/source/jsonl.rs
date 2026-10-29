//! Leitura tolerante de JSONL/NDJSON: dedup O(N log N) e merge determinístico de shards (D219/D223).
//!
//! - **Ficheiro único**: dedup por `id` com a última linha vencendo (D26).
//! - **Diretório de shards**: cada `<agent_id>.jsonl` é lido em ordem lexicográfica; conflito do
//!   mesmo `id` entre shards resolve por `(created_at, agent_id, ordem do shard)` — sempre
//!   determinístico (D223).
//! - Linha malformada = *skip + warning* com a origem; ficheiro/diretório ausente = corpus vazio.
//! - A saída é ordenada por `id`, o mesmo contrato de `Index::build` e do índice derivado.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::jsonl;
use crate::ports::Fs;
use crate::schema::hash::hex8;
use crate::schema::provenance_from_value;
use crate::store::Note;

use super::Loaded;

/// Extensão dos shards/fontes.
const JSONL_EXT: &str = "jsonl";

/// Teto de caracteres seguros no nome do shard (D220).
const MAX_SHARD_STEM: usize = 96;

/// Teto do prefixo saneado no fallback com hash.
const MAX_SANITIZED: usize = 48;

/// Lê notas de um **texto** JSONL: dedup-on-read por `id` (a última linha vence — D26) e linha
/// malformada = *skip + warning* (D18), em ordem determinística por `id`.
#[must_use]
pub fn parse_jsonl(text: &str) -> Loaded {
    let mut notes = BTreeMap::new();
    let mut warnings = Vec::new();
    parse_into(text, None, &mut notes, &mut warnings);
    (notes.into_values().collect(), warnings)
}

/// Nome de ficheiro determinístico e seguro para o shard de um `agent_id` (D220).
///
/// Um id já seguro (`[A-Za-z0-9._-]`, ≤ 96 chars, ≠ `.`/`..`) vira `<id>.jsonl`; qualquer outro
/// vira `<prefixo-saneado>-<hex8>.jsonl`, onde o hash é do id **original** (sem colisões práticas
/// e sem perda de identidade).
#[must_use]
pub fn shard_file_name(agent_id: &str) -> String {
    if is_safe_stem(agent_id) {
        return format!("{agent_id}.{JSONL_EXT}");
    }
    let mut prefix = String::new();
    for ch in agent_id.chars().take(MAX_SANITIZED) {
        prefix.push(
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            },
        );
    }
    if prefix.is_empty()
        || prefix.contains("..")
        || prefix.chars().all(|ch| matches!(ch, '.' | '-'))
    {
        prefix.clear();
        prefix.push_str("agent");
    }
    format!("{prefix}-{}.{JSONL_EXT}", hex8(agent_id.as_bytes()))
}

fn is_safe_stem(text: &str) -> bool {
    !text.is_empty()
        && text != "."
        && text != ".."
        && text.chars().count() <= MAX_SHARD_STEM
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

/// Lê o JSONL de um ficheiro. Ficheiro ausente ⇒ corpus vazio (sem erro nem aviso).
pub(super) fn read_file(fs: &dyn Fs, path: &Path) -> Loaded {
    let mut warnings = Vec::new();
    let Some(text) = read_text(fs, path, &mut warnings) else {
        return (Vec::new(), warnings);
    };
    let mut notes = BTreeMap::new();
    parse_into(&text, Some(path), &mut notes, &mut warnings);
    (notes.into_values().collect(), warnings)
}

/// Lê todos os `*.jsonl` de um diretório em ordem lexicográfica e mescla os shards (D220/D223).
pub(super) fn read_dir(fs: &dyn Fs, dir: &Path) -> Loaded {
    let mut warnings = Vec::new();
    if !fs.exists(dir) {
        return (Vec::new(), warnings);
    }
    let Ok(entries) = fs.list_dir(dir) else {
        warnings.push(format!("fonte ilegível: {}", dir.display()));
        return (Vec::new(), warnings);
    };
    let mut files: Vec<PathBuf> = entries
        .into_iter()
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some(JSONL_EXT) && !fs.is_dir(path)
        })
        .collect();
    files.sort();
    let mut merged: BTreeMap<String, (Note, String)> = BTreeMap::new();
    for file in files {
        let Some(text) = read_text(fs, &file, &mut warnings) else {
            continue;
        };
        let mut local = BTreeMap::new();
        parse_into(&text, Some(&file), &mut local, &mut warnings);
        let origin = file.display().to_string();
        for (id, note) in local {
            match merged.get(&id) {
                Some((current, current_origin))
                    if !wins(&note, &origin, current, current_origin) => {}
                _ => {
                    let _ignored = merged.insert(id, (note, origin.clone()));
                }
            }
        }
    }
    (
        merged.into_values().map(|(note, _origin)| note).collect(),
        warnings,
    )
}

/// `true` se `candidate` vence `current` no conflito entre shards (D223):
/// maior `created_at`, depois maior `agent_id`, depois maior caminho de shard.
fn wins(candidate: &Note, candidate_origin: &str, current: &Note, current_origin: &str) -> bool {
    let candidate_key = (created_ms(candidate), effective_agent_id(candidate));
    let current_key = (created_ms(current), effective_agent_id(current));
    candidate_key
        .cmp(&current_key)
        .then_with(|| candidate_origin.cmp(current_origin))
        .is_gt()
}

fn created_ms(note: &Note) -> i64 {
    note.frontmatter.created_at().unwrap_or(0)
}

fn effective_agent_id(note: &Note) -> String {
    provenance_from_value(note.frontmatter.get("provenance")).map_or_else(
        |_| "legacy".to_string(),
        |prov| prov.agent_id_or_legacy().to_string(),
    )
}

/// Lê o ficheiro como UTF-8; ausente ⇒ `None` sem aviso; ilegível/não-UTF-8 ⇒ aviso + `None`.
fn read_text(fs: &dyn Fs, path: &Path, warnings: &mut Vec<String>) -> Option<String> {
    if !fs.exists(path) {
        return None;
    }
    let Ok(bytes) = fs.read(path) else {
        warnings.push(format!("fonte ilegível: {}", path.display()));
        return None;
    };
    if let Ok(text) = String::from_utf8(bytes) {
        Some(text)
    } else {
        warnings.push(format!("fonte não é UTF-8: {}", path.display()));
        None
    }
}

/// Acumula as linhas válidas no mapa (a última linha de um mesmo ficheiro vence).
fn parse_into(
    text: &str,
    origin: Option<&Path>,
    notes: &mut BTreeMap<String, Note>,
    warnings: &mut Vec<String>,
) {
    for (index, line) in jsonl::lines(text).enumerate() {
        match Note::from_jsonl(line) {
            Ok(note) => {
                let id = note.frontmatter.id().unwrap_or_default().to_string();
                let _ignored = notes.insert(id, note);
            }
            Err(error) => warnings.push(match origin {
                Some(path) => format!(
                    "{}: linha {} ignorada: {error}",
                    path.display(),
                    index.saturating_add(1)
                ),
                None => format!("linha {} ignorada: {error}", index.saturating_add(1)),
            }),
        }
    }
}
