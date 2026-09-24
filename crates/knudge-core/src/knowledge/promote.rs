//! Promoção de conhecimento a regras governadas (`AGENTS.md`) — D157.
//!
//! O `AGENTS.md` é **orçamento escasso**: ~150–200 instruções e daí para cima o contexto
//! degrada. A promoção é **subtrativa por decisão do usuário**: o sistema recomenda, o humano
//! aprova/edita/remove; nada é escrito sozinho (D47). O bloco é delimitado por marcadores
//! próprios (`knudge:rules:start/end`), **irmão** do bloco de protocolo (D60) — assim o
//! `kd init`/`onboard`, que reescreve o bloco de protocolo, nunca o apaga.
//!
//! Cada linha carrega proveniência (`[id]` + confiança derivada, D87) e o ranking é
//! determinístico (`confidence desc, id asc`).

use std::path::Path;

use crate::Result;
use crate::config::Config;
use crate::git::block::{read_text, upsert};
use crate::graph::Graph;
use crate::lifecycle::confidence::{ConfidenceInput, confidence_score};
use crate::lifecycle::shelf_life::age_days;
use crate::ports::Fs;
use crate::schema::{Classification, EdgeKind, NoteType};
use crate::store::Note;

/// Início do bloco governado de regras.
pub const RULES_BEGIN: &str = "<!-- knudge:rules:start -->";
/// Fim do bloco governado de regras.
pub const RULES_END: &str = "<!-- knudge:rules:end -->";
/// Prefixo do version marker do bloco de regras.
pub const RULES_VERSION_PREFIX: &str = "<!-- knudge:rules:version:";
/// Versão do bloco de regras.
pub const RULES_VERSION: u32 = 1;
/// Título do bloco.
pub const RULES_HEADING: &str = "## knudge — regras promovidas";
/// Nome do arquivo governado.
pub const FILE: &str = "AGENTS.md";

/// Política de promoção.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RulesPolicy {
    /// Promoção habilitada.
    pub enabled: bool,
    /// Teto rígido de regras promovidas (admission control).
    pub max_promoted: usize,
    /// Confiança derivada mínima.
    pub min_confidence: f64,
}

impl Default for RulesPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_promoted: 15,
            min_confidence: 0.7,
        }
    }
}

impl RulesPolicy {
    /// Lê a política da config efetiva (defaults embutidos como fallback).
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let defaults = Self::default();
        Self {
            enabled: config.get_bool("rules.enabled").unwrap_or(defaults.enabled),
            max_promoted: config
                .get_int("rules.max_promoted")
                .and_then(|raw| usize::try_from(raw).ok())
                .unwrap_or(defaults.max_promoted),
            min_confidence: config
                .get_float("rules.min_confidence")
                .unwrap_or(defaults.min_confidence),
        }
    }
}

/// Candidata a regra promovida.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Id de origem.
    pub id: String,
    /// Afirmação (statement).
    pub statement: String,
    /// Confiança derivada (D87).
    pub confidence: f64,
    /// Motivo legível.
    pub reason: String,
}

/// Recomenda candidatas (puro, determinístico).
///
/// Elegibilidade: `type` em `meta|decision`, `classification = foundational`, confiança
/// derivada ≥ `min_confidence` e **sem** `contradicts` aberto. A "similaridade" do cálculo de
/// confiança é a amplitude (âncoras+tags normalizadas) — quanto mais ancorada, mais fundacional.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se algum campo tipado estiver malformado.
pub fn recommend(
    notes: &[Note],
    graph: &Graph,
    policy: &RulesPolicy,
    now_ms: i64,
) -> Result<Vec<Candidate>> {
    let mut candidates = Vec::new();
    for note in notes {
        let id = note.id()?.to_string();
        let note_type = note.frontmatter.note_type()?;
        if !matches!(note_type, NoteType::Meta | NoteType::Decision) {
            continue;
        }
        if note.frontmatter.classification()? != Classification::Foundational {
            continue;
        }
        if !graph.targets(&id, EdgeKind::Contradicts).is_empty() {
            continue;
        }
        let anchors = note.frontmatter.string_list("anchors")?.len();
        let tags = note.frontmatter.string_list("tags")?.len();
        let breadth = breadth_factor(anchors.saturating_add(tags));
        let score = confidence_score(&ConfidenceInput {
            similarity: breadth,
            age_days: age_days_f64(age_days(note, now_ms)),
            ..ConfidenceInput::default()
        });
        if score < policy.min_confidence {
            continue;
        }
        candidates.push(Candidate {
            id,
            statement: note.frontmatter.statement().unwrap_or_default().to_string(),
            confidence: score,
            reason: format!(
                "{}/{} conf={score:.2} sinais={}",
                note_type.as_str(),
                note.frontmatter.classification()?.as_str(),
                anchors.saturating_add(tags)
            ),
        });
    }
    candidates.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(candidates)
}

/// Renderiza o bloco governado a partir das candidatas aprovadas.
#[must_use]
pub fn render_block(approved: &[Candidate]) -> String {
    let mut lines = String::new();
    for candidate in approved {
        lines.push_str("- [");
        lines.push_str(&candidate.id);
        lines.push_str("] ");
        lines.push_str(&candidate.statement);
        lines.push('\n');
    }
    format!(
        "{RULES_BEGIN}\n{RULES_VERSION_PREFIX} {RULES_VERSION} -->\n{RULES_HEADING}\n\n{lines}{RULES_END}\n"
    )
}

/// Extrai os ids das linhas promovidas no texto (entre os marcadores).
#[must_use]
pub fn parse_block(text: &str) -> Vec<String> {
    parse_entries(text).into_iter().map(|(id, _)| id).collect()
}

/// Extrai `(id, statement)` das linhas promovidas (preserva edições do usuário).
#[must_use]
pub fn parse_entries(text: &str) -> Vec<(String, String)> {
    let mut inside = false;
    let mut entries = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == RULES_BEGIN {
            inside = true;
            continue;
        }
        if trimmed == RULES_END {
            break;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- [")
            && let Some((id, tail)) = rest.split_once(']')
        {
            entries.push((id.trim().to_string(), tail.trim().to_string()));
        }
    }
    entries
}

/// Escreve/remove o bloco governado no `AGENTS.md`. Devolve `true` se mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` em falha de leitura/escrita.
pub fn apply_block(fs: &dyn Fs, root: &Path, approved: &[Candidate]) -> Result<bool> {
    let path = root.join(FILE);
    let original = read_text(fs, &path)?;
    let block = if approved.is_empty() {
        None
    } else {
        Some(render_block(approved))
    };
    let updated = upsert(&original, RULES_BEGIN, RULES_END, block.as_deref());
    if updated == original {
        return Ok(false);
    }
    fs.write_atomic(&path, updated.as_bytes())?;
    Ok(true)
}

/// Amplitude normalizada de sinais (`0..=1`, saturando em 5).
#[must_use]
fn breadth_factor(signals: usize) -> f64 {
    let capped = u8::try_from(signals.min(5)).unwrap_or(5);
    f64::from(capped) / 5.0
}

/// Idade em dias como float (clamp em `i32` para conversão exata).
#[must_use]
fn age_days_f64(age: i64) -> f64 {
    f64::from(i32::try_from(age).unwrap_or(0))
}
