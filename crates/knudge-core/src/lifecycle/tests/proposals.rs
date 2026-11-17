//! Testes do derivado de propostas de manutenção (D230).

use crate::Result;
use crate::lifecycle::plan::{DemotionAction, DemotionReason};
use crate::lifecycle::proposals::{ProposalEntry, ProposalStore};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::store::purge_derived;

use super::ROOT;

fn entry(id: &str, reason: DemotionReason, action: DemotionAction) -> ProposalEntry {
    ProposalEntry {
        id: id.to_string(),
        reason,
        action,
    }
}

#[test]
fn proposals_round_trip_and_empty_clears_the_plan() -> Result<()> {
    let fs = MemFs::default();
    let store = ProposalStore::new(&fs, ROOT);
    assert_eq!(store.count()?, 0, "arquivo ausente ⇒ zero propostas");

    let entries = vec![
        entry("b", DemotionReason::Drifted, DemotionAction::Review),
        entry("a", DemotionReason::AnchorDecay, DemotionAction::Forget),
    ];
    store.persist(&entries)?;
    assert_eq!(store.count()?, 2);
    let loaded = store.load()?;
    let first = loaded
        .first()
        .ok_or_else(|| crate::Error::internal("sem primeira proposta"))?;
    assert_eq!(first.id, "a", "ordenação determinística por id");
    assert_eq!(first.action, DemotionAction::Forget);
    let second = loaded
        .get(1)
        .ok_or_else(|| crate::Error::internal("sem segunda proposta"))?;
    assert_eq!(second.reason, DemotionReason::Drifted);

    store.persist(&[])?;
    assert_eq!(store.count()?, 0, "plano vazio limpa o anterior");
    Ok(())
}

#[test]
fn count_ignores_blank_and_malformed_lines() -> Result<()> {
    let fs = MemFs::default();
    let store = ProposalStore::new(&fs, ROOT);
    let valid = b"{\"id\":\"a\",\"reason\":\"drifted\",\"action\":\"review\"}\n";
    fs.write_atomic(&store.path(), b"linha ruim\n\n")?;
    assert_eq!(store.count()?, 0);
    assert!(store.load()?.is_empty());

    let mut content = b"linha ruim\n".to_vec();
    content.extend_from_slice(valid);
    content.extend_from_slice(b"\n{incompleto\n");
    fs.write_atomic(&store.path(), &content)?;
    assert_eq!(
        store.count()?,
        1,
        "linhas inválidas não contam como proposta"
    );
    assert_eq!(store.load()?.len(), 1, "count e load devem concordar");
    Ok(())
}

#[test]
fn purge_derived_removes_a_proposal_by_id() -> Result<()> {
    let fs = MemFs::default();
    let store = ProposalStore::new(&fs, ROOT);
    store.persist(&[
        entry("a", DemotionReason::Expired, DemotionAction::Forget),
        entry("b", DemotionReason::Drifted, DemotionAction::Review),
    ])?;
    purge_derived(&fs, std::path::Path::new(ROOT), "a")?;
    let loaded = store.load()?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.first().map(|entry| entry.id.as_str()), Some("b"));
    Ok(())
}
