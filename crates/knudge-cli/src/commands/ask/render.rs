//! Renderização de hits do `ask` no pipe e no `--json` (D39/D151/D161).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::retrieval::{
    RecallHit, body_matches, body_snippet, format_brief, format_hit, get,
};
use serde_json::json;

use crate::cli::AskArgs;
use crate::session::Session;

/// Formato de renderização de um hit no pipe/JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HitFormat {
    /// `id|statement|score|why` com corpo completo em todos os hits (`--full-content`).
    Full,
    /// `id|statement` (`--brief`).
    Brief,
    /// Revelação progressiva (D161): 1º completo, 2–5 truncados, restante padrão.
    Preview,
}

/// Número de hits com corpo na revelação progressiva (D161): 1º completo + 4 parciais.
const PREVIEW_HITS: usize = 5;

/// Formato do pipe: `--brief` (2 col) < padrão (progressivo) < `--full-content` (todos).
pub(super) fn hit_format(args: &AskArgs) -> HitFormat {
    if args.brief {
        HitFormat::Brief
    } else if args.full_content {
        HitFormat::Full
    } else {
        HitFormat::Preview
    }
}

/// Limite de caracteres do corpo nos hits parciais (config `recall.preview_chars`).
pub(super) fn preview_chars(session: &Session) -> usize {
    session
        .config()
        .get_int("recall.preview_chars")
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(280)
}

/// Trunca em `max_chars` caracteres, marcando o corte com `…`.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars).collect();
    out.push('…');
    out
}

/// Carrega os corpos dos hits (revelação progressiva e `--full-content`).
pub(super) fn load_bodies(
    session: &Session,
    hits: &[RecallHit],
) -> Result<BTreeMap<String, String>> {
    let ids: Vec<String> = hits.iter().map(|hit| hit.id.clone()).collect();
    let out = get(&session.store(), &ids)?;
    let mut bodies = BTreeMap::new();
    for note in out.notes {
        if let Ok(id) = note.frontmatter.id() {
            let _ignored = bodies.insert(id.to_string(), note.body);
        }
    }
    Ok(bodies)
}

/// Renderiza um hit no pipe: `--brief` → `id|statement`; `--full-content` → 4 colunas + corpo;
/// padrão → revelação progressiva (D161).
pub(super) fn render_hit(
    hit: &RecallHit,
    position: usize,
    format: HitFormat,
    bodies: &BTreeMap<String, String>,
    preview_chars: usize,
) -> String {
    if format == HitFormat::Brief {
        return format_brief(hit);
    }
    let line = format_hit(hit);
    let Some(text) = bodies.get(hit.id.as_str()).filter(|text| !text.is_empty()) else {
        return line;
    };
    match format {
        HitFormat::Brief => line,
        HitFormat::Full => format!("{line}\n{text}"),
        HitFormat::Preview => {
            if position == 0 {
                format!("{line}\n{text}")
            } else if position < PREVIEW_HITS {
                format!("{line}\n{}", truncate(text, preview_chars))
            } else {
                line
            }
        }
    }
}

/// Envelope JSON de um hit (completo; `body_match`/`body_snippet` quando há corpo).
pub(super) fn hit_json(
    hit: &RecallHit,
    format: HitFormat,
    bodies: &BTreeMap<String, String>,
    as_of: Option<&str>,
    query: &str,
) -> serde_json::Value {
    let mut value = if format == HitFormat::Brief {
        json!({ "id": hit.id, "statement": hit.statement })
    } else {
        json!({
            "id": hit.id,
            "statement": hit.statement,
            "score": hit.score,
            "confidence": hit.confidence,
            "why": hit.why.as_str(),
            "channels": {
                "lexical": hit.channels.lexical,
                "anchor": hit.channels.anchor,
                "semantic": hit.channels.semantic,
                "recent": hit.channels.recent,
                "stars": hit.channels.stars,
                "body": hit.channels.body,
            },
        })
    };
    if as_of.is_some()
        && let Some(object) = value.as_object_mut()
    {
        let _ignored = object.insert("historical".to_string(), json!(true));
    }
    if let Some(text) = bodies.get(hit.id.as_str())
        && let Some(object) = value.as_object_mut()
    {
        let matched = body_matches(text, query);
        let _ignored = object.insert("body".to_string(), json!(text));
        let _ignored = object.insert("body_match".to_string(), json!(matched));
        if matched {
            let _ignored = object.insert(
                "body_snippet".to_string(),
                json!(body_snippet(text, query, 240)),
            );
        }
    }
    value
}
