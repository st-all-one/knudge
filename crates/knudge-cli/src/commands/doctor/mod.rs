//! `kd doctor` — diagnóstico completo do corpus e reparo do reversível (D163).
//!
//! Roda os **13 checks** do `doctor` **mais** a auditoria de integridade/arestas e apresenta um
//! caminho de resolução por achado (`próximos:` e `--explain`). Não é comando de rotina: o custo
//! da validação completa é aceito em troca de **consistência, garantia e resolubilidade**.

mod explain;
mod render;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::handoff::{ContextStore, RefFinding, is_valid_context_id, verify_refs};
use knudge_core::health::{AnchorStore, AuditInput, DoctorInput, audit, doctor, doctor_fix};
use knudge_core::lifecycle::{ShelfLife, UsageStore, freshness_with};

use crate::cli::DoctorArgs;
use crate::commands::embedder;
use crate::output::Output;
use crate::session::Session;
use serde_json::json;

/// Idade a partir da qual um lock é considerado abandonado.
const LOCK_STALE_MS: i64 = 30_000;

/// Executa `kd doctor [--fix] [--explain]`.
///
/// # Errors
/// Propaga erros de I/O/leitura do corpus e do derivado.
#[allow(
    clippy::too_many_lines,
    reason = "orquestra checks e o modo refs (D227)"
)]
pub fn run(session: &Session, args: &DoctorArgs) -> Result<Output> {
    if let Some(context_id) = &args.refs {
        return run_refs(session, context_id);
    }
    let store = session.store();
    let events = session.events();
    let thresholds = session.thresholds()?;
    let root = session.knowledge_dir();
    let project_root = session.project_root();
    let now_ms = session.now_ms();

    let pre = session.corpus()?;
    let report = {
        let input = DoctorInput {
            fs: session.fs_dyn(),
            root: &root,
            project_root,
            store: &store,
            events: &events,
            config: session.config(),
            graph: &pre.graph,
            now_ms,
            lock_stale_ms: LOCK_STALE_MS,
            thresholds: &thresholds,
        };
        if args.fix {
            doctor_fix(&input)?
        } else {
            doctor(&input)?
        }
    };

    // `--fix` grava no disco; índice/grafo/auditoria são relidos depois para refletir o reparo.
    // Sem reparo, o mesmo corpus serve às duas fases (uma só passada pelo store — E15-T02/O1).
    let post = if args.fix {
        Some(session.corpus()?)
    } else {
        None
    };
    let loaded = match &post {
        Some(corpus) => corpus,
        None => &pre,
    };
    let audit = {
        let input = AuditInput {
            fs: session.fs_dyn(),
            root: &root,
            project_root,
            store: &store,
            graph: &loaded.graph,
            index: &loaded.index,
            now_ms,
            lock_stale_ms: LOCK_STALE_MS,
            thresholds: &thresholds,
        };
        audit(&input)?
    };

    let suggestions = render::suggestions(&report, &audit);
    let detail = if args.explain {
        render::Detail::Explained
    } else {
        render::Detail::Summary
    };
    let text = render::text(&report, &audit, &suggestions, detail);
    let fresh = {
        let notes = &loaded.notes;
        let policy = ShelfLife::from_config(session.config());
        let pending = embedder::pending(session, notes)?;
        let usage = UsageStore::new(session.fs_dyn(), &root).index()?;
        freshness_with(notes, now_ms, &policy, pending, &usage)?
    };
    let mut data = render::json(&report, &audit, &suggestions, detail);
    if let Some(map) = data.as_object_mut() {
        map.insert(
            "freshness".to_string(),
            json!({
                "stale": fresh.stale,
                "expiring": fresh.expiring,
                "pending": fresh.pending,
            }),
        );
    }
    Ok(Output::new(text, data).with_warnings(report.warnings))
}

/// Verifica as refs de um handoff sem rodar os 13 checks (D227).
///
/// Achados vão para stderr via `warnings`; stdout fica vazio para não poluir o pipe.
fn run_refs(session: &Session, context_id: &str) -> Result<Output> {
    if !is_valid_context_id(context_id) {
        return Err(Error::invalid_input(format!(
            "context_id inválido: {context_id:?}"
        )));
    }
    let contexts = ContextStore::new(session.fs_dyn(), session.knowledge_dir());
    let Some(text) = contexts.load(context_id)? else {
        return Err(Error::not_found(format!("contexto ausente: {context_id}")));
    };
    let anchors = AnchorStore::new(session.fs_dyn(), session.knowledge_dir());
    let findings = verify_refs(
        &session.store(),
        &anchors,
        session.fs_dyn(),
        session.project_root(),
        &text,
    )?;
    let data = json!({
        "context_id": context_id,
        "findings": findings.iter().map(finding_json).collect::<Vec<_>>(),
    });
    let warnings = findings.iter().map(RefFinding::render).collect();
    Ok(Output::new(String::new(), data).with_warnings(warnings))
}

/// Representação JSON de um achado de ref.
fn finding_json(finding: &RefFinding) -> serde_json::Value {
    json!({
        "kind": finding.kind.as_str(),
        "name": finding.name,
        "old_hash": finding.old_hash,
        "new_hash": finding.new_hash,
        "status": finding.status.as_str(),
        "detail": finding.detail,
    })
}
