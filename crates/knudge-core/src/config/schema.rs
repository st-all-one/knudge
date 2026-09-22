//! Schema da configuração: chaves conhecidas, tipos, defaults e validação (D64).

use crate::config::table::{flatten, set_path};
use crate::config::value::{ConfigValue, Table};

/// Tipo esperado de uma folha de configuração.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Booleano.
    Bool,
    /// Inteiro.
    Int,
    /// Float.
    Float,
    /// String livre.
    Text,
    /// String restrita a um conjunto fechado.
    Enum(&'static [&'static str]),
}

/// Valor default de uma chave (const-friendly).
#[derive(Debug, Clone, Copy)]
pub enum Default {
    /// Booleano.
    Bool(bool),
    /// Inteiro.
    Int(i64),
    /// Float.
    Float(f64),
    /// Texto.
    Text(&'static str),
}

/// Descrição de uma chave canônica.
#[derive(Debug, Clone, Copy)]
pub struct KeySpec {
    /// Caminho pontilhado (`secao.chave`).
    pub key: &'static str,
    /// Tipo esperado.
    pub kind: Kind,
    /// Valor default.
    pub default: Default,
}

/// Prefixo da seção de segredos (só no global — D91).
pub const SECRETS_PREFIX: &str = "secrets";

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
        key: "mcp.observation_mode",
        kind: Kind::Bool,
        default: Default::Bool(true),
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
        key: "ids.prefix_style",
        kind: Kind::Enum(&["declarative", "compact"]),
        default: Default::Text("declarative"),
    },
    KeySpec {
        key: "embeddings.enabled",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.provider",
        kind: Kind::Enum(&["local", "http", "lightweight", "none"]),
        default: Default::Text("local"),
    },
    KeySpec {
        key: "embeddings.model",
        kind: Kind::Text,
        default: Default::Text("sentence-transformers/msmarco-MiniLM-L12-cos-v5"),
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
        key: "embeddings.cache",
        kind: Kind::Bool,
        default: Default::Bool(true),
    },
    KeySpec {
        key: "embeddings.flush_ms",
        kind: Kind::Int,
        default: Default::Int(2000),
    },
];

/// Devolve a especificação de uma chave canônica (sem `secrets`).
#[must_use]
pub fn spec(key: &str) -> Option<&'static KeySpec> {
    KEYS.iter().find(|entry| entry.key == key)
}

/// Converte o default de uma especificação em valor.
#[must_use]
pub fn default_value(spec: &KeySpec) -> ConfigValue {
    match spec.default {
        Default::Bool(value) => ConfigValue::Bool(value),
        Default::Int(value) => ConfigValue::Int(value),
        Default::Float(value) => ConfigValue::Float(value),
        Default::Text(value) => ConfigValue::String(value.to_string()),
    }
}

/// Constrói a tabela default na ordem canônica.
#[must_use]
pub fn default_table() -> Table {
    let mut table = Table::new();
    for entry in KEYS {
        let parts: Vec<&str> = entry.key.split('.').collect();
        // `set_path` só falha se um ancestral não for tabela, o que não ocorre aqui.
        let _ignored = set_path(&mut table, &parts, default_value(entry));
    }
    table
}

/// `true` se a chave pertence à seção de segredos.
#[must_use]
pub fn is_secret(key: &str) -> bool {
    key == SECRETS_PREFIX || key.starts_with(&format!("{SECRETS_PREFIX}."))
}

/// Valida uma folha contra o schema.
///
/// # Errors
/// Retorna mensagem legível quando a chave é desconhecida ou o tipo/enum não bate.
pub fn validate_leaf(key: &str, value: &ConfigValue) -> Result<(), String> {
    if is_secret(key) {
        return match value {
            ConfigValue::String(_) => Ok(()),
            other => Err(format!(
                "segredo `{key}` deve ser string, não {}",
                other.type_name()
            )),
        };
    }
    let spec = spec(key).ok_or_else(|| format!("chave desconhecida: `{key}`"))?;
    let ok = match spec.kind {
        Kind::Bool => value.as_bool().is_some(),
        Kind::Int => value.as_int().is_some(),
        Kind::Float => value.as_float().is_some(),
        Kind::Text => value.as_str().is_some(),
        Kind::Enum(allowed) => value.as_str().is_some_and(|s| allowed.contains(&s)),
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "`{key}` espera {}, recebeu {}",
            describe_kind(&spec.kind),
            value.type_name()
        ))
    }
}

/// Valida todas as folhas de uma tabela.
///
/// # Errors
/// Retorna o primeiro problema encontrado.
pub fn validate_table(table: &Table) -> Result<(), String> {
    for (key, value) in flatten(table) {
        validate_leaf(&key, value)?;
    }
    Ok(())
}

fn describe_kind(kind: &Kind) -> String {
    match kind {
        Kind::Bool => "booleano".to_string(),
        Kind::Int => "inteiro".to_string(),
        Kind::Float => "float".to_string(),
        Kind::Text => "string".to_string(),
        Kind::Enum(allowed) => format!("um de {}", allowed.join("|")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate_and_are_in_canonical_order() {
        let table = default_table();
        assert!(validate_table(&table).is_ok());
        let keys: Vec<String> = flatten(&table).into_iter().map(|(k, _)| k).collect();
        assert_eq!(
            keys.first().map(String::as_str),
            Some("knowledge.persist_in_project")
        );
        assert_eq!(keys.last().map(String::as_str), Some("embeddings.flush_ms"));
    }

    #[test]
    fn rejects_unknown_key_and_bad_enum() {
        assert!(validate_leaf("nope", &ConfigValue::Bool(true)).is_err());
        assert!(validate_leaf("ids.prefix_style", &ConfigValue::String("x".into())).is_err());
        assert!(validate_leaf("ids.prefix_style", &ConfigValue::String("compact".into())).is_ok());
    }

    #[test]
    fn secrets_must_be_strings() {
        assert!(validate_leaf("secrets.token", &ConfigValue::String("x".into())).is_ok());
        assert!(validate_leaf("secrets.token", &ConfigValue::Int(1)).is_err());
    }
}
