//! Programas externos: o Épico-raiz ancorado a `plan/*.md` (D119).
//!
//! O **Programa** é um arquivo markdown real (o "porquê"), git-tracked; o knudge cuida da
//! corrente menor (Épico → User Story → Task). O elo é a **âncora** (D86): nenhum `scope` novo,
//! nenhuma chave TOON nova. O papel "Programa" é o rótulo do Épico-raiz ancorado.

use std::collections::BTreeSet;

use crate::Result;
use crate::graph::Graph;
use crate::retrieval::anchor::glob_match;
use crate::schema::NoteType;
use crate::store::Note;

use super::children;

/// Nó da subárvore de um programa (pré-ordem determinística).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramNode {
    /// Id.
    pub id: String,
    /// Profundidade a partir da raiz (0 = Épico-raiz).
    pub depth: usize,
}

/// Resolve os Épicos-raiz de um programa: **todos** os containers sem pai cujo `anchors` casa
/// `path` (nas duas direções), em ordem de `id` (D139).
///
/// # Errors
/// Propaga erros de parse do frontmatter.
pub fn roots_for_path(notes: &[Note], path: &str) -> Result<Vec<String>> {
    let mut roots = Vec::new();
    for note in notes {
        if note.frontmatter.note_type()? != NoteType::Epic {
            continue;
        }
        if super::parent_of(note).is_some() {
            continue;
        }
        let anchors = note.frontmatter.string_list("anchors")?;
        if !anchors.iter().any(|anchor| anchor_matches(anchor, path)) {
            continue;
        }
        roots.push(note.id()?.to_string());
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}

/// Path do programa ancorado a `note` (primeiro anchor que casa o glob) — D135.
///
/// # Errors
/// Propaga erros de parse do frontmatter.
pub fn program_of(note: &Note, glob: &str) -> Result<Option<String>> {
    let anchors = note.frontmatter.string_list("anchors")?;
    Ok(anchors
        .iter()
        .find(|anchor| glob_match(glob, anchor) || glob_match(anchor, glob))
        .map(|anchor| (*anchor).to_string()))
}

/// Subárvore de `root` em pré-ordem determinística (filhos por `id asc`), incluindo o root.
#[must_use]
pub fn subtree(graph: &Graph, root: &str) -> Vec<ProgramNode> {
    let mut out = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![(root.to_string(), 0_usize)];
    while let Some((current, depth)) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        out.push(ProgramNode {
            id: current.clone(),
            depth,
        });
        let mut kids = children(graph, &current);
        kids.sort();
        for child in kids.into_iter().rev() {
            stack.push((child, depth.saturating_add(1)));
        }
    }
    out
}

fn anchor_matches(anchor: &str, path: &str) -> bool {
    glob_match(anchor, path) || glob_match(path, anchor)
}
