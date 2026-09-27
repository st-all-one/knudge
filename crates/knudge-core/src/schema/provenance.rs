//! Proveniência PROV-lite (E19-T09/D207).
//!
//! Modelo reduzido do W3C PROV: `entity` (o que a nota descreve), `activity` (como foi
//! produzida — `write`/`learn`/`import`/`derive`) e `agent` (quem produziu — agente, modelo,
//! pessoa). Opcional e aditivo: campo ausente devolve [`Provenance::default`]. Complementa
//! `source` (origem externa) e `anchors` (código), sem os substituir.

use indexmap::IndexMap;

use crate::schema::{Frontmatter, Value};
use crate::{Error, Result};

/// Proveniência de uma nota (todos os campos opcionais).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    /// Entidade descrita (rótulo/URI); ausente quando é a própria nota.
    pub entity: Option<String>,
    /// Atividade que produziu a nota.
    pub activity: Option<String>,
    /// Agente que produziu a nota.
    pub agent: Option<String>,
}

impl Provenance {
    /// `true` quando nenhum campo está presente (nada a emitir).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entity.is_none() && self.activity.is_none() && self.agent.is_none()
    }

    /// Converte em [`Value::Map`], omitindo campos ausentes (D05).
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut map = IndexMap::new();
        for (key, value) in [
            ("entity", &self.entity),
            ("activity", &self.activity),
            ("agent", &self.agent),
        ] {
            if let Some(text) = value {
                map.insert(key.to_string(), Value::Str(text.clone()));
            }
        }
        Value::Map(map)
    }
}

/// Lê `provenance:` do frontmatter; campo ausente devolve [`Provenance::default`].
pub fn provenance(frontmatter: &Frontmatter) -> Result<Provenance> {
    provenance_from_value(frontmatter.get("provenance"))
}

/// Lê proveniência de um [`Value`] (mapa com `entity`/`activity`/`agent` opcionais).
pub fn provenance_from_value(value: Option<&Value>) -> Result<Provenance> {
    let Some(value) = value else {
        return Ok(Provenance::default());
    };
    let map = value
        .as_map()
        .ok_or_else(|| Error::schema("provenance deve ser um mapa"))?;
    let mut out = Provenance::default();
    for (key, slot) in [
        ("entity", &mut out.entity),
        ("activity", &mut out.activity),
        ("agent", &mut out.agent),
    ] {
        match map.get(key) {
            None => {}
            Some(Value::Str(text)) if !text.trim().is_empty() => {
                *slot = Some(text.trim().to_string());
            }
            Some(_) => return Err(Error::schema(format!("provenance.{key} deve ser string"))),
        }
    }
    Ok(out)
}
