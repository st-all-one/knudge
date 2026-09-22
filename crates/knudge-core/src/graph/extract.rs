//! Extração conservadora de arestas como **sugestão revisável** (D49/D50).
//!
//! Regras deliberadamente conservadoras: só sugere quando o alvo é um id válido e conhecido
//! (ou um wikilink explícito). Um verbo imediatamente antes do id escolhe a aresta; caso
//! contrário a sugestão é `references`. Nada aqui vira aresta — o `expand` ignora sugestões.

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices de varredura ASCII com domínio limitado ao tamanho do texto"
)]

use std::collections::BTreeSet;

use crate::schema::{EdgeKind, NoteType};

/// Sugestão de aresta `kind → target`, com o motivo legível (auditoria).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suggestion {
    /// Tipo sugerido.
    pub kind: EdgeKind,
    /// Id de destino.
    pub target: String,
    /// Por que foi sugerido.
    pub reason: String,
}

/// Verbos que definem a aresta quando imediatamente antes do id.
const CUES: [(&str, EdgeKind); 15] = [
    ("depende de", EdgeKind::DependsOn),
    ("dependência de", EdgeKind::DependsOn),
    ("requer", EdgeKind::DependsOn),
    ("precisa de", EdgeKind::DependsOn),
    ("contradiz", EdgeKind::Contradicts),
    ("refuta", EdgeKind::Contradicts),
    ("invalida", EdgeKind::Contradicts),
    ("confirma", EdgeKind::Supports),
    ("reforça", EdgeKind::Supports),
    ("suporta", EdgeKind::Supports),
    ("estende", EdgeKind::Extends),
    ("refina", EdgeKind::Extends),
    ("generaliza", EdgeKind::Extends),
    ("substitui", EdgeKind::Replaces),
    ("rejeita", EdgeKind::Rejects),
];

/// Extrai sugestões de aresta de um texto, mantendo só alvos em `known`.
#[must_use]
pub fn extract(text: &str, known: &BTreeSet<String>) -> Vec<Suggestion> {
    let bytes = text.as_bytes();
    let mut found: BTreeSet<(EdgeKind, String, String)> = BTreeSet::new();
    for index in 0..bytes.len() {
        if bytes.get(index) != Some(&b'_') {
            continue;
        }
        let Some((target, start)) = candidate(text, index) else {
            continue;
        };
        if !known.contains(&target) {
            continue;
        }
        let (kind, reason) = classify(text.get(..start).unwrap_or_default());
        found.insert((kind, target, reason.to_string()));
    }
    found
        .into_iter()
        .map(|(kind, target, reason)| Suggestion {
            kind,
            target,
            reason,
        })
        .collect()
}

/// Reconstrói um id válido cujo `_` está em `index`, se houver (com a posição inicial).
fn candidate(text: &str, index: usize) -> Option<(String, usize)> {
    let suffix = text.get(index + 1..index + 9)?;
    if !suffix
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    {
        return None;
    }
    if text
        .as_bytes()
        .get(index + 9)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return None;
    }
    let head = text.get(..index)?;
    let note_type = NoteType::ALL
        .iter()
        .find(|note_type| head.ends_with(note_type.prefix()))?;
    let start = index.checked_sub(note_type.prefix().len())?;
    if text
        .as_bytes()
        .get(start.wrapping_sub(1))
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return None;
    }
    Some((format!("{}_{suffix}", note_type.prefix()), start))
}

/// Classifica a aresta pelo contexto imediatamente anterior ao id.
fn classify(context: &str) -> (EdgeKind, &'static str) {
    let trimmed = context.trim_end();
    if trimmed.ends_with("[[") {
        return (EdgeKind::References, "wikilink");
    }
    let tail = tail_lowercase(trimmed, 24);
    for (cue, kind) in CUES {
        if tail.ends_with(cue) {
            return (kind, "verbo");
        }
    }
    (EdgeKind::References, "menção")
}

fn tail_lowercase(text: &str, max_chars: usize) -> String {
    let tail: String = text
        .chars()
        .rev()
        .take(max_chars)
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();
    tail.to_lowercase()
}
