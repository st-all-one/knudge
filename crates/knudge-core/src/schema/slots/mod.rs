//! Slots mínimos por espécie — obrigatoriedades **soft** (E19-T03/D191).
//!
//! O data contract por tipo é um **aviso**, não um erro: uma nota `decision` sem
//! `## Alternativas` continua válida, mas o `doctor`/`write` sinalizam o slot ausente. O
//! contrato não adiciona chave canônica nem muda `schema_version` (R1/D14); a verificação é
//! puramente derivada do **corpo** e do **lastro** (âncora/aresta) já existentes.
//!
//! A busca de slot casa cabeçalho Markdown (`## Alternativas`) ou rótulo em linha
//! (`Alternativas:`) com **fold de diacríticos** e sem caixa (D172), em PT-BR ou inglês. O corpo
//! é `normalize`-livre aqui: nada é reescrito, só inspecionado (D06 intacto).

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::schema::NoteType;

#[cfg(test)]
mod tests;

/// Um slot esperado de uma espécie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// Nome canônico (exibido no aviso).
    pub name: &'static str,
    /// Sinônimos aceitos no corpo (PT-BR/EN, já dobrados).
    pub aliases: &'static [&'static str],
}

impl Slot {
    const fn new(name: &'static str, aliases: &'static [&'static str]) -> Self {
        Self { name, aliases }
    }
}

const DECISION_SLOTS: &[Slot] = &[
    Slot::new(
        "alternativas",
        &["alternativas", "alternatives", "opcoes", "options"],
    ),
    Slot::new("por quê", &["por que", "porque", "why", "rationale"]),
    Slot::new(
        "consequência",
        &["consequencia", "consequence", "impacto", "impact"],
    ),
];
const ERROR_SLOTS: &[Slot] = &[
    Slot::new("causa", &["causa", "cause", "root cause"]),
    Slot::new(
        "correção",
        &["correcao", "fix", "correction", "solucao", "solution"],
    ),
];
const RISK_SLOTS: &[Slot] = &[
    Slot::new("probabilidade", &["probabilidade", "probability", "chance"]),
    Slot::new("impacto", &["impacto", "impact"]),
];
const DEF_SLOTS: &[Slot] = &[Slot::new(
    "significado",
    &["significado", "meaning", "definicao", "definition"],
)];
const SNIPPET_SLOTS: &[Slot] = &[Slot::new("linguagem", &["linguagem", "language", "lang"])];

/// Slots de **corpo** esperados de `note_type` (vazio quando não há contrato de corpo).
#[must_use]
pub fn expected_slots(note_type: NoteType) -> &'static [Slot] {
    match note_type {
        NoteType::Decision => DECISION_SLOTS,
        NoteType::Error => ERROR_SLOTS,
        NoteType::Risk => RISK_SLOTS,
        NoteType::Def => DEF_SLOTS,
        NoteType::Snippet => SNIPPET_SLOTS,
        _ => &[],
    }
}

/// Slots ausentes numa nota: corpo + lastro de frontmatter (D191).
///
/// `anchors`/`edges` são as contagens já lidas do frontmatter. Devolve os nomes canônicos dos
/// slots que faltam, em ordem determinística (corpo, depois lastro).
#[must_use]
pub fn missing_slots(
    note_type: NoteType,
    body: &str,
    anchors: usize,
    edges: usize,
) -> Vec<&'static str> {
    let expected = expected_slots(note_type);
    let mut missing = Vec::new();
    if !expected.is_empty() {
        let haystack = searchable(body);
        for slot in expected {
            if !slot
                .aliases
                .iter()
                .any(|alias| contains_alias(&haystack, alias))
            {
                missing.push(slot.name);
            }
        }
    }
    match note_type {
        // `fact` sem lastro já é coberto pelo D162 (`body_check` sem corpo+lastro); não duplicar.
        NoteType::Snippet if anchors == 0 => missing.push("âncora"),
        NoteType::Question if anchors == 0 && edges == 0 => {
            missing.push("o que falta (âncora ou depends_on)");
        }
        _ => {}
    }
    // Cerca de código com linguagem (` ```rust `) satisfaz o slot `linguagem`.
    if note_type == NoteType::Snippet && has_fenced_language(body) {
        missing.retain(|slot| *slot != "linguagem");
    }
    missing
}

/// `true` se o corpo tem uma cerca de código com linguagem explícita (` ```rust `).
fn has_fenced_language(body: &str) -> bool {
    body.lines().any(|line| {
        line.trim_start().strip_prefix("```").is_some_and(|rest| {
            let lang = rest.trim();
            !lang.is_empty()
                && lang.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || matches!(ch, '+' | '#' | '-' | '.' | '_')
                })
        })
    })
}

/// Forma de busca: fold de diacríticos, minúsculo e não-alfanumérico → espaço único.
fn searchable(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push(' ');
    let mut pending_space = true;
    for ch in text.nfd() {
        if is_combining_mark(ch) {
            continue;
        }
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            pending_space = false;
        } else if !pending_space {
            out.push(' ');
            pending_space = true;
        }
    }
    if !pending_space {
        out.push(' ');
    }
    out
}

/// `true` se o alias aparece como **palavra/frase inteira** em `haystack` (bordas com espaço).
fn contains_alias(haystack: &str, alias: &str) -> bool {
    let needle = format!(" {alias} ");
    haystack.contains(needle.as_str())
}
