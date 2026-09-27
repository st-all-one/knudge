//! Fusão de campos semânticos numa nota existente (dedup `0.75–0.92`, D26/D207).
//!
//! Extraído de `write/mod.rs` para manter o arquivo abaixo do teto. Une claims e proveniência
//! sem duplicar; as tags/âncoras/corpo seguem em `merge_into`.

use crate::Result;
use crate::schema::{Frontmatter, claims, claims_to_value, provenance};

/// Une as claims do `incoming` no `target` (sem duplicar triplas) — D207.
pub(super) fn merge_claims(target: &mut Frontmatter, incoming: &Frontmatter) -> Result<()> {
    let mut existing = claims(target)?;
    for claim in claims(incoming)? {
        if !existing.contains(&claim) {
            existing.push(claim);
        }
    }
    if !existing.is_empty() {
        target.set("claims", claims_to_value(&existing))?;
    }
    Ok(())
}

/// Preenche os campos ausentes de `provenance` a partir do `incoming` — D207.
pub(super) fn merge_provenance(target: &mut Frontmatter, incoming: &Frontmatter) -> Result<()> {
    let mut current = provenance(target)?;
    let other = provenance(incoming)?;
    current.entity = current.entity.or(other.entity);
    current.activity = current.activity.or(other.activity);
    current.agent = current.agent.or(other.agent);
    if !current.is_empty() {
        target.set("provenance", current.to_value())?;
    }
    Ok(())
}
