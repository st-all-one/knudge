//! Reparo reversível do `doctor --fix` (D19, E09-T04).
//!
//! Corrige `body_hash` desatualizado, remove âncoras quebradas, re-hasheia âncoras, remove
//! locks stale e reconstrói o índice derivado divergente. Nunca apaga notas nem resolve
//! duplicatas/ciclos (decisão do agente — D47). Idempotente.

use crate::Result;
use crate::retrieval::Index;
use crate::schema::Value;
use crate::store::{Event, Note};

use super::super::anchors::{AnchorStore, is_glob, refresh};
use super::super::audit::stale_lock_paths;
use super::super::tolerant::read_tolerant;
use super::{DoctorInput, DoctorReport, body_hash_mismatch, derived_diverges, doctor};

/// Executa os checks e aplica os reparos reversíveis.
///
/// # Errors
/// Propaga erros de I/O; o relatório final reflete o estado após os reparos.
pub fn doctor_fix(input: &DoctorInput<'_>) -> Result<DoctorReport> {
    let mut fixed = Vec::new();
    fix_body_hashes(input, &mut fixed)?;
    fix_broken_anchors(input, &mut fixed)?;
    let anchor_store = AnchorStore::new(input.fs, input.root);
    let changed = refresh(input.fs, input.project_root, input.store, &anchor_store)?;
    if changed > 0 {
        fixed.push(format!("âncoras re-hasheadas: {changed} nota(s)"));
    }
    for path in stale_lock_paths(input.fs, input.root, input.now_ms, input.lock_stale_ms)? {
        input.fs.remove_file(&path)?;
        fixed.push(format!("lock stale removido: {}", path.display()));
    }
    let read = read_tolerant(input.store)?;
    let expected = Index::build(&read.notes)?;
    let mut warnings = Vec::new();
    if derived_diverges(input, &expected, &mut warnings) {
        expected.save(input.fs, input.root, &mut warnings)?;
        fixed.push("índice derivado reconstruído".to_string());
    }
    let mut report = doctor(input)?;
    report.fixed = fixed;
    Ok(report)
}

fn fix_body_hashes(input: &DoctorInput<'_>, fixed: &mut Vec<String>) -> Result<()> {
    let read = read_tolerant(input.store)?;
    for note in &read.notes {
        if !body_hash_mismatch(note)? {
            continue;
        }
        let id = note.id()?;
        let mut repaired = note.clone();
        repaired.refresh_body_hash()?;
        input.store.write(&repaired)?;
        input.events.append(
            &Event::new("doctor", input.now_ms)
                .with_note_id(id)
                .with_data("fix", Value::Str("body_hash".to_string())),
        )?;
        fixed.push(format!("body_hash recalculado: {id}"));
    }
    Ok(())
}

fn fix_broken_anchors(input: &DoctorInput<'_>, fixed: &mut Vec<String>) -> Result<()> {
    let read = read_tolerant(input.store)?;
    for note in &read.notes {
        let anchors: Vec<String> = note
            .frontmatter
            .string_list("anchors")?
            .into_iter()
            .map(str::to_string)
            .collect();
        let kept: Vec<String> = anchors
            .iter()
            .filter(|anchor| is_glob(anchor) || input.fs.exists(&input.project_root.join(anchor)))
            .cloned()
            .collect();
        if kept.len() == anchors.len() {
            continue;
        }
        let id = note.id()?;
        let mut repaired = note.clone();
        set_anchors(&mut repaired, &kept)?;
        let revision = repaired.revision().saturating_add(1);
        repaired.set_revision(revision)?;
        repaired.refresh_body_hash()?;
        repaired.frontmatter.validate()?;
        input.store.write(&repaired)?;
        input.events.append(
            &Event::new("doctor", input.now_ms)
                .with_note_id(id)
                .with_data("fix", Value::Str("broken_anchors".to_string())),
        )?;
        fixed.push(format!("âncoras quebradas removidas: {id}"));
    }
    Ok(())
}

fn set_anchors(note: &mut Note, anchors: &[String]) -> Result<()> {
    if anchors.is_empty() {
        note.frontmatter.remove("anchors");
        return Ok(());
    }
    let value = Value::List(anchors.iter().map(|a| Value::Str(a.clone())).collect());
    note.frontmatter.set("anchors", value)
}
