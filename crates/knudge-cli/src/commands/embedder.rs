//! Construção do embedder a partir da config (E12-T01, D101).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::adapters::HttpEmbedder;
use knudge_core::embeddings::{EmbeddingIndex, EmbeddingMeta, EmbeddingState, LightweightEmbedder};
use knudge_core::ports::Embedder;
use knudge_core::schema::Value;

use crate::session::Session;

/// Constrói o embedder configurado (`http`/`lightweight`/`none`).
///
/// # Errors
/// Retorna `ErrorKind::Config` para provedor inválido e propaga erro do adaptador.
pub fn build(session: &Session) -> Result<Option<Box<dyn Embedder>>> {
    let config = session.config();
    if !config.get_bool("embeddings.enabled").unwrap_or(true) {
        return Ok(None);
    }
    match config.get_str("embeddings.provider").unwrap_or("http") {
        "http" => Ok(Some(Box::new(HttpEmbedder::new(config, session.env())?))),
        "lightweight" => {
            let dimensions = config
                .get_int("embeddings.dimensions")
                .and_then(|raw| usize::try_from(raw).ok())
                .unwrap_or(384);
            Ok(Some(Box::new(LightweightEmbedder::new(dimensions)?)))
        }
        "none" => Ok(None),
        other => Err(Error::config(format!(
            "embeddings.provider inválido: {other:?}"
        ))),
    }
}

/// Identidade do embedder configurado (para leitura do índice).
///
/// # Errors
/// Retorna `ErrorKind::Config` para provedor/dimensão inválidos.
pub fn meta(session: &Session) -> Result<EmbeddingMeta> {
    let config = session.config();
    if config.get_str("embeddings.provider").unwrap_or("http") == "lightweight" {
        let dimensions = config
            .get_int("embeddings.dimensions")
            .and_then(|raw| usize::try_from(raw).ok())
            .unwrap_or(384);
        return Ok(LightweightEmbedder::new(dimensions)?.meta().clone());
    }
    EmbeddingMeta::from_config(config)
}

/// Conta notas `pending`/`stale` na fila de embeddings.
///
/// # Errors
/// Propaga erros de leitura do índice/store.
pub fn pending(session: &Session) -> Result<usize> {
    let meta = meta(session)?;
    let mut warnings = Vec::new();
    let Some(index) = EmbeddingIndex::load(
        session.fs_dyn(),
        &session.knowledge_dir(),
        &meta,
        &mut warnings,
    )?
    else {
        return Ok(session.store().list_ids()?.len());
    };
    let store = session.store();
    let mut pending = 0_usize;
    for id in store.list_ids()? {
        let note = store.read(&id)?;
        let body_hash = note
            .frontmatter
            .get("body_hash")
            .and_then(Value::as_str)
            .unwrap_or("");
        if index.state_of(&id, body_hash) != EmbeddingState::Indexed {
            pending = pending.saturating_add(1);
        }
    }
    Ok(pending)
}
