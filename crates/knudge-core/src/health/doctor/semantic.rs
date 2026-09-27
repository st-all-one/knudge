//! Checks semânticos do `doctor`: integridade do grafo + conflitos de claims/ciclos de ontologia
//! (E19-T09/D207). Extraído de `checks.rs` para manter o arquivo abaixo do teto.

use crate::Result;
use crate::graph::{claim_conflicts, has_hierarchy_cycle};
use crate::store::Note;

use super::{CheckId, DoctorCheck, DoctorInput};

pub(super) fn integrity_check(input: &DoctorInput<'_>, notes: &[Note]) -> Result<DoctorCheck> {
    let issues = input.graph.integrity();
    let conflicts = claim_conflicts(notes)?;
    let hierarchy_cycle = has_hierarchy_cycle(input.graph);
    let semantic = conflicts.len().saturating_add(usize::from(hierarchy_cycle));
    let ok = issues.is_empty() && semantic == 0;
    Ok(DoctorCheck {
        id: CheckId::Integrity,
        ok,
        detail: if ok {
            "grafo íntegro".to_string()
        } else if semantic > 0 {
            format!(
                "{} problema(s); {semantic} conflito(s) semântico(s) (claims/ciclo de ontologia); \
                 revise `replaces ↔ superseded_by` e arestas penduradas",
                issues.len()
            )
        } else {
            format!(
                "{} problema(s); revise `replaces ↔ superseded_by` e arestas penduradas",
                issues.len()
            )
        },
        fixable: false,
    })
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
