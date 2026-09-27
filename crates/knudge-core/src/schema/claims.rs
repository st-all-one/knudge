//! Claims tipados SPO (E19-T09/D207).
//!
//! Uma **claim** é uma afirmação atômica `(sujeito, relação, objeto)` declarada no frontmatter
//! (`claims:`), **sem substituir** o `statement`. Habilita inferência, contradição precisa e
//! dedup semântica. Os três campos são strings normalizadas (trim) e limitadas a
//! [`STATEMENT_MAX`](super::text::STATEMENT_MAX) escalares — não há vocabulário fechado de
//! relações (mundo aberto), mas a **forma** é fixa.

use indexmap::IndexMap;

use crate::schema::{Frontmatter, Value, text};
use crate::{Error, Result};

/// Afirmação atômica `(sujeito, relação, objeto)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Claim {
    /// Sujeito (id de nota ou rótulo de entidade).
    pub subject: String,
    /// Relação (predicado; mundo aberto).
    pub relation: String,
    /// Objeto (id de nota ou rótulo de entidade).
    pub object: String,
}

impl Claim {
    /// Cria uma claim com os campos já trimados.
    #[must_use]
    pub fn new(
        subject: impl Into<String>,
        relation: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into().trim().to_string(),
            relation: relation.into().trim().to_string(),
            object: object.into().trim().to_string(),
        }
    }

    /// Chave `(sujeito, relação)` usada para detectar contradição (objetos divergentes).
    #[must_use]
    pub fn key(&self) -> (&str, &str) {
        (self.subject.as_str(), self.relation.as_str())
    }

    /// Valida forma e limites dos três campos.
    pub fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("subject", &self.subject),
            ("relation", &self.relation),
            ("object", &self.object),
        ] {
            if value.is_empty() {
                return Err(Error::schema(format!("claim sem {name}")));
            }
            text::validate_statement(value)
                .map_err(|_| Error::schema(format!("claim {name} acima do limite")))?;
        }
        Ok(())
    }

    /// Converte em [`Value::Map`] na ordem canônica (`subject`, `relation`, `object`).
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut map = IndexMap::new();
        map.insert("subject".to_string(), Value::Str(self.subject.clone()));
        map.insert("relation".to_string(), Value::Str(self.relation.clone()));
        map.insert("object".to_string(), Value::Str(self.object.clone()));
        Value::Map(map)
    }
}

/// Lê as claims de `claims:` no frontmatter, na ordem declarada.
///
/// Campo ausente devolve `Vec` vazio. Item que não seja mapa com os três campos string é erro
/// de schema (write estrito); a leitura tolerante fica a cargo do chamador.
pub fn claims(frontmatter: &Frontmatter) -> Result<Vec<Claim>> {
    claims_from_value(frontmatter.get("claims"))
}

/// Lê claims de um [`Value`] (lista de mapas `subject`/`relation`/`object`).
pub fn claims_from_value(value: Option<&Value>) -> Result<Vec<Claim>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::List(items) = value else {
        return Err(Error::schema("claims deve ser uma lista"));
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let map = item
            .as_map()
            .ok_or_else(|| Error::schema("claim deve ser um mapa"))?;
        let subject = required(map, "subject")?;
        let relation = required(map, "relation")?;
        let object = required(map, "object")?;
        let claim = Claim::new(subject, relation, object);
        claim.validate()?;
        out.push(claim);
    }
    Ok(out)
}

/// Converte uma lista de claims em [`Value::List`] (vazio quando não há claims).
#[must_use]
pub fn claims_to_value(claims: &[Claim]) -> Value {
    Value::List(claims.iter().map(Claim::to_value).collect())
}

fn required<'a>(map: &'a IndexMap<String, Value>, key: &str) -> Result<&'a str> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::schema(format!("claim sem {key}")))
}
