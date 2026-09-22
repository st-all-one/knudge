//! Valor do subconjunto TOON (D74/D75).
//!
//! Escalares, listas e **mapas ordenados** (D04). É a ponte entre o parser/emissor TOON e o
//! schema tipado (`Frontmatter`).

use indexmap::IndexMap;

/// Valor TOON.
///
/// Inteiros nunca são emitidos como `1.0` (D09): `Value::Float(1.0)` é emitido como `1` e
/// re-lido como `Value::Int(1)`.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// String (UTF-8 cru; aspas só quando necessário).
    Str(String),
    /// Inteiro de 64 bits.
    Int(i64),
    /// Ponto flutuante de 64 bits.
    Float(f64),
    /// Booleano.
    Bool(bool),
    /// Lista ordenada.
    List(Vec<Self>),
    /// Mapa com ordem de inserção preservada.
    Map(IndexMap<String, Self>),
}

impl Value {
    /// Constrói um mapa ordenado a partir de pares.
    #[must_use]
    pub fn map(pairs: impl IntoIterator<Item = (String, Self)>) -> Self {
        Self::Map(pairs.into_iter().collect())
    }

    /// Devolve a string, se for [`Value::Str`].
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Devolve o inteiro, se for [`Value::Int`].
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Devolve o número como `f64` (aceita inteiro e float).
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "int→float é exato até 2^53; domínio de confidence/duration é pequeno"
    )]
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Devolve o booleano, se for [`Value::Bool`].
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Devolve a lista, se for [`Value::List`].
    #[must_use]
    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// Devolve o mapa, se for [`Value::Map`].
    #[must_use]
    pub fn as_map(&self) -> Option<&IndexMap<String, Self>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }
}
