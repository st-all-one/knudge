//! Proveniência PROV-lite (E19-T09/D207) com identidade de agente (D221).
//!
//! Modelo reduzido do W3C PROV: `entity` (o que a nota descreve), `activity` (como foi
//! produzida — `write`/`learn`/`import`/`derive`) e o agente que a produziu em três níveis:
//! `agent` (nome lógico), `agent_id` (instância/processo) e `agent_role`
//! (`main`/`subagent`) + `parent_agent_id`. Opcional e aditivo: campo ausente devolve
//! [`Provenance::default`] e os acessores aplicam os defaults de leitura — nunca reescrevem a
//! nota (D221: notas v2 continuam byte-idênticas).

use indexmap::IndexMap;

use crate::schema::{AgentRole, Frontmatter, Value};
use crate::{Error, Result};

/// Tamanho máximo de `agent_id`/`parent_agent_id` (contrato de identidade, D221).
pub const MAX_AGENT_ID_CHARS: usize = 128;

/// Proveniência de uma nota (todos os campos opcionais).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    /// Entidade descrita (rótulo/URI); ausente quando é a própria nota.
    pub entity: Option<String>,
    /// Atividade que produziu a nota.
    pub activity: Option<String>,
    /// Agente lógico que produziu a nota (`omp`, `ktu`, …).
    pub agent: Option<String>,
    /// Instância do agente que produziu a nota (processo/sessão; D221).
    pub agent_id: Option<String>,
    /// Papel da instância (`main`/`subagent`; D221).
    pub agent_role: Option<AgentRole>,
    /// Id do agente que criou o subagente (só em `subagent`; D221).
    pub parent_agent_id: Option<String>,
}

impl Provenance {
    /// `true` quando nenhum campo está presente (nada a emitir).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entity.is_none()
            && self.activity.is_none()
            && self.agent.is_none()
            && self.agent_id.is_none()
            && self.agent_role.is_none()
            && self.parent_agent_id.is_none()
    }

    /// `agent_id` efetivo na **leitura**: a instância, senão o nome lógico, senão `legacy`.
    #[must_use]
    pub fn agent_id_or_legacy(&self) -> &str {
        self.agent_id
            .as_deref()
            .or(self.agent.as_deref())
            .unwrap_or("legacy")
    }

    /// `agent_role` efetivo na leitura: explícito, senão [`AgentRole::Main`] (nota v2).
    #[must_use]
    pub fn agent_role_or_main(&self) -> AgentRole {
        self.agent_role.unwrap_or_default()
    }

    /// Converte em [`Value::Map`], omitindo campos ausentes (D05).
    ///
    /// A ordem é **contrato** (D221): os campos históricos (`entity`, `activity`, `agent`)
    /// primeiro; a identidade nova é anexada no fim, preservando os bytes de notas v2.
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
        if let Some(text) = &self.agent_id {
            map.insert("agent_id".to_string(), Value::Str(text.clone()));
        }
        if let Some(role) = self.agent_role {
            map.insert(
                "agent_role".to_string(),
                Value::Str(role.as_str().to_string()),
            );
        }
        if let Some(text) = &self.parent_agent_id {
            map.insert("parent_agent_id".to_string(), Value::Str(text.clone()));
        }
        Value::Map(map)
    }
}

/// Lê `provenance:` do frontmatter; campo ausente devolve [`Provenance::default`].
pub fn provenance(frontmatter: &Frontmatter) -> Result<Provenance> {
    provenance_from_value(frontmatter.get("provenance"))
}

/// Lê proveniência de um [`Value`] (mapa com campos opcionais de [`Provenance`]).
///
/// # Errors
/// `ErrorKind::Schema` se o valor não for mapa, se um campo não for string ou se
/// `agent_role` estiver fora do enum fechado.
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
        ("agent_id", &mut out.agent_id),
        ("parent_agent_id", &mut out.parent_agent_id),
    ] {
        match map.get(key) {
            None => {}
            Some(Value::Str(text)) if !text.trim().is_empty() => {
                *slot = Some(text.trim().to_string());
            }
            Some(_) => return Err(Error::schema(format!("provenance.{key} deve ser string"))),
        }
    }
    if let Some(value) = map.get("agent_role") {
        let text = value
            .as_str()
            .ok_or_else(|| Error::schema("provenance.agent_role deve ser string"))?;
        out.agent_role = Some(text.parse::<AgentRole>()?);
    }
    for (key, slot) in [
        ("agent_id", &out.agent_id),
        ("parent_agent_id", &out.parent_agent_id),
    ] {
        if let Some(text) = slot {
            validate_agent_id(key, text)?;
        }
    }
    Ok(out)
}

/// Valida um id de agente: não vazio e limitado (o nome do shard é derivado na escrita).
fn validate_agent_id(key: &str, text: &str) -> Result<()> {
    let len = text.chars().count();
    if len > MAX_AGENT_ID_CHARS {
        return Err(Error::schema(format!(
            "provenance.{key} excede {MAX_AGENT_ID_CHARS} caracteres"
        )));
    }
    if text.chars().any(char::is_control) {
        return Err(Error::schema(format!(
            "provenance.{key} contém caractere de controlo"
        )));
    }
    Ok(())
}
