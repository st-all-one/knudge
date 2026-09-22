//! Tools MCP dos gatilhos (E14-T03, D68).
//!
//! Os hints são **ponteiros**: nunca carregam corpo de nota. Argumentos inválidos devolvem um
//! resultado com `isError: true` — não derrubam o servidor.

use knudge_core::handoff::{ManifestItem, TrustTier};
use knudge_core::maintenance::{LearnKind, LearnProposal};
use knudge_core::write::Candidate;
use serde_json::{Value, json};

use crate::triggers::{Hint, HintEngine};

/// Tool pré-`write` (quase-duplicados).
pub const PRE_WRITE: &str = "knudge_pre_write";
/// Tool pré-edição (contexto do working set).
pub const PRE_EDIT: &str = "knudge_pre_edit";
/// Tool de fim de sessão (`learn` sem writes).
pub const SESSION_END: &str = "knudge_session_end";
/// Tool de status (modo observação).
pub const STATUS: &str = "knudge_status";

/// Lista de tools com `inputSchema`.
#[must_use]
pub fn list() -> Value {
    json!({ "tools": [pre_write_def(), pre_edit_def(), session_end_def(), status_def()] })
}

/// Executa uma tool; nunca falha (erros de argumento viram `isError`).
pub fn call(engine: &mut HintEngine, name: &str, arguments: &Value) -> Value {
    match name {
        PRE_WRITE => match parse_candidates(arguments) {
            Ok(candidates) => hints_result(&engine.pre_write(&candidates)),
            Err(message) => error_result(&message),
        },
        PRE_EDIT => match parse_items(arguments) {
            Ok(items) => hints_result(&engine.pre_edit(&items)),
            Err(message) => error_result(&message),
        },
        SESSION_END => match parse_proposals(arguments) {
            Ok(proposals) => {
                let writes = arguments.get("writes").and_then(Value::as_u64).unwrap_or(0);
                let writes = usize::try_from(writes).unwrap_or(usize::MAX);
                let hints = engine.session_end(writes, &proposals);
                let result = hints_result(&hints);
                engine.end_session();
                result
            }
            Err(message) => error_result(&message),
        },
        STATUS => status_result(engine),
        other => error_result(&format!("tool desconhecida: {other}")),
    }
}

fn pre_write_def() -> Value {
    json!({
        "name": PRE_WRITE,
        "description": "Quase-duplicatas antes de gravar (ponteiros, sem corpo).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "candidates": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "statement": { "type": "string" },
                            "score": { "type": "number" }
                        },
                        "required": ["id", "score"]
                    }
                }
            },
            "required": ["candidates"]
        }
    })
}

fn pre_edit_def() -> Value {
    json!({
        "name": PRE_EDIT,
        "description": "Contexto do working set antes de editar arquivos (ponteiros).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "statement": { "type": "string" },
                            "score": { "type": "number" }
                        },
                        "required": ["id", "score"]
                    }
                }
            },
            "required": ["items"]
        }
    })
}

fn session_end_def() -> Value {
    json!({
        "name": SESSION_END,
        "description": "Propostas de learn quando a sessão termina sem writes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "writes": { "type": "integer", "minimum": 0 },
                "proposals": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "kind": { "type": "string" },
                            "ids": { "type": "array", "items": { "type": "string" } },
                            "why": { "type": "string" },
                            "score": { "type": "number" }
                        },
                        "required": ["kind", "ids"]
                    }
                }
            },
            "required": ["writes", "proposals"]
        }
    })
}

fn status_def() -> Value {
    json!({
        "name": STATUS,
        "description": "Status do motor de hints (modo observação).",
        "inputSchema": { "type": "object", "properties": {} }
    })
}

fn parse_candidates(arguments: &Value) -> Result<Vec<Candidate>, String> {
    let array = arguments
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| "campo candidates ausente ou não é array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for raw in array {
        let id = raw
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "candidate.id ausente".to_string())?;
        let statement = raw
            .get("statement")
            .and_then(Value::as_str)
            .unwrap_or(id)
            .to_string();
        let score = raw.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        out.push(Candidate {
            id: id.to_string(),
            statement,
            score,
        });
    }
    Ok(out)
}

fn parse_items(arguments: &Value) -> Result<Vec<ManifestItem>, String> {
    let array = arguments
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| "campo items ausente ou não é array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for raw in array {
        let id = raw
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "item.id ausente".to_string())?;
        let statement = raw
            .get("statement")
            .and_then(Value::as_str)
            .unwrap_or(id)
            .to_string();
        let score = raw.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        out.push(ManifestItem {
            id: id.to_string(),
            statement,
            tier: TrustTier::Observational,
            score,
            created_ms: 0,
        });
    }
    Ok(out)
}

fn parse_proposals(arguments: &Value) -> Result<Vec<LearnProposal>, String> {
    let array = arguments
        .get("proposals")
        .and_then(Value::as_array)
        .ok_or_else(|| "campo proposals ausente ou não é array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for raw in array {
        let kind = raw
            .get("kind")
            .and_then(Value::as_str)
            .and_then(learn_kind)
            .ok_or_else(|| "proposal.kind desconhecido".to_string())?;
        let ids = raw
            .get("ids")
            .and_then(Value::as_array)
            .ok_or_else(|| "proposal.ids ausente".to_string())?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
        let why = raw
            .get("why")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let score = raw.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        out.push(LearnProposal {
            kind,
            ids,
            why,
            score,
        });
    }
    Ok(out)
}

fn learn_kind(value: &str) -> Option<LearnKind> {
    match value {
        "create_note" => Some(LearnKind::CreateNote),
        "merge" => Some(LearnKind::Merge),
        "supersede" => Some(LearnKind::Supersede),
        "link" => Some(LearnKind::Link),
        _ => None,
    }
}

fn hints_result(hints: &[Hint]) -> Value {
    let structured = json!({
        "hints": hints.iter().map(hint_value).collect::<Vec<Value>>(),
    });
    success_result(&structured)
}

fn hint_value(hint: &Hint) -> Value {
    json!({
        "kind": hint.kind.as_str(),
        "ids": &hint.ids,
        "score": hint.score,
        "why": hint.why,
        "observed": hint.observed,
    })
}

fn status_result(engine: &HintEngine) -> Value {
    success_result(&json!({
        "observing": engine.is_observing(),
        "sessions_seen": engine.sessions_seen(),
        "observation_sessions": engine.observation_sessions(),
        "cap": engine.cap(),
    }))
}

fn success_result(structured: &Value) -> Value {
    let text = serde_json::to_string(structured).unwrap_or_else(|_| "{}".to_string());
    json!({
        "content": [ { "type": "text", "text": text } ],
        "structuredContent": structured,
        "isError": false,
    })
}

fn error_result(message: &str) -> Value {
    json!({
        "content": [ { "type": "text", "text": message } ],
        "isError": true,
    })
}
