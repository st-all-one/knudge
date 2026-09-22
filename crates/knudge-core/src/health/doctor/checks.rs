//! Checks individuais do `doctor` (E09-T04).

use crate::Result;
use crate::retrieval::Index;
use crate::store::Note;
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
