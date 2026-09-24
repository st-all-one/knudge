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

mod keys;
mod keys_embeddings;

pub use keys::KEYS;

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
    for entry in KEYS.iter() {
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
