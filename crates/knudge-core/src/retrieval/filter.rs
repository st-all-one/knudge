//! Filtros determinísticos aplicados **antes** da estatística (D41/D53).
//!
//! Filtrar é O(1) por nota e reduz `N` antes do BM25: primeiro estrutura, depois similaridade.
//! O `scope` é resolvido no grafo (via `depends_on` transitivo) e não entra aqui.

use crate::Result;
use crate::retrieval::anchor::glob_match;
use crate::schema::{
    AgentRole, Classification, Frontmatter, NoteType, Scope, Status, Value, outcome_stats,
    provenance,
};
use crate::time::Timestamp;

/// Metadados de uma nota usados por filtros, `why` e boost.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    /// Id da nota.
    pub id: String,
    /// Tipo fechado.
    pub note_type: NoteType,
    /// Escopo de tarefa, quando houver (nível; D113).
    pub scope: Option<Scope>,
    /// Classificação de maturidade.
    pub classification: Classification,
    /// Estado.
    pub status: Status,
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Âncoras declaradas (paths/globs).
    pub anchors: Vec<String>,
    /// Agente que produziu a nota (`provenance.agent`, D207) — `None` se ausente.
    pub agent: Option<String>,
    /// Instância do agente (`provenance.agent_id`, D221) — `None` se ausente.
    pub agent_id: Option<String>,
    /// Papel da instância (`provenance.agent_role`, D221) — `None` se ausente.
    pub agent_role: Option<AgentRole>,
    /// Agente que criou o subagente (`provenance.parent_agent_id`, D221).
    pub parent_agent_id: Option<String>,
    /// Atividade que produziu a nota (`provenance.activity`, D207) — `None` se ausente.
    pub activity: Option<String>,
    /// `created_at` em milissegundos desde a época (0 se ilegível).
    pub created_ms: i64,
    /// Confirmação derivada de `outcomes`: `success + partial*0.5` (D38).
    pub confirmation: f64,
    /// Falhas derivadas de `outcomes`: `failure + abandoned + partial*0.5` (D189).
    pub failures: f64,
}

impl Meta {
    /// Extrai os metadados de um frontmatter válido.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
    pub fn from_frontmatter(frontmatter: &Frontmatter) -> Result<Self> {
        let outcomes = outcome_stats(frontmatter);
        let provenance = provenance(frontmatter)?;
        Ok(Self {
            id: frontmatter.id()?.to_string(),
            note_type: frontmatter.note_type()?,
            scope: frontmatter.scope()?,
            classification: frontmatter.classification()?,
            status: frontmatter.status()?,
            tags: string_list(frontmatter, "tags"),
            anchors: string_list(frontmatter, "anchors"),
            agent: provenance.agent,
            agent_id: provenance.agent_id,
            agent_role: provenance.agent_role,
            parent_agent_id: provenance.parent_agent_id,
            activity: provenance.activity,
            created_ms: created_ms(frontmatter),
            confirmation: outcomes.successes,
            failures: outcomes.failures,
        })
    }
}

/// Filtros estruturais da consulta.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Tipos aceitos (vazio = todos).
    pub types: Vec<NoteType>,
    /// Classificações aceitas (vazio = todas).
    pub classifications: Vec<Classification>,
    /// Status aceitos (vazio = todos).
    pub statuses: Vec<Status>,
    /// Tags exigidas (basta uma).
    pub tags: Vec<String>,
    /// Âncoras (path/glob) exigidas (basta uma). O match aceita as duas direções: o
    /// valor pedido como glob sobre a âncora da nota (`V2/**` casa `V2/x.rs`) ou a âncora
    /// da nota como glob sobre o valor pedido (`src/**` casa `src/x.rs`) — alinhado ao
    /// canal de âncoras (`retrieval/anchor.rs`) e a `rewind --files`.
    pub anchors: Vec<String>,
    /// Agentes exigidos (basta um), por igualdade com `provenance.agent` (D218).
    pub agents: Vec<String>,
    /// Instâncias exigidas (basta uma), por igualdade com `provenance.agent_id` (D221).
    pub agent_ids: Vec<String>,
    /// Papéis exigidos (basta um), por igualdade com `provenance.agent_role` (D221).
    pub agent_roles: Vec<AgentRole>,
    /// Atividades exigidas (basta uma), por igualdade com `provenance.activity` (D218).
    pub activities: Vec<String>,
    /// `created_at >=` (ms), se definido.
    pub created_since: Option<i64>,
    /// `created_at <=` (ms), se definido.
    pub created_until: Option<i64>,
}

