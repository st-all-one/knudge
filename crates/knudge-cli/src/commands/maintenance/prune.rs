//! `kd maintenance prune` — propostas de aposentadoria/revisão (D112/D230).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use knudge_core::Result;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{
    AnchorValidity, DecayPolicy, DemotionAction, DemotionCandidate, DemotionInput, DemotionReason,
    DriftStore, MAX_WALK_ENTRIES, ProposalEntry, ProposalStore, ShelfLife, UsageIndex, UsageStore,
    compute_anchor_validity_cached, demotion_candidates, entries_from_validity, retention_for,
    walk_paths_ignoring,
};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::CorpusArgs;
use crate::output::Output;
use crate::session::Session;

use super::super::corpus::CorpusScope;
/// Cobertura da varredura de âncoras feita pelo `prune`.
struct AnchorCoverage {
    declared: usize,
    valid: usize,
    ratio: f64,
    truncated: bool,
}

/// Abaixo disso, o walk não prova que as âncoras estão quebradas (D230).
const MIN_ANCHOR_COVERAGE: f64 = 0.5;

/// Validade de âncoras por id, calculada off-path.
type ValidityMap = BTreeMap<String, AnchorValidity>;
/// `kd maintenance prune` — propõe aposentadoria (`forget`) por shelf-life/decay (D112).
///
/// Nunca grava (D47): o agente aplica com `kd forget`. Membros de ciclo ficam de fora (D45).
///
/// # Errors
/// Propaga erros de leitura do store/grafo e varredura de âncoras.
#[allow(
    clippy::too_many_lines,
    reason = "orquestra cobertura, ações e persistência derivada do prune (D230)"
)]
pub fn prune(session: &Session, scope: Option<&str>, corpus: &CorpusArgs) -> Result<Output> {
    let corpus = CorpusScope::from(corpus);
    corpus.require("maintenance prune")?;
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let selection = corpus.select(index, graph)?;
    let notes = &loaded.notes;
    let shelf_life = ShelfLife::from_config(session.config());
    let decay = DecayPolicy::from_config(session.config());
    let (validity, coverage) = validity_map(session, notes)?;
    let usage = UsageStore::new(session.fs_dyn(), session.knowledge_dir()).index()?;
    let input = DemotionInput {
        now_ms: session.now_ms(),
        shelf_life: &shelf_life,
        decay: &decay,
        validity: &validity,
        usage: Some(&usage),
    };
    let mut candidates = demotion_candidates(notes, &input, graph)?;
    if let Some(container) = scope {
        candidates.retain(|candidate| belongs_to(graph, &candidate.id, container));
    }
    candidates.retain(|candidate| selection.matches_id(&candidate.id));

    let mut warnings = Vec::new();
    let coverage_ok = coverage.ratio >= MIN_ANCHOR_COVERAGE && !coverage.truncated;
    if !coverage_ok {
        candidates.retain(|candidate| {
            candidate.reason != DemotionReason::AnchorDecay
                || candidate.action != DemotionAction::Forget
        });
        warnings.push(format!(
            "cobertura de âncoras {:.0}% ({}/{}); não propus âncoras quebradas",
            coverage.ratio * 100.0,
            coverage.valid,
            coverage.declared
        ));
    }

    let proposals: Vec<ProposalEntry> = candidates
        .iter()
        .map(|candidate| ProposalEntry {
            id: candidate.id.clone(),
            reason: candidate.reason,
            action: candidate.action,
        })
        .collect();
    ProposalStore::new(session.fs_dyn(), session.knowledge_dir()).persist(&proposals)?;

    let forget = candidates
        .iter()
        .filter(|candidate| candidate.action == DemotionAction::Forget)
        .count();
    let review = candidates
        .iter()
        .filter(|candidate| candidate.action == DemotionAction::Review)
        .count();
    let mut text = candidates
        .iter()
        .map(|candidate| {
            format!(
                "{}|{}|{}",
                candidate.action.as_str(),
                candidate.id,
                candidate.reason.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    let _ignored = write!(
        text,
        "propostas: forget={forget} review={review}\npróximos: kd forget --id <ID> (forget) | kd maintenance learn/compact (review)",
    );
    let retentions = retention_map(session, &shelf_life, notes, &usage, &candidates)?;
    let data = json!({
        "proposals": candidates.iter().map(|candidate| json!({
            "action": candidate.action.as_str(),
            "id": candidate.id,
            "reason": candidate.reason.as_str(),
            "retention": retentions.get(&candidate.id),
        })).collect::<Vec<_>>(),
        "anchor_coverage": json!({
            "declared": coverage.declared,
            "valid": coverage.valid,
            "ratio": coverage.ratio,
            "walk_truncated": coverage.truncated,
        }),
    });
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Retenção atual (D190) das notas candidatas, por id — campo aditivo do `--json`.
fn retention_map(
    session: &Session,
    shelf_life: &ShelfLife,
    notes: &[Note],
    usage: &UsageIndex,
    candidates: &[DemotionCandidate],
) -> Result<BTreeMap<String, f64>> {
    let by_id: BTreeMap<&str, &Note> = notes
        .iter()
        .filter_map(|note| note.id().ok().map(|id| (id, note)))
        .collect();
    let mut retentions = BTreeMap::new();
    for candidate in candidates {
        if let Some(note) = by_id.get(candidate.id.as_str()) {
            let value = retention_for(
                note,
                session.now_ms(),
                shelf_life,
                usage.last_seen(&candidate.id),
            )?;
            let _ignored = retentions.insert(candidate.id.clone(), value);
        }
    }
    Ok(retentions)
}

/// Validade de âncoras por id (varredura off-path) para o plano de demolição.
///
/// Caminha o projeto **uma vez** e reusa os caminhos para todas as notas (E16/T10): antes era um
/// walk por nota, `O(notas × projeto)`.
fn validity_map(session: &Session, notes: &[Note]) -> Result<(ValidityMap, AnchorCoverage)> {
    let ignored = session.project().ignored_dirs();
    let paths = walk_paths_ignoring(session.fs_dyn(), session.project_root(), &ignored);
    let mut map = BTreeMap::new();
    let mut declared = 0_usize;
    let mut valid = 0_usize;
    for note in notes {
        let anchors: Vec<String> = note
            .frontmatter
            .string_list("anchors")?
            .into_iter()
            .map(str::to_string)
            .collect();
        if anchors.is_empty() {
            continue;
        }
        let validity = compute_anchor_validity_cached(
            session.fs_dyn(),
            session.project_root(),
            &paths,
            &anchors,
        );
        declared = declared.saturating_add(usize::try_from(validity.total).unwrap_or(usize::MAX));
        valid = valid.saturating_add(usize::try_from(validity.valid).unwrap_or(usize::MAX));
        let _ignored = map.insert(note.id()?.to_string(), validity);
    }
    DriftStore::new(session.fs_dyn(), session.knowledge_dir())
        .persist(&entries_from_validity(&map))?;
    let ratio = if declared == 0 {
        1.0
    } else {
        f64::from(u32::try_from(valid).unwrap_or(u32::MAX))
            / f64::from(u32::try_from(declared).unwrap_or(u32::MAX))
    };
    Ok((
        map,
        AnchorCoverage {
            declared,
            valid,
            ratio,
            truncated: paths.len() >= MAX_WALK_ENTRIES,
        },
    ))
}
