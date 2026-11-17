//! `kd rewind` — estado/handoff ponto-no-tempo (E12-T01, D57/D88).

use std::collections::{BTreeMap, BTreeSet};

use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::handoff::{
    ContextStore, CorpusScope as HandoffScope, DEFAULT_BUDGET, RefFinding, RewindInput, RewindMode,
    RewindRequest, annotate_refs, derive_id, rewind, verify_refs,
};
use knudge_core::health::{AnchorStore, hash_file};
use knudge_core::lifecycle::{
    DEFAULT_TASK_CONFIRMATION, ProposalStore, ShelfLife, UsageStore, freshness_with,
};
use knudge_core::schema::Value;
use knudge_core::store::{Event, Note};
use knudge_core::task::{critical_path, durations_from_flows, task_flows, throughput};
use knudge_core::time::Timestamp;
use serde_json::json;

use crate::cli::{AiArea, AiArgs, RewindArgs};
use crate::commands::ai;
use crate::commands::corpus::CorpusScope;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::embedder;
const DAY_MS: i64 = 86_400_000;

/// Executa `kd rewind`.
///
/// # Errors
/// Propaga erros de índice/eventos e de `context_id` inválido.
#[allow(
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    reason = "handoff assembly"
)]
pub fn run(session: &Session, args: &RewindArgs) -> Result<Output> {
    let loaded = session.corpus()?;
    let index = &loaded.index;
    let graph = &loaded.graph;
    let corpus = corpus_of(args);
    let selection = corpus.select(index, graph)?;
    let (events, mut warnings) = session.events().read_all()?;
    let changed = session.changed_paths()?;
    let request = RewindRequest {
        mode: mode_of(args),
        budget: args.budget.unwrap_or(DEFAULT_BUDGET),
        since: parse::timestamp_opt(args.since.as_ref())?,
        until: parse::timestamp_opt(args.until.as_ref())?,
        resume: args.resume.clone(),
    };
    let contexts = ContextStore::new(session.fs_dyn(), session.knowledge_dir());
    let notes = &loaded.notes;
    let policy = ShelfLife::from_config(session.config());
    let pending = embedder::pending(session, notes)?;
    let usage = UsageStore::new(session.fs_dyn(), session.knowledge_dir()).index()?;
    let fresh = freshness_with(notes, session.now_ms(), &policy, pending, &usage)?;
    let bodies = bodies_of(notes);
    let input = RewindInput {
        index,
        graph,
        events: &events,
        changed_paths: &changed,
        freshness: fresh,
        bodies: &bodies,
        scope: HandoffScope {
            filter: selection.filter().clone(),
            allowed: selection.allowed().cloned(),
        },
        task_confirmation_weight: session
            .config()
            .get_float("recall.confirmation_from_tasks")
            .unwrap_or(DEFAULT_TASK_CONFIRMATION),
        now_ms: session.now_ms(),
    };
    let mut out = rewind(&input, &request, &contexts)?;
    warnings.extend(out.warnings.iter().cloned());
    let ids: Vec<String> = out.items.iter().map(|item| item.id.clone()).collect();
    warnings.extend(record_usage(session, &policy, &ids));
    let mut refs_json: Option<Vec<serde_json::Value>> = None;
    if args.with_refs && args.resume.is_none() {
        let cited: BTreeSet<String> = out.items.iter().map(|item| item.id.clone()).collect();
        let (body_hashes, anchor_hashes) = refs_maps(session, notes, &cited);
        out.text = annotate_refs(&out.text, &body_hashes, &anchor_hashes);
        out.context_id = derive_id(&out.text);
        contexts.save(&out.context_id, &out.text)?;
    }
    if args.check_refs {
        let anchors = AnchorStore::new(session.fs_dyn(), session.knowledge_dir());
        let findings = verify_refs(
            &session.store(),
            &anchors,
            session.fs_dyn(),
            session.project_root(),
            &out.text,
        )?;
        refs_json = Some(findings.iter().map(ref_json).collect());
        warnings.extend(findings.iter().map(RefFinding::render));
    }
    // Atividade de IA (opt-in)
    let mut activity_text = String::new();
    let mut activity_json: Option<serde_json::Value> = None;
    if args.with_activity {
        let ai_args = AiArgs {
            area: AiArea::SessionLog,
            recap: true,
            limit: Some(args.activity_limit),
            agent_id: if let Some(a) = &args.activity_agent {
                vec![a.clone()]
            } else {
                Vec::new()
            },
            ..AiArgs::default()
        };
        let source = ai::resolve_source(session, &ai_args);
        match source.load(&session.store()) {
            Ok((corpus, mut w)) => {
                warnings.append(&mut w);
                match ai::run_recap(&ai_args, &corpus) {
                    Ok((t, j)) => {
                        activity_text = format!("\n\n## Atividade recente\n{t}");
                        activity_json = Some(j);
                    }
                    Err(e) => warnings.push(e.to_string()),
                }
            }
            Err(e) => warnings.push(e.to_string()),
        }
    }
    let text = if activity_text.is_empty() {
        out.text.clone()
    } else {
        format!("{}{activity_text}", out.text)
    };
    let pending_proposals =
        ProposalStore::new(session.fs_dyn(), session.knowledge_dir()).count()?;
    let mut data = json!({
        "context_id": out.context_id,
        "text": text,
        "freshness": json!({
            "stale": fresh.stale,
            "expiring": fresh.expiring,
            "pending": fresh.pending,
        }),
        "pending_proposals": pending_proposals,
        "items": out.items.iter().map(|item| json!({
            "id": item.id,
            "statement": item.statement,
            "score": item.score,
            "tier": item.tier.as_str(),
        })).collect::<Vec<_>>(),
        "truncated": out.truncated,
        "dropped": out.dropped,
        "embeddings_pending": out.embeddings_pending,
        "flow": flow_json(&events, graph, session.now_ms()),
        "activity": activity_json,
    });
    if let Some(findings) = refs_json
        && let Some(map) = data.as_object_mut()
    {
        map.insert("refs".to_string(), json!(findings));
    }
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Mapas de hashes ostentados pelo handoff com refs (D227).
type RefsMaps = (BTreeMap<String, String>, BTreeMap<String, String>);
/// Hashes das notas citadas e das suas âncoras, para `rewind --with-refs` (D227).
fn refs_maps(session: &Session, notes: &[Note], cited: &BTreeSet<String>) -> RefsMaps {
    let mut bodies = BTreeMap::new();
    let mut anchors = BTreeMap::new();
    for note in notes {
        let Some(id) = note.frontmatter.get("id").and_then(Value::as_str) else {
            continue;
        };
        if !cited.contains(id) {
            continue;
        }
        if let Some(hash) = note.frontmatter.get("body_hash").and_then(Value::as_str) {
            bodies.insert(id.to_string(), hash.to_string());
        }
        let Some(list) = note.frontmatter.get("anchors") else {
            continue;
        };
        for anchor in list.as_list().into_iter().flatten() {
            let Some(path) = anchor.as_str() else {
                continue;
            };
            if let Ok(Some(hash)) = hash_file(session.fs_dyn(), &session.project_root().join(path))
            {
                anchors.insert(path.to_string(), hash);
            }
        }
    }
    (bodies, anchors)
}

/// Representação JSON de um achado de ref (aditiva no envelope do `rewind`).
fn ref_json(finding: &RefFinding) -> serde_json::Value {
    json!({
        "kind": finding.kind.as_str(),
        "name": finding.name,
        "old_hash": finding.old_hash,
        "new_hash": finding.new_hash,
        "status": finding.status.as_str(),
        "detail": finding.detail,
    })
}

/// Bloco `flow` aditivo do `rewind --json` (E19/T11/D205): throughput + caminho crítico.
fn flow_json(events: &[Event], graph: &Graph, now_ms: i64) -> serde_json::Value {
    let flows = task_flows(events);
    let durations = durations_from_flows(&flows, now_ms);
    let path = critical_path(graph, &durations);
    let buckets = throughput(events, 7_i64.saturating_mul(DAY_MS));
    json!({
        "throughput": buckets.iter().map(|bucket| json!({
            "start": Timestamp::from_millis(bucket.start_ms).to_rfc3339(),
            "closed": bucket.closed,
        })).collect::<Vec<_>>(),
        "critical_path": {
            "ids": path.ids,
            "total_ms": path.total_ms,
        },
    })
}

/// Credita o uso dos itens devolvidos (D154), se `renew_on_use` estiver ligado.
///
/// Devolve aviso (R33) quando o derivado não puder ser gravado; nunca derruba a leitura.
fn record_usage(session: &Session, policy: &ShelfLife, ids: &[String]) -> Option<String> {
    if !policy.renew_on_use || ids.is_empty() {
        return None;
    }
    let store = UsageStore::new(session.fs_dyn(), session.knowledge_dir());
    match store.record(ids, session.now_ms()) {
        Ok(_ignored) => None,
        Err(error) => Some(format!("uso: {error}")),
    }
}

/// Mapa id → corpo das notas (para o `rewind` anexar `foundational`/`decision`, D162).
fn bodies_of(notes: &[Note]) -> BTreeMap<String, String> {
    notes
        .iter()
        .filter_map(|note| {
            note.frontmatter
                .id()
                .ok()
                .map(|id| (id.to_string(), note.body.clone()))
        })
        .collect()
}

fn mode_of(args: &RewindArgs) -> RewindMode {
    if let Some(scope) = &args.scope {
        RewindMode::Scope(scope.clone())
    } else if !args.files.is_empty() {
        RewindMode::Files(args.files.clone())
    } else {
        RewindMode::Manifest
    }
}

/// Escopo de corpus do `rewind` (filtros + vizinhança) — D143.
fn corpus_of(args: &RewindArgs) -> CorpusScope {
    CorpusScope {
        types: args.types.clone(),
        classes: args.classes.clone(),
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
        around: args.around.clone(),
        depth: args.depth,
        universe: false,
        ..CorpusScope::default()
    }
}