impl Filter {
    /// Cria um filtro vazio (aceita tudo).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` se nenhum critério foi definido.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
            && self.classifications.is_empty()
            && self.statuses.is_empty()
            && self.tags.is_empty()
            && self.anchors.is_empty()
            && self.agents.is_empty()
            && self.agent_ids.is_empty()
            && self.agent_roles.is_empty()
            && self.activities.is_empty()
            && self.created_since.is_none()
            && self.created_until.is_none()
    }
    /// `true` se a nota passa por todos os critérios definidos.
    #[must_use]
    pub fn matches(&self, meta: &Meta) -> bool {
        (self.types.is_empty() || self.types.contains(&meta.note_type))
            && (self.classifications.is_empty()
                || self.classifications.contains(&meta.classification))
            && (self.statuses.is_empty() || self.statuses.contains(&meta.status))
            && (self.tags.is_empty() || meta.tags.iter().any(|tag| self.tags.contains(tag)))
            && (self.agents.is_empty()
                || meta
                    .agent
                    .as_ref()
                    .is_some_and(|agent| self.agents.contains(agent)))
            && (self.agent_ids.is_empty()
                || self
                    .agent_ids
                    .iter()
                    .any(|id| id == effective_agent_id(meta)))
            && (self.agent_roles.is_empty()
                || self.agent_roles.contains(&effective_agent_role(meta)))
            && (self.activities.is_empty()
                || meta
                    .activity
                    .as_ref()
                    .is_some_and(|activity| self.activities.contains(activity)))
            && (self.anchors.is_empty()
                || meta
                    .anchors
                    .iter()
                    .any(|anchor| self.anchors.iter().any(|req| anchor_matches(req, anchor))))
            && self.created_since.is_none_or(|s| meta.created_ms >= s)
            && self.created_until.is_none_or(|u| meta.created_ms <= u)
    }
}

/// `agent_id` efetivo nos filtros: instância, nome lógico ou `legacy` (D221).
fn effective_agent_id(meta: &Meta) -> &str {
    meta.agent_id
        .as_deref()
        .or(meta.agent.as_deref())
        .unwrap_or("legacy")
}

/// `agent_role` efetivo nos filtros: ausente ⇒ `main` (D221).
fn effective_agent_role(meta: &Meta) -> AgentRole {
    meta.agent_role.unwrap_or_default()
}

/// `true` se o valor pedido casa a âncora da nota em qualquer direção (D81).
///
/// Aceita o pedido como glob (`V2/**` casa `V2/x.rs`) e a âncora como glob
/// (`src/**` casa `src/x.rs`), para que `ask --anchor <path>` se comporte como o canal.
pub(crate) fn anchor_matches(requested: &str, anchor: &str) -> bool {
    glob_match(requested, anchor) || glob_match(anchor, requested)
}

fn string_list(frontmatter: &Frontmatter, key: &str) -> Vec<String> {
    frontmatter
        .string_list(key)
        .map(|items| items.into_iter().map(str::to_string).collect())
        .unwrap_or_default()
}

fn created_ms(frontmatter: &Frontmatter) -> i64 {
    match frontmatter.get("created_at") {
        Some(Value::Int(ms)) => *ms,
        Some(Value::Str(text)) => text.parse::<Timestamp>().map_or(0, Timestamp::as_millis),
        _ => 0,
    }
}
