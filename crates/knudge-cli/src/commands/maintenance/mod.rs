//! `kd maintenance` — doctor, audit, compact, learn e prune (E12-T01).

pub mod extra;
pub mod proposals;
pub mod watch;

use knudge_core::Result;
use knudge_core::health::{AuditInput, DoctorInput, audit, doctor, doctor_fix};
use knudge_core::schema::EdgeKind;
use serde_json::json;

use crate::cli::MaintenanceCommand;
use crate::output::Output;
use crate::session::Session;

/// Modo do `doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorMode {
    /// Relatório de saúde.
    Report,
    /// Relatório + reparo reversível.
    Fix,
    /// Auditoria de grafo/arestas.
    Audit,
}

/// Executa `kd maintenance <subcomando>`.
///
/// # Errors
/// Propaga erros de leitura/escrita do domínio.
pub fn run(session: &Session, command: &MaintenanceCommand) -> Result<Output> {
    match command {
        MaintenanceCommand::Doctor { fix, audit: mode } => doctor_cmd(
            session,
            if *mode {
                DoctorMode::Audit
            } else if *fix {
                DoctorMode::Fix
            } else {
                DoctorMode::Report
            },
        ),
        MaintenanceCommand::Compact {
            scope,
            corpus,
            verify,
        } => extra::compact(
            session,
            scope.as_deref(),
            corpus,
            if *verify {
                proposals::VerifyMode::Run
            } else {
                proposals::VerifyMode::Skip
            },
        ),
        MaintenanceCommand::Learn {
            scope,
            corpus,
            verify,
        } => extra::learn_cmd(
            session,
            scope.as_deref(),
            corpus,
            if *verify {
                proposals::VerifyMode::Run
            } else {
                proposals::VerifyMode::Skip
            },
        ),
        MaintenanceCommand::Prune { scope, corpus, .. } => {
            extra::prune(session, scope.as_deref(), corpus)
        }
        MaintenanceCommand::WatchService(args) => watch::run(session, args),
    }
}

fn doctor_cmd(session: &Session, mode: DoctorMode) -> Result<Output> {
    match mode {
        DoctorMode::Audit => audit_report(session),
        DoctorMode::Report | DoctorMode::Fix => doctor_report(session, mode),
    }
}

fn audit_report(session: &Session) -> Result<Output> {
    let store = session.store();
    let index = session.index()?;
    let graph = session.graph()?;
    let root = session.knowledge_dir();
    let thresholds = session.thresholds()?;
    let input = AuditInput {
        fs: session.fs_dyn(),
        root: &root,
        project_root: session.project_root(),
        store: &store,
        graph: &graph,
        index: &index,
        now_ms: session.now_ms(),
        lock_stale_ms: 30_000,
        thresholds: &thresholds,
    };
    let report = audit(&input)?;
    let text = format!(
        "audit: {} problema(s){}",
        report.total(),
        if report.is_clean() { " (limpo)" } else { "" }
    );
    let data = json!({
        "clean": report.is_clean(),
        "total": report.total(),
        "integrity": report.integrity.len(),
        "supersession_cycles": report.supersession_cycles.len(),
        "dependency_cycles": report.dependency_cycles.len(),
        "broken_anchors": report.broken_anchors.len(),
        "duplicates": report.duplicates.len(),
        "missing_edges": report.missing_edges.len(),
        "stale_locks": report.stale_locks.len(),
        // Detalhes (ids/pares) para o agente agir sem rodar `compact --universe` à parte.
        "integrity_issues": report.integrity.iter().map(|issue| json!({
            "kind": issue.kind.as_str(),
            "from": issue.from,
            "to": issue.to,
            "edge": issue.edge.map(EdgeKind::as_str),
        })).collect::<Vec<_>>(),
        "supersession_cycle_details": report.supersession_cycles,
        "dependency_cycle_details": report.dependency_cycles,
        "broken_anchor_details": report.broken_anchors.iter().map(|anchor| json!({
            "id": anchor.id, "anchor": anchor.anchor,
        })).collect::<Vec<_>>(),
        "duplicate_pairs": report.duplicates.iter().map(|dup| json!({
            "keep": dup.keep, "drop": dup.drop, "score": dup.score,
        })).collect::<Vec<_>>(),
        "missing_edge_details": report.missing_edges.iter().map(|edge| json!({
            "id": edge.id, "kind": edge.kind, "targets": edge.targets, "reason": edge.reason,
        })).collect::<Vec<_>>(),
        "stale_lock_details": report.stale_locks.iter().map(|lock| json!({
            "path": lock.path, "age_ms": lock.age_ms,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

fn doctor_report(session: &Session, mode: DoctorMode) -> Result<Output> {
    let store = session.store();
    let events = session.events();
    let graph = session.graph()?;
    let root = session.knowledge_dir();
    let thresholds = session.thresholds()?;
    let input = DoctorInput {
        fs: session.fs_dyn(),
        root: &root,
        project_root: session.project_root(),
        store: &store,
        events: &events,
        config: session.config(),
        graph: &graph,
        now_ms: session.now_ms(),
        lock_stale_ms: 30_000,
        thresholds: &thresholds,
    };
    let report = if mode == DoctorMode::Fix {
        doctor_fix(&input)?
    } else {
        doctor(&input)?
    };
    let text = report
        .checks
        .iter()
        .map(|check| {
            let status = if check.ok {
                "ok"
            } else if check.is_warning() {
                "warn"
            } else {
                "fail"
            };
            format!("{status} {} {}", check.id.as_str(), check.detail)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "healthy": report.is_healthy(),
        "checks": report.checks.iter().map(|check| json!({
            "id": check.id.as_str(),
            "ok": check.ok,
            "warn": check.is_warning(),
            "detail": check.detail,
            "fixable": check.fixable,
        })).collect::<Vec<_>>(),
        "fixed": report.fixed,
    });
    Ok(Output::new(text, data).with_warnings(report.warnings))
}
