//! `kd task flow` — métricas de fluxo e caminho crítico (E19/T11/D205).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::task::{
    CriticalPath, TaskFlow, ThroughputBucket, critical_path, durations_from_flows, task_flows,
    throughput,
};
use knudge_core::time::Timestamp;
use serde_json::{Value, json};

use crate::cli::TaskFlowArgs;
use crate::output::Output;
use crate::session::Session;

/// Milissegundos em um dia.
const DAY_MS: i64 = 86_400_000;

/// Executa `kd task flow`.
///
/// # Errors
/// Propaga erros de leitura do store/eventos e de derivação do grafo.
pub(super) fn run(session: &Session, args: &TaskFlowArgs) -> Result<Output> {
    let notes = session.notes()?;
    let graph = Graph::from_notes_ref(&notes)?;
    let (events, warnings) = session.events().read_all()?;
    let now_ms = session.now_ms();
    let flows = task_flows(&events);
    let durations = durations_from_flows(&flows, now_ms);
    let path = critical_path(&graph, &durations);
    let window = i64::from(args.window_days).saturating_mul(DAY_MS);
    let buckets = throughput(&events, window);
    let summary = Summary::of(&flows, now_ms);
    let text = render_text(&summary, &buckets, &path);
    let data = render_json(&summary, &buckets, &path, &flows, now_ms);
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Resumo agregado do fluxo.
struct Summary {
    tasks: usize,
    closed: usize,
    cycle_mean_ms: i64,
    lead_mean_ms: i64,
}

impl Summary {
    fn of(flows: &BTreeMap<String, TaskFlow>, now_ms: i64) -> Self {
        let closed: Vec<i64> = flows.values().filter_map(|flow| flow.cycle_ms()).collect();
        let lead: Vec<i64> = flows.values().map(|flow| flow.lead_ms(now_ms)).collect();
        Self {
            tasks: flows.len(),
            closed: closed.len(),
            cycle_mean_ms: average(&closed),
            lead_mean_ms: average(&lead),
        }
    }
}

/// Linhas de texto (pipe) do `task flow`.
fn render_text(summary: &Summary, buckets: &[ThroughputBucket], path: &CriticalPath) -> String {
    let mut lines = vec![format!(
        "resumo|tarefas={}|fechadas={}|cycle_medio={}|lead_medio={}",
        summary.tasks, summary.closed, summary.cycle_mean_ms, summary.lead_mean_ms
    )];
    for bucket in buckets {
        lines.push(format!(
            "throughput|{}|{}",
            Timestamp::from_millis(bucket.start_ms).to_rfc3339(),
            bucket.closed
        ));
    }
    if !path.ids.is_empty() {
        lines.push(format!(
            "critico|{}|total_ms={}",
            path.ids.join(" -> "),
            path.total_ms
        ));
    }
    lines.join("\n")
}

/// Envelope `--json` (aditivo).
fn render_json(
    summary: &Summary,
    buckets: &[ThroughputBucket],
    path: &CriticalPath,
    flows: &BTreeMap<String, TaskFlow>,
    now_ms: i64,
) -> Value {
    let tasks: BTreeMap<&String, Value> = flows
        .iter()
        .map(|(id, flow)| {
            (
                id,
                json!({
                    "cycle_ms": flow.cycle_ms(),
                    "lead_ms": flow.lead_ms(now_ms),
                }),
            )
        })
        .collect();
    json!({
        "summary": {
            "tasks": summary.tasks,
            "closed": summary.closed,
            "cycle_mean_ms": summary.cycle_mean_ms,
            "lead_mean_ms": summary.lead_mean_ms,
        },
        "throughput": buckets.iter().map(|bucket| json!({
            "start": Timestamp::from_millis(bucket.start_ms).to_rfc3339(),
            "closed": bucket.closed,
        })).collect::<Vec<_>>(),
        "critical_path": {
            "ids": path.ids,
            "total_ms": path.total_ms,
        },
        "tasks": tasks,
    })
}

/// Média inteira (truncada) de uma amostra; 0 quando vazia.
fn average(values: &[i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }
    let total = values.iter().copied().fold(0_i64, i64::saturating_add);
    let count = i64::try_from(values.len()).unwrap_or(1).max(1);
    total.checked_div(count).unwrap_or(0)
}
