//! `kd maintenance` — doctor, audit, compact, learn e prune (E12-T01).

pub mod extra;
pub mod watch;

use knudge_core::Result;
use knudge_core::health::{AuditInput, DoctorInput, audit, doctor, doctor_fix};
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
        MaintenanceCommand::Compact { scope, corpus } => {
            extra::compact(session, scope.as_deref(), corpus)
        }
        MaintenanceCommand::Learn { scope, corpus } => {
            extra::learn_cmd(session, scope.as_deref(), corpus)
        }
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
            format!(
                "{} {} {}",
                if check.ok { "ok" } else { "fail" },
                check.id.as_str(),
                check.detail
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "healthy": report.is_healthy(),
        "checks": report.checks.iter().map(|check| json!({
            "id": check.id.as_str(),
            "ok": check.ok,
            "detail": check.detail,
            "fixable": check.fixable,
        })).collect::<Vec<_>>(),
        "fixed": report.fixed,
    });
    Ok(Output::new(text, data).with_warnings(report.warnings))
}
