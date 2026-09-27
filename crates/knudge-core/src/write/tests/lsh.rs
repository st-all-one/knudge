//! Blocking MinHash/LSH do dedup (E19/T07/D204).

use proptest::prelude::*;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::write::dedup::lsh::{MIN_LSH_CORPUS, SIGNATURE_LEN, candidate_pairs, signature};
use crate::write::{DedupThresholds, propose_merges};

use super::{note, seeded};

/// Razão `part / whole` em `f64` (contagens pequenas).
#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "contagens de termos em teste cabem em f64"
)]
fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        return 1.0;
    }
    (part as f64) / (whole as f64)
}

#[test]
fn signature_is_deterministic_and_order_independent() {
    let first = signature(["alpha", "beta", "gamma"]);
    let second = signature(["gamma", "beta", "alpha"]);
    assert_eq!(first, second, "assinatura independe da ordem");
    assert_eq!(first.len(), SIGNATURE_LEN);
}

#[test]
fn candidate_pairs_are_canonical_and_deterministic() {
    let signatures = [
        signature(["alpha", "beta", "gamma", "delta"]),
        signature(["alpha", "beta", "gamma", "delta"]),
        signature(["zeta", "eta", "theta", "iota"]),
    ];
    let pairs = candidate_pairs(&signatures);
    assert!(
        pairs.contains(&(0, 1)),
        "conjuntos iguais compartilham bandas"
    );
    assert!(pairs.iter().all(|(left, right)| left < right));
    assert_eq!(pairs, candidate_pairs(&signatures), "determinístico");
}

proptest! {
    /// MinHash estima Jaccard com erro pequeno (64 permutações).
    #[test]
    fn minhash_approximates_jaccard(
        left in prop::collection::btree_set("[a-h]{1,3}", 1..6),
        right in prop::collection::btree_set("[a-h]{1,3}", 1..6),
    ) {
        let left_terms: Vec<&str> = left.iter().map(String::as_str).collect();
        let right_terms: Vec<&str> = right.iter().map(String::as_str).collect();
        let left_signature = signature(left_terms.iter().copied());
        let right_signature = signature(right_terms.iter().copied());
        let equal = left_signature
            .iter()
            .zip(right_signature.iter())
            .filter(|(a, b)| a == b)
            .count();
        let estimate = ratio(equal, SIGNATURE_LEN);
        let true_jaccard = ratio(left.intersection(&right).count(), left.union(&right).count());
        prop_assert!(
            (estimate - true_jaccard).abs() < 0.4,
            "estimativa {estimate} distante de {true_jaccard}"
        );
    }
}

#[test]
fn lsh_proposes_every_near_duplicate_in_a_dense_corpus() -> Result<()> {
    // Corpus denso no limiar do LSH: grupos de 16 notas que compartilham 12 dos 13 termos.
    const GROUP: usize = 16;
    let fs = MemFs::new();
    let mut notes = Vec::new();
    for index in 0..MIN_LSH_CORPUS {
        let group = index / GROUP;
        let shared: Vec<String> = (0..12).map(|term| format!("g{group}t{term}")).collect();
        let statement = format!("{} u{index}", shared.join(" "));
        notes.push(note(NoteType::Fact, &statement, "")?);
    }
    let ctx = seeded(&fs, &notes)?;
    let proposals = propose_merges(ctx.index(), &DedupThresholds::default());
    let per_group = GROUP.saturating_mul(GROUP.saturating_sub(1)) / 2;
    assert_eq!(
        proposals.len(),
        MIN_LSH_CORPUS / GROUP * per_group,
        "cada par quase-duplicado deveria virar uma proposta"
    );
    Ok(())
}
