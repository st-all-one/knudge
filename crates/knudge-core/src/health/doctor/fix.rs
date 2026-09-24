//! Reparo reversível do `doctor --fix` (D19, E09-T04).
//!
//! Corrige `body_hash` desatualizado, remove âncoras quebradas, re-hasheia âncoras, remove
//! locks stale e reconstrói o índice derivado divergente. Nunca apaga notas nem resolve
//! duplicatas/ciclos (decisão do agente — D47). Idempotente.

use crate::Result;
use crate::retrieval::Index;
use crate::schema::{Value, id};
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
    fix_note_layout(input, &mut fixed)?;
    fix_legacy_group(input, &mut fixed)?;
    fix_body_hashes(input, &mut fixed)?;
    fix_removed_keys(input, &mut fixed)?;
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

/// Move notas do layout plano antigo (`notas/<id>.md`) para `notas/<tipo>/<id>.md` (D150).
///
/// Roda **primeiro**: os reparos seguintes leem/escrevem pelo `note_path` derivado.
fn fix_note_layout(input: &DoctorInput<'_>, fixed: &mut Vec<String>) -> Result<()> {
    let dir = input.store.notes_dir();
    if !input.fs.exists(&dir) {
        return Ok(());
    }
    for path in input.fs.list_dir(&dir)? {
        if input.fs.is_dir(&path) {
            continue;
        }
        let is_markdown = path.extension().and_then(|e| e.to_str()) == Some("md");
        if !is_markdown {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !id::is_valid_note_id(id) {
            continue;
        }
        let target = input.store.note_path(id);
        if target == path {
            continue;
        }
        let bytes = input.fs.read(&path)?;
        if let Some(parent) = target.parent() {
            input.fs.create_dir_all(parent)?;
        }
        input.fs.write_atomic(&target, &bytes)?;
        input.fs.remove_file(&path)?;
        fixed.push(format!("layout migrado: {id}"));
    }
    Ok(())
}

/// Migra notas legadas de grupo (D134/D149): `scope: plan` → `scope: epic`, `type`
/// `container`/`epic` removido (o grupo passa a ser `scope=epic` com `type` omitido).
/// Lossless (mesmo id).
fn fix_legacy_group(input: &DoctorInput<'_>, fixed: &mut Vec<String>) -> Result<()> {
    for id in input.store.list_ids()? {
        let path = input.store.note_path(&id);
        let bytes = input.fs.read(&path)?;
        let raw = String::from_utf8_lossy(&bytes);
        let has_group_type = raw.lines().any(|line| {
            let trimmed = line.trim_end();
            trimmed == "type: container" || trimmed == "type: epic"
        });
        let has_plan = raw.lines().any(|line| line.trim_end() == "scope: plan");
        if !has_group_type && !has_plan {
            continue;
        }
        let has_scope = raw
            .lines()
            .any(|line| line.trim_start().starts_with("scope:"));
        let mut out: Vec<String> = Vec::new();
        for line in raw.lines() {
            let trimmed = line.trim_end();
            if trimmed == "type: container" || trimmed == "type: epic" {
                continue;
            }
            if trimmed == "scope: plan" {
                out.push(line.replace("scope: plan", "scope: epic"));
            } else {
                out.push(line.to_string());
            }
        }
        // `type` de grupo saiu sem `scope`: o grupo vira `scope=epic` (D149).
        if has_group_type
            && !has_scope
            && out.first().is_some_and(|line| line.trim_end() == "---")
            && let Some(close) = out.iter().skip(1).position(|line| line.trim_end() == "---")
        {
            out.insert(close.saturating_add(1), "scope: epic".to_string());
        }
        let migrated = out.join("\n");
        input
            .fs
            .write_atomic(&path, format!("{migrated}\n").as_bytes())?;
        input.events.append(
            &Event::new("doctor", input.now_ms)
                .with_note_id(&id)
                .with_data("fix", Value::Str("legacy_group".to_string())),
        )?;
        fixed.push(format!("grupo legado normalizado: {id}"));
    }
    Ok(())
}

/// Chaves que saíram do schema e o `--fix` retira de notas antigas (D135/D142).
const REMOVED_KEYS: [&str; 3] = ["confidence", "expires_at", "not_before"];

/// Remove de notas antigas as chaves que saíram do schema, reescrevendo a nota limpa.
fn fix_removed_keys(input: &DoctorInput<'_>, fixed: &mut Vec<String>) -> Result<()> {
    let read = read_tolerant(input.store)?;
    for note in &read.notes {
        let id = note.id()?;
        let path = input.store.note_path(id);
        let bytes = input.fs.read(&path)?;
        let raw = String::from_utf8_lossy(&bytes);
        let has_removed = REMOVED_KEYS.iter().any(|key| {
            raw.lines().any(|line| {
                line.strip_prefix(key)
                    .is_some_and(|rest| rest.starts_with(':'))
            })
        });
        if !has_removed {
            continue;
        }
        let mut repaired = note.clone();
        let revision = repaired.revision().saturating_add(1);
        repaired.set_revision(revision)?;
        repaired.refresh_body_hash()?;
        repaired.frontmatter.validate()?;
        input.store.write(&repaired)?;
        input.events.append(
            &Event::new("doctor", input.now_ms)
                .with_note_id(id)
                .with_data("fix", Value::Str("removed_keys".to_string())),
        )?;
        fixed.push(format!("chaves removidas do schema: {id}"));
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
            .filter(|anchor| {
                if is_glob(anchor) {
                    return true;
                }
                let path = input.project_root.join(anchor.as_str());
                // Diretório não tem conteúdo para hashear: trata como âncora inválida.
                input.fs.exists(&path) && !input.fs.is_dir(&path)
            })
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
