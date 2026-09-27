//! Métricas de fluxo e caminho crítico (E19/T11/R7/D205).
//!
//! Deriva do **log de eventos** (append-only), sem gravar verdade nova: `cycle time`
//! (`review − create`), `lead time` (tarefas abertas contam o tempo em voo) e `throughput`.
//! O **caminho crítico** é o maior caminho ponderado do DAG `depends_on` (PERT/CPM), com as
//! durações derivadas do log.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::schema::{EdgeKind, Value};
use crate::store::Event;

/// Ação canônica de fechamento de tarefa no log.
const REVIEW: &str = "review";

/// Fluxo de uma tarefa derivado do log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TaskFlow {
    /// Primeiro evento da nota (ms).
    pub created_ms: i64,
    /// Evento `task/review` (ms), quando fechada.
    pub closed_ms: Option<i64>,
}

impl TaskFlow {
    /// Cycle time (`close − create`); `None` para tarefa aberta.
    #[must_use]
    pub fn cycle_ms(self) -> Option<i64> {
        self.closed_ms
            .map(|closed| closed.saturating_sub(self.created_ms))
    }

    /// Lead time (`(close ou agora) − create`).
    #[must_use]
    pub fn lead_ms(self, now_ms: i64) -> i64 {
        self.closed_ms
            .unwrap_or(now_ms)
            .saturating_sub(self.created_ms)
    }

    /// `true` se a tarefa foi fechada (`task/review`).
    #[must_use]
    pub const fn is_closed(self) -> bool {
        self.closed_ms.is_some()
    }
}

/// Deriva o fluxo por nota do log (`task/review` fecha; qualquer evento cria).
#[must_use]
pub fn task_flows(events: &[Event]) -> BTreeMap<String, TaskFlow> {
    let mut flows: BTreeMap<String, TaskFlow> = BTreeMap::new();
    for event in events {
        let Some(id) = event.note_id.as_ref() else {
            continue;
        };
        let flow = flows.entry(id.clone()).or_insert(TaskFlow {
            created_ms: event.at,
            closed_ms: None,
        });
        if event.at < flow.created_ms {
            flow.created_ms = event.at;
        }
        if is_review(event) {
            flow.closed_ms = Some(
                flow.closed_ms
                    .map_or(event.at, |closed| closed.max(event.at)),
            );
        }
    }
    flows
}

/// Durações por nota (ms) para o caminho crítico: `lead time` no instante `now_ms`.
#[must_use]
pub fn durations_from_flows(
    flows: &BTreeMap<String, TaskFlow>,
    now_ms: i64,
) -> BTreeMap<String, i64> {
    flows
        .iter()
        .map(|(id, flow)| (id.clone(), flow.lead_ms(now_ms)))
        .collect()
}

/// Balde de throughput: início do intervalo (ms) e contagem de fechamentos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThroughputBucket {
    /// Início do intervalo (ms).
    pub start_ms: i64,
    /// Fechamentos no intervalo.
    pub closed: u32,
}

/// Fechamentos (`task/review`) agrupados em janelas de `window_ms`, em ordem cronológica.
#[must_use]
pub fn throughput(events: &[Event], window_ms: i64) -> Vec<ThroughputBucket> {
    let window = window_ms.max(1);
    let mut buckets: BTreeMap<i64, u32> = BTreeMap::new();
    for event in events.iter().filter(|event| is_review(event)) {
        let bucket = event.at.div_euclid(window).saturating_mul(window);
        let entry = buckets.entry(bucket).or_default();
        *entry = entry.saturating_add(1);
    }
    buckets
        .into_iter()
        .map(|(start_ms, closed)| ThroughputBucket { start_ms, closed })
        .collect()
}

/// Caminho crítico (PERT/CPM): maior soma de durações ao longo do DAG `depends_on`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CriticalPath {
    /// Ids ao longo do caminho (do item até suas dependências).
    pub ids: Vec<String>,
    /// Soma das durações (ms).
    pub total_ms: i64,
}

/// Calcula o caminho crítico do DAG `depends_on` com a duração por nó (0 quando ausente).
#[must_use]
pub fn critical_path(graph: &Graph, durations: &BTreeMap<String, i64>) -> CriticalPath {
    let mut memo: BTreeMap<String, CriticalPath> = BTreeMap::new();
    let mut best: Option<CriticalPath> = None;
    for id in graph.ids() {
        let mut visiting = BTreeSet::new();
        let candidate = longest_from(graph, durations, id, &mut memo, &mut visiting);
        if best
            .as_ref()
            .is_none_or(|current| better(&candidate, current))
        {
            best = Some(candidate);
        }
    }
    best.unwrap_or_default()
}

/// Maior caminho a partir de `id` (memoizado; quebra ciclos devolvendo vazio).
fn longest_from(
    graph: &Graph,
    durations: &BTreeMap<String, i64>,
    id: &str,
    memo: &mut BTreeMap<String, CriticalPath>,
    visiting: &mut BTreeSet<String>,
) -> CriticalPath {
    if let Some(cached) = memo.get(id) {
        return cached.clone();
    }
    if !visiting.insert(id.to_string()) {
        return CriticalPath::default();
    }
    let mut tail = CriticalPath::default();
    for target in graph.targets(id, EdgeKind::DependsOn) {
        let candidate = longest_from(graph, durations, target, memo, visiting);
        if better(&candidate, &tail) {
            tail = candidate;
        }
    }
    let _removed = visiting.remove(id);
    let duration = durations.get(id).copied().unwrap_or(0);
    let mut ids = Vec::with_capacity(tail.ids.len().saturating_add(1));
    ids.push(id.to_string());
    ids.extend(tail.ids);
    let result = CriticalPath {
        ids,
        total_ms: duration.saturating_add(tail.total_ms),
    };
    let _ignored = memo.insert(id.to_string(), result.clone());
    result
}

/// `true` se o evento é o fechamento canônico (`task/review`).
fn is_review(event: &Event) -> bool {
    event.op == "task" && event.data.get("action").and_then(Value::as_str) == Some(REVIEW)
}

/// Ordem do caminho: maior duração; empate pelo caminho mais longo, depois menor lexicográfico.
fn better(candidate: &CriticalPath, current: &CriticalPath) -> bool {
    match candidate.total_ms.cmp(&current.total_ms) {
        Ordering::Greater => true,
        Ordering::Equal => match candidate.ids.len().cmp(&current.ids.len()) {
            Ordering::Greater => true,
            Ordering::Equal => candidate.ids < current.ids,
            Ordering::Less => false,
        },
        Ordering::Less => false,
    }
}
