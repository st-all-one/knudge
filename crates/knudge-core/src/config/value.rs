//! Valores tipados da configuração e tabelas ordenadas.
//!
//! A ordem de inserção é preservada (`IndexMap`) para que `config set` produza **diff mínimo**
//! (D63) e o arquivo gerado saia em ordem canônica.

use indexmap::IndexMap;

/// Tabela TOML: mapa ordenado de chave → valor.
pub type Table = IndexMap<String, ConfigValue>;

/// Valor de configuração (subconjunto tipado do TOML usado pelo knudge).
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// Booleano.
    Bool(bool),
    /// Inteiro decimal.
    Int(i64),
    /// Ponto flutuante.
    Float(f64),
    /// String.
    String(String),
    /// Lista homogênea ou heterogênea.
    Array(Vec<Self>),
    /// Tabela aninhada.
    Table(Table),
}

impl ConfigValue {
    /// Nome do tipo, para mensagens de erro.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "booleano",
            Self::Int(_) => "inteiro",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Array(_) => "lista",
            Self::Table(_) => "tabela",
        }
    }

    /// Vê como booleano.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Vê como inteiro.
    #[must_use]
    pub const fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Vê como float (inteiros coagem para float).
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "coerção de confiança: inteiros de config cabem no f64 do schema"
    )]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    /// Vê como string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Vê como lista.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    /// Vê como tabela.
    #[must_use]
    pub const fn as_table(&self) -> Option<&Table> {
        match self {
            Self::Table(value) => Some(value),
            _ => None,
        }
    }

    /// Vê como tabela mutável.
    #[must_use]
    pub const fn as_table_mut(&mut self) -> Option<&mut Table> {
        match self {
            Self::Table(value) => Some(value),
            _ => None,
        }
    }

    /// Renderiza como TOML inline (usado pelo emissor e por `config get/list`).
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Self::Bool(value) => if *value { "true" } else { "false" }.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => render_float(*value),
            Self::String(value) => quote(value),
            Self::Array(items) => {
                let parts: Vec<String> = items.iter().map(Self::render).collect();
                format!("[{}]", parts.join(", "))
            }
            Self::Table(table) => {
                let parts: Vec<String> = table
                    .iter()
                    .map(|(key, value)| format!("{} = {}", bare_or_quoted(key), value.render()))
                    .collect();
                format!("{{ {} }}", parts.join(", "))
            }
        }
    }
}

/// Renderiza um float garantindo parte fracionária ou expoente (TOML exige).
#[must_use]
pub fn render_float(value: f64) -> String {
    if !value.is_finite() {
        return "0.0".to_string();
    }
    let text = format!("{value}");
    if text.contains('.') || text.contains('e') || text.contains('E') {
        text
    } else {
        format!("{text}.0")
    }
}

/// Coloca uma chave TOML entre aspas quando não for uma chave "bare" válida.
#[must_use]
pub fn bare_or_quoted(key: &str) -> String {
    let is_bare = !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if is_bare { key.to_string() } else { quote(key) }
}

/// Escapa e coloca uma string entre aspas duplas (quoting estável — D63).
#[must_use]
pub fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c.is_control() => {
                let code = u32::from(c);
                out.push_str("\\u");
                for shift in [12, 8, 4, 0] {
                    let nibble = (code >> shift) & 0xf;
                    out.push(char::from_digit(nibble, 16).unwrap_or('0'));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_scalars_stably() {
        assert_eq!(ConfigValue::Bool(true).render(), "true");
        assert_eq!(ConfigValue::Int(10).render(), "10");
        assert_eq!(ConfigValue::Float(0.75).render(), "0.75");
        assert_eq!(ConfigValue::Float(1.0).render(), "1.0");
        assert_eq!(ConfigValue::String("a\"b".into()).render(), "\"a\\\"b\"");
        assert_eq!(
            ConfigValue::Array(vec![ConfigValue::Int(1), ConfigValue::Int(2)]).render(),
            "[1, 2]"
        );
    }

    #[test]
    fn coerces_int_to_float() {
        assert_eq!(ConfigValue::Int(3).as_float(), Some(3.0));
        assert_eq!(ConfigValue::String("3".into()).as_float(), None);
    }

    #[test]
    fn quotes_non_bare_keys() {
        assert_eq!(bare_or_quoted("dedup"), "dedup");
        assert_eq!(bare_or_quoted("a.b"), "\"a.b\"");
    }
}
