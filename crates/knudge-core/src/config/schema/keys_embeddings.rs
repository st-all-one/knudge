//! Chaves canônicas de embeddings (E11) — recortadas para manter o teto de 300 linhas.

use super::{Default, KeySpec, Kind};

/// Chaves de `embeddings.*`, em ordem canônica (D63).
pub(super) const EMBEDDINGS: &[KeySpec] = &[
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
        default: Default::Text("ibm-granite/granite-embedding-97m-multilingual-r2"),
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
        kind: Kind::Enum(&["lazy", "manual"]),
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
        key: "embeddings.version_cache",
        kind: Kind::Bool,
        default: Default::Bool(false),
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
