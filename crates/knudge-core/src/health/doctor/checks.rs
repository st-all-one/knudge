//! Checks individuais do `doctor` (E09-T04).

use std::path::Path;

use crate::Result;
use crate::embeddings::index::WARN_BYTES as EMBEDDINGS_WARN_BYTES;
use crate::embeddings::{CACHE_FILE, EMBEDDINGS_FILE};
use crate::ports::Fs;
use crate::retrieval::Index;
use crate::retrieval::anchor::glob_match;
use crate::schema::NoteType;
use crate::store::Note;
use crate::task::{parent_of, program_of, root_for_path};
use crate::write::dedup::propose_merges;

use super::super::anchors::{AnchorStore, invalidated_notes, verify};
use super::super::audit::stale_lock_paths;
use super::super::tolerant::SkippedNote;
use super::{CheckId, DoctorCheck, DoctorInput, body_hash_mismatch, derived_diverges};

pub(super) fn schema_check(skipped: &[SkippedNote]) -> DoctorCheck {
    let ok = skipped.is_empty();
    let detail = if ok {
        "todas as notas parseiam".to_string()
    } else {
        let ids: Vec<&str> = skipped.iter().map(|note| note.id.as_str()).collect();
        let guidance = skipped.first().map_or("", |note| note.guidance.as_str());
        format!("notas puladas: {}; {guidance}", ids.join(", "))
    };
    DoctorCheck {
        id: CheckId::Schema,
        ok,
        detail,
        fixable: false,
    }
}

pub(super) fn integrity_check(input: &DoctorInput<'_>) -> DoctorCheck {
    let issues = input.graph.integrity();
    DoctorCheck {
        id: CheckId::Integrity,
        ok: issues.is_empty(),
        detail: if issues.is_empty() {
            "grafo íntegro".to_string()
        } else {
            format!(
                "{} problema(s); revise `replaces ↔ superseded_by` e arestas penduradas",
                issues.len()
            )
        },
        fixable: false,
    }
}

pub(super) fn cycles_check(input: &DoctorInput<'_>) -> DoctorCheck {
    let count = input
        .graph
        .supersession_cycles()
        .len()
        .saturating_add(input.graph.dependency_cycles().len());
    DoctorCheck {
        id: CheckId::Cycles,
        ok: count == 0,
        detail: if count == 0 {
            "sem ciclos".to_string()
        } else {
            format!("{count} ciclo(s); quebre a supersessão/dependência")
        },
        fixable: false,
    }
}

pub(super) fn anchors_check(input: &DoctorInput<'_>) -> Result<DoctorCheck> {
    let store = AnchorStore::new(input.fs, input.root);
    let stale = verify(input.fs, input.project_root, &store)?;
    let invalidating = invalidated_notes(&stale);
    Ok(DoctorCheck {
        id: CheckId::Anchors,
        ok: stale.is_empty(),
        detail: if stale.is_empty() {
            "âncoras em dia".to_string()
        } else {
            format!(
                "{} âncora(s) stale; {} nota(s) invalidada(s); `--fix` re-hasheia",
                stale.len(),
                invalidating.len()
            )
        },
        fixable: true,
    })
}

pub(super) fn duplicates_check(expected: &Index, input: &DoctorInput<'_>) -> DoctorCheck {
    let merges = propose_merges(expected, input.thresholds);
    DoctorCheck {
        id: CheckId::Duplicates,
        ok: merges.is_empty(),
        detail: if merges.is_empty() {
            "sem quase-duplicatas".to_string()
        } else {
            format!(
                "{} quase-duplicata(s); use `kd maintenance learn`/`compact`",
                merges.len()
            )
        },
        fixable: false,
    }
}

pub(super) fn locks_check(input: &DoctorInput<'_>) -> Result<DoctorCheck> {
    let paths = stale_lock_paths(input.fs, input.root, input.now_ms, input.lock_stale_ms)?;
    Ok(DoctorCheck {
        id: CheckId::Locks,
        ok: paths.is_empty(),
        detail: if paths.is_empty() {
            "sem locks stale".to_string()
        } else {
            format!("{} lock(s) stale; `--fix` remove", paths.len())
        },
        fixable: true,
    })
}

pub(super) fn config_check(input: &DoctorInput<'_>) -> DoctorCheck {
    let result = input.config.validate();
    DoctorCheck {
        id: CheckId::Config,
        ok: result.is_ok(),
        detail: match result {
            Ok(()) => "config válida".to_string(),
            Err(error) => format!("{error}; corrija `.knudge/config.toml`"),
        },
        fixable: false,
    }
}

pub(super) fn body_hash_check(notes: &[Note]) -> Result<DoctorCheck> {
    let mut stale = Vec::new();
    for note in notes {
        if body_hash_mismatch(note)? {
            stale.push(note.id()?.to_string());
        }
    }
    Ok(DoctorCheck {
        id: CheckId::BodyHash,
        ok: stale.is_empty(),
        detail: if stale.is_empty() {
            "body_hash em dia".to_string()
        } else {
            format!(
                "{} body_hash desatualizado(s); `--fix` recalcula",
                stale.len()
            )
        },
        fixable: true,
    })
}

