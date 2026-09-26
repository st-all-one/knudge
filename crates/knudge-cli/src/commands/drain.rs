//! `kd drain` — estado e digestão da fila de embeddings (D170).
//!
//! Absorve o antigo `kd knowledge digest` (D145). Três modos, mutuamente exclusivos:
//!
//! - `kd drain` (sem flags) = **help**: o `clap` (`arg_required_else_help`) mostra os modos e
//!   **não executa nada**;
//! - `kd drain --status` = estado rico (enabled/provider/mode/dimensions, indexed/pending/stale
//!   e uma recomendação), sem tocar em `.idx/`;
//! - `kd drain --digest [--force]` = digestão em lotes até esvaziar/estagnar; `--force` é o
//!   último recurso: apaga `.idx/` (100 % derivado, D84) e redige todas as notas do zero.

use knudge_core::Result;
use knudge_core::embeddings::{EmbeddingIndex, EmbeddingState};
use knudge_core::schema::Value;
use serde_json::json;

use crate::cli::DrainArgs;
use crate::commands::embedder;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd drain [--status | --digest [--force]]`.
///
/// # Errors
/// Propaga erros de leitura do índice/store e de execução do provedor.
pub fn run(session: &Session, args: &DrainArgs) -> Result<Output> {
    if args.digest {
        let rebuild = if args.force {
            Rebuild::Wipe
        } else {
            Rebuild::Keep
        };
        return digest(session, rebuild);
    }
    status(session)
}

/// Modo de reconstrução do derivado em `--digest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rebuild {
    /// Digere o que falta, mantendo `.idx/`.
    Keep,
    /// Apaga `.idx/` e redigeri tudo do zero (`--force`).
    Wipe,
}

impl Rebuild {
    const fn wipes(self) -> bool {
        matches!(self, Self::Wipe)
    }
}

/// Contagem de estados da fila + se há índice persistido.
struct Counts {
    indexed: usize,
    pending: usize,
    stale: usize,
    has_index: bool,
    warnings: Vec<String>,
}

fn status(session: &Session) -> Result<Output> {
    let enabled = session
        .config()
        .get_bool("embeddings.enabled")
        .unwrap_or(true);
    let provider = session
        .config()
        .get_str("embeddings.provider")
        .unwrap_or("http")
        .to_string();
    let mode = session
        .config()
        .get_str("embeddings.mode")
        .unwrap_or("lazy")
        .to_string();
    let dimensions = session
        .config()
        .get_int("embeddings.dimensions")
        .unwrap_or(384);
    if !enabled || provider == "none" {
        return Ok(disabled_status(&provider, &mode, dimensions));
    }
    let counts = counts(session)?;
    Ok(enabled_status(&provider, &mode, dimensions, &counts))
}

fn disabled_status(provider: &str, mode: &str, dimensions: i64) -> Output {
    let recommendation = "embeddings desligados; nada a fazer".to_string();
    let text = format!(
        "enabled=false provider={provider} mode={mode} dimensions={dimensions}\n{recommendation}"
    );
    Output::new(
        text,
        json!({
            "enabled": false,
            "provider": provider,
            "mode": mode,
            "dimensions": dimensions,
            "indexed": 0,
            "pending": 0,
            "stale": 0,
            "recommendation": recommendation,
        }),
    )
}

fn enabled_status(provider: &str, mode: &str, dimensions: i64, counts: &Counts) -> Output {
    let backlog = counts.pending.saturating_add(counts.stale);
    let recommendation = if !counts.has_index {
        "nada digerido ainda; rode `kd drain --digest`".to_string()
    } else if backlog > 0 {
        format!("há {backlog} pendente(s)/estragado(s); rode `kd drain --digest`")
    } else {
        "fila limpa".to_string()
    };
    let text = format!(
        "enabled=true provider={provider} mode={mode} dimensions={dimensions}\n\
         indexed={} pending={} stale={}\n{recommendation}",
        counts.indexed, counts.pending, counts.stale
    );
    let data = json!({
        "enabled": true,
        "provider": provider,
        "mode": mode,
        "dimensions": dimensions,
        "indexed": counts.indexed,
        "pending": counts.pending,
        "stale": counts.stale,
        "recommendation": recommendation,
    });
    Output::new(text, data).with_warnings(counts.warnings.clone())
}

/// Conta `indexed`/`pending`/`stale` sobre o corpus e diz se há índice persistido.
fn counts(session: &Session) -> Result<Counts> {
    let meta = embedder::meta(session)?;
    let store = session.store();
    let ids = store.list_ids()?;
    let mut warnings = Vec::new();
    let index = EmbeddingIndex::load(
        session.fs_dyn(),
        &session.knowledge_dir(),
        &meta,
        &mut warnings,
    )?;
    let has_index = index.is_some();
    let mut indexed = 0_usize;
    let mut pending = 0_usize;
    let mut stale = 0_usize;
    for id in &ids {
        let Some(note) = store.read_optional(id)? else {
            continue;
        };
        let body_hash = note
            .frontmatter
            .get("body_hash")
            .and_then(Value::as_str)
            .unwrap_or("");
        let current = index
            .as_ref()
            .map_or(EmbeddingState::Pending, |idx| idx.state_of(id, body_hash));
        match current {
            EmbeddingState::Indexed => indexed = indexed.saturating_add(1),
            EmbeddingState::Stale => stale = stale.saturating_add(1),
            EmbeddingState::Pending => pending = pending.saturating_add(1),
        }
    }
    Ok(Counts {
        indexed,
        pending,
        stale,
        has_index,
        warnings,
    })
}

fn digest(session: &Session, rebuild: Rebuild) -> Result<Output> {
    let force = rebuild.wipes();
    let mut warnings = Vec::new();
    let mut removed = Vec::new();
    if force {
        removed = remove_derived(session)?;
        warnings.push(format!(
            "`.idx/` apagado ({} item(ns)); redigerindo tudo",
            removed.len()
        ));
    }

    let mut batches = 0_usize;
    let mut indexed = 0_usize;
    let mut cache_hits = 0_usize;
    loop {
        let Some(outcome) = embedder::drain_once(session)? else {
            return Ok(Output::new(
                "embeddings desligado (provider = none)",
                json!({
                    "enabled": false,
                    "batches": batches,
                    "indexed": indexed,
                    "cache_hits": cache_hits,
                    "rebuilt": force,
                    "removed": removed,
                }),
            )
            .with_warnings(warnings));
        };
        warnings.extend(outcome.warnings);
        batches = batches.saturating_add(1);
        indexed = indexed.saturating_add(outcome.indexed);
        cache_hits = cache_hits.saturating_add(outcome.cache_hits);
        if outcome.indexed == 0 {
            break;
        }
    }

    let text = format!("indexed={indexed} batches={batches} cache_hits={cache_hits}");
    let data = json!({
        "enabled": true,
        "batches": batches,
        "indexed": indexed,
        "cache_hits": cache_hits,
        "rebuilt": force,
        "removed": removed,
    });
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Remove `.idx/` (derivado) e devolve o nome dos itens apagados.
fn remove_derived(session: &Session) -> Result<Vec<String>> {
    let dir = session.knowledge_dir().join(".idx");
    let fs = session.fs_dyn();
    if !fs.exists(&dir) {
        return Ok(Vec::new());
    }
    let entries = fs.list_dir(&dir)?;
    let names = entries
        .iter()
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    fs.remove_dir_all(&dir)?;
    Ok(names)
}
