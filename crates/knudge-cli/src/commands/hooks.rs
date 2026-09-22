//! Orquestração de hooks de ciclo de vida na borda (E12-T04, D59).

use std::time::Duration;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::adapters::ProcessHookRunner;
use knudge_core::ports::HookRunner;
use knudge_core::write::Draft;
use serde_json::Value;

use crate::session::Session;

/// Evento de ciclo de vida (D59).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    /// Antes de gravar uma nota (pode bloquear/mutar).
    PreRecord,
    /// Depois de gravar.
    PostRecord,
    /// Antes de emitir o protocolo (reservado; `prime` é byte-idêntico — D57).
    #[allow(dead_code, reason = "reservado: prime é estático (D57)")]
    PrePrime,
    /// Antes de purgar.
    PrePrune,
    /// Antes de compactar.
    PreCompact,
}

impl HookEvent {
    /// Chave de config (`hooks.<key>`).
    #[must_use]
    pub const fn config_key(self) -> &'static str {
        match self {
            Self::PreRecord => "hooks.pre_record",
            Self::PostRecord => "hooks.post_record",
            Self::PrePrime => "hooks.pre_prime",
            Self::PrePrune => "hooks.pre_prune",
            Self::PreCompact => "hooks.pre_compact",
        }
    }

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreRecord => "pre-record",
            Self::PostRecord => "post-record",
            Self::PrePrime => "pre-prime",
            Self::PrePrune => "pre-prune",
            Self::PreCompact => "pre-compact",
        }
    }
}

/// Resultado de um hook: se bloqueou e o payload (possivelmente mutado).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookResult {
    /// `true` se o hook retornou status ≠ 0.
    pub blocked: bool,
    /// Payload devolvido (igual ao de entrada quando o hook não emite JSON).
    pub payload: Value,
}

/// Executa o hook configurado para o evento, se houver.
///
/// # Errors
/// Propaga erro de execução do hook (inclusive timeout).
pub fn run(session: &Session, event: HookEvent, payload: &Value) -> Result<HookResult> {
    let command = session
        .config()
        .get_str(event.config_key())
        .filter(|value| !value.trim().is_empty());
    let Some(command) = command else {
        return Ok(HookResult {
            blocked: false,
            payload: payload.clone(),
        });
    };
    let timeout_ms = session
        .config()
        .get_int("hooks.timeout_ms")
        .and_then(|raw| u64::try_from(raw).ok())
        .unwrap_or(30_000);
    let runner = ProcessHookRunner::new(
        session.project_root(),
        Duration::from_millis(timeout_ms.max(1)),
    );
    run_with(&runner, command, payload)
}

/// Executa um hook com um runner arbitrário (testável).
///
/// # Errors
/// Propaga erro de execução do runner e de JSON malformado no stdout.
pub fn run_with(runner: &dyn HookRunner, command: &str, payload: &Value) -> Result<HookResult> {
    let input = serde_json::to_vec(payload)
        .map_err(|error| Error::internal(format!("payload de hook inválido: {error}")))?;
    let output = runner.run(command, &input)?;
    let blocked = output.status != 0;
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    let payload = if trimmed.is_empty() {
        payload.clone()
    } else {
        serde_json::from_str(trimmed).map_err(|error| {
            Error::invalid_input(format!("hook devolveu JSON inválido: {error}"))
        })?
    };
    Ok(HookResult { blocked, payload })
}

/// Aplica mutações de payload a um rascunho (`statement`/`body`/`tags`/`anchors`).
pub fn apply_to_draft(draft: &mut Draft, payload: &Value) {
    if let Some(statement) = payload.get("statement").and_then(Value::as_str) {
        draft.statement = statement.to_string();
    }
    if let Some(body) = payload.get("body").and_then(Value::as_str) {
        draft.body = body.to_string();
    }
    if let Some(tags) = payload.get("tags").and_then(Value::as_array) {
        draft.tags = tags
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
    }
    if let Some(anchors) = payload.get("anchors").and_then(Value::as_array) {
        draft.anchors = anchors
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
    }
}