pub(super) fn events_check(input: &DoctorInput<'_>) -> DoctorCheck {
    match input.events.read_all() {
        Ok((_events, warnings)) => DoctorCheck {
            id: CheckId::Events,
            ok: warnings.is_empty(),
            detail: if warnings.is_empty() {
                "eventos íntegros".to_string()
            } else {
                format!(
                    "{} linha(s) malformada(s); leitura tolerante as ignora",
                    warnings.len()
                )
            },
            fixable: false,
        },
        Err(error) => DoctorCheck {
            id: CheckId::Events,
            ok: false,
            detail: format!("{error}; verifique `eventos/`"),
            fixable: false,
        },
    }
}

pub(super) fn derived_check(
    input: &DoctorInput<'_>,
    expected: &Index,
    warnings: &mut Vec<String>,
) -> DoctorCheck {
    let diverges = derived_diverges(input, expected, warnings);
    DoctorCheck {
        id: CheckId::Derived,
        ok: !diverges,
        detail: if diverges {
            "índice derivado ausente/divergente; `--fix` reconstrói".to_string()
        } else {
            "índice derivado coerente".to_string()
        },
        fixable: true,
    }
}

/// Reporta o tamanho do índice vetorial e do cache de embeddings (E11-T02/R14).
pub(super) fn embeddings_check(input: &DoctorInput<'_>) -> DoctorCheck {
    let index_bytes = file_len(input.fs, &input.root.join(".idx").join(EMBEDDINGS_FILE));
    let cache_bytes = file_len(input.fs, &input.root.join(".idx").join(CACHE_FILE));
    let limit = EMBEDDINGS_WARN_BYTES;
    let ok = index_bytes <= limit && cache_bytes <= limit;
    DoctorCheck {
        id: CheckId::Embeddings,
        ok,
        detail: format!("índice {index_bytes} bytes; cache {cache_bytes} bytes"),
        fixable: false,
    }
}

fn file_len(fs: &dyn Fs, path: &Path) -> u64 {
    if !fs.exists(path) {
        return 0;
    }
    fs.read(path)
        .map_or(0, |bytes| u64::try_from(bytes.len()).unwrap_or(u64::MAX))
}

/// Verifica o elo Épico-raiz ↔ programa externo (`plan/*.md`) — D119.
///
/// Warn (não erro): (a) Épico-raiz sem programa; (b) programa órfão (arquivo do glob sem
/// Épico-raiz). A divergência de `content_hash` já é coberta pelo check `anchors` (verify-on-hit).
pub(super) fn program_anchor_check(input: &DoctorInput<'_>, notes: &[Note]) -> Result<DoctorCheck> {
    let glob = input
        .config
        .get_str("programs.glob")
        .unwrap_or("plan/*.md")
        .to_string();
    let mut missing = Vec::new();
    for note in notes {
        if note.frontmatter.note_type()? != NoteType::Container {
            continue;
        }
        if parent_of(note).is_some() {
            continue;
        }
        if program_of(note, &glob)?.is_none() {
            missing.push(note.id()?.to_string());
        }
    }
    let orphans = orphan_programs(input, notes, &glob)?;
    let ok = missing.is_empty() && orphans.is_empty();
    let detail = if ok {
        "programas ancorados".to_string()
    } else {
        format!(
            "{} épico(s)-raiz sem programa, {} programa(s) órfão(s); ancore o container-raiz com `--anchors <arquivo>`",
            missing.len(),
            orphans.len()
        )
    };
    Ok(DoctorCheck {
        id: CheckId::ProgramAnchor,
        ok,
        detail,
        fixable: false,
    })
}

fn orphan_programs(input: &DoctorInput<'_>, notes: &[Note], glob: &str) -> Result<Vec<String>> {
    let Some((dir, pattern)) = split_glob(glob) else {
        return Ok(Vec::new());
    };
    let dir_path = input.project_root.join(dir);
    if !input.fs.is_dir(&dir_path) {
        return Ok(Vec::new());
    }
    let mut orphans = Vec::new();
    for entry in input.fs.list_dir(&dir_path)? {
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !glob_match(pattern, name) {
            continue;
        }
        let path = if dir.is_empty() {
            name.to_string()
        } else {
            format!("{dir}/{name}")
        };
        if root_for_path(notes, &path)?.is_none() {
            orphans.push(path);
        }
    }
    orphans.sort();
    Ok(orphans)
}

/// Divide `dir/pattern` quando `dir` não tem metacaracteres de glob.
fn split_glob(glob: &str) -> Option<(&str, &str)> {
    let (dir, pattern) = glob.rsplit_once('/')?;
    if dir.contains('*') || dir.contains('?') || dir.contains('[') {
        return None;
    }
    Some((dir, pattern))
}
