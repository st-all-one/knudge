//! Catálogo de chaves canônicas da config (E04/E11).
//!
//! A ordem de [`KEYS`] é a ordem canônica de emissão do `config.toml` (D63). As chaves de
//! `embeddings.*` vivem em [`super::keys_embeddings`] para manter cada arquivo sob o teto.

use std::sync::LazyLock;

use super::keys_embeddings::EMBEDDINGS;
use super::{Default, KeySpec, Kind};

/// Chaves canônicas, em ordem canônica (D63).
pub static KEYS: LazyLock<Vec<KeySpec>> = LazyLock::new(|| {
    let mut all = Vec::with_capacity(BASE.len().saturating_add(EMBEDDINGS.len()));
    all.extend_from_slice(BASE);
    all.extend_from_slice(EMBEDDINGS);
    all
});

/// Chaves canônicas (exceto `embeddings.*`), em ordem canônica.
const BASE: &[KeySpec] = &[
    KeySpec {
        key: "knowledge.persist_in_project",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "dedup.create_below",
        kind: Kind::Float,
        default: Default::Float(0.75),
    },
    KeySpec {
        key: "dedup.merge_below",
        kind: Kind::Float,
        default: Default::Float(0.92),
    },
    KeySpec {
        key: "write.batch_max",
        kind: Kind::Int,
        default: Default::Int(100),
    },
    KeySpec {
        key: "task.batch_max",
        kind: Kind::Int,
        default: Default::Int(100),
    },
    KeySpec {
        key: "recall.default_limit",
        kind: Kind::Int,
        default: Default::Int(5),
    },
    KeySpec {
        key: "recall.expand_depth",
        kind: Kind::Int,
        default: Default::Int(1),
    },
    KeySpec {
        key: "recall.rrf_k",
        kind: Kind::Int,
        default: Default::Int(60),
    },
    KeySpec {
        key: "recall.confirmation_from_tasks",
        kind: Kind::Float,
        default: Default::Float(0.1),
    },
    KeySpec {
        key: "recall.lexical_weight",
        kind: Kind::Float,
        default: Default::Float(1.0),
    },
    KeySpec {
        key: "recall.anchor_weight",
        kind: Kind::Float,
        default: Default::Float(1.0),
    },
    KeySpec {
        key: "recall.semantic_weight",
        kind: Kind::Float,
        default: Default::Float(30.0),
    },
    KeySpec {
        key: "recall.semantic",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "recall.semantic_top_k",
        kind: Kind::Int,
        default: Default::Int(50),
    },
    KeySpec {
        key: "mcp.observation_mode",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "mcp.observation_sessions",
        kind: Kind::Int,
        default: Default::Int(3),
    },
    KeySpec {
        key: "mcp.hints_cap",
        kind: Kind::Int,
        default: Default::Int(3),
    },
    KeySpec {
        key: "behavior.strict",
        kind: Kind::Bool,
        default: Default::Bool(false),
    },
    KeySpec {
        key: "proposals.gate",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "proposals.min_delta",
        kind: Kind::Float,
        default: Default::Float(0.0),
    },
    KeySpec {
        key: "proposals.enforce",
        kind: Kind::Bool,
        default: Default::Bool(false),
    },
    KeySpec {
        key: "suggestions.enabled",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "suggestions.contradiction_low",
        kind: Kind::Float,
        default: Default::Float(0.4),
    },
    KeySpec {
        key: "suggestions.contradiction_high",
        kind: Kind::Float,
        default: Default::Float(0.75),
    },
    KeySpec {
        key: "rules.enabled",
        kind: Kind::Bool,
        default: Default::Bool(false),
    },
    KeySpec {
        key: "rules.max_promoted",
        kind: Kind::Int,
        default: Default::Int(15),
    },
    KeySpec {
        key: "rules.min_confidence",
        kind: Kind::Float,
        default: Default::Float(0.7),
    },
    KeySpec {
        key: "retention.foundational_days",
        kind: Kind::Int,
        default: Default::Int(0),
    },
    KeySpec {
        key: "retention.tactical_days",
        kind: Kind::Int,
        default: Default::Int(365),
    },
    KeySpec {
        key: "retention.observational_days",
        kind: Kind::Int,
        default: Default::Int(30),
    },
    KeySpec {
        key: "retention.renew_on_use",
        kind: Kind::Bool,
        default: Default::Bool(false),
    },
    KeySpec {
        key: "retention.retired_days",
        kind: Kind::Int,
        default: Default::Int(30),
    },
    KeySpec {
        key: "decay.anchor_threshold",
        kind: Kind::Float,
        default: Default::Float(0.5),
    },
    KeySpec {
        key: "decay.grace_days",
        kind: Kind::Int,
        default: Default::Int(30),
    },
    KeySpec {
        key: "clusters.min_volume",
        kind: Kind::Int,
        default: Default::Int(10),
    },
    KeySpec {
        key: "clusters.similarity_threshold",
        kind: Kind::Float,
        default: Default::Float(0.8),
    },
    KeySpec {
        key: "hooks.pre_record",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "hooks.post_record",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "hooks.pre_prime",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "hooks.pre_prune",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "hooks.pre_compact",
        kind: Kind::Text,
        default: Default::Text(""),
    },
    KeySpec {
        key: "hooks.timeout_ms",
        kind: Kind::Int,
        default: Default::Int(30_000),
    },
    KeySpec {
        key: "ids.prefix_style",
        kind: Kind::Enum(&["declarative", "compact"]),
        default: Default::Text("declarative"),
    },
    KeySpec {
        key: "programs.glob",
        kind: Kind::Text,
        default: Default::Text("plan/*.md"),
    },
];
