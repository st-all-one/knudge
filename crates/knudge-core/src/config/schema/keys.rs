//! Catálogo de chaves canônicas da config (E04/E11).
//!
//! A ordem de [`KEYS`] é a ordem canônica de emissão do `config.toml` (D63).

use super::{Default, KeySpec, Kind};

/// Chaves canônicas, em ordem canônica (D63).
pub const KEYS: &[KeySpec] = &[
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
        key: "recall.default_limit",
        kind: Kind::Int,
        default: Default::Int(10),
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
    KeySpec {
        key: "embeddings.enabled",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.provider",
        kind: Kind::Enum(&["http", "lightweight", "none"]),
        default: Default::Text("http"),
    },
    KeySpec {
        key: "embeddings.model",
        kind: Kind::Text,
        default: Default::Text("sentence-transformers/msmarco-MiniLM-L12-cos-v5"),
    },
    KeySpec {
        key: "embeddings.revision",
        kind: Kind::Text,
        default: Default::Text("main"),
    },
    KeySpec {
        key: "embeddings.dimensions",
        kind: Kind::Int,
        default: Default::Int(384),
    },
    KeySpec {
        key: "embeddings.similarity",
        kind: Kind::Text,
        default: Default::Text("cosine"),
    },
    KeySpec {
        key: "embeddings.normalize",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.mode",
        kind: Kind::Enum(&["lazy", "eager", "manual"]),
        default: Default::Text("lazy"),
    },
    KeySpec {
        key: "embeddings.async",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.batch",
        kind: Kind::Int,
        default: Default::Int(32),
    },
    KeySpec {
        key: "embeddings.max_pending",
        kind: Kind::Int,
        default: Default::Int(1000),
    },
    KeySpec {
        key: "embeddings.cache",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.cache_max_bytes",
        kind: Kind::Int,
        default: Default::Int(33_554_432),
    },
    KeySpec {
        key: "embeddings.cache_ttl_days",
        kind: Kind::Int,
        default: Default::Int(30),
    },
    KeySpec {
        key: "embeddings.flush_ms",
        kind: Kind::Int,
        default: Default::Int(2000),
    },
    KeySpec {
        key: "embeddings.endpoint",
        kind: Kind::Text,
        default: Default::Text("http://127.0.0.1:8080/v1/embeddings"),
    },
    KeySpec {
        key: "embeddings.timeout_ms",
        kind: Kind::Int,
        default: Default::Int(30_000),
    },
    KeySpec {
        key: "embeddings.retries",
        kind: Kind::Int,
        default: Default::Int(2),
    },
    KeySpec {
        key: "embeddings.api_key_env",
        kind: Kind::Text,
        default: Default::Text("KNUDGE_EMBEDDING_API_KEY"),
    },
];
