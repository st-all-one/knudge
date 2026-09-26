//! `kd doctor` — diagnóstico completo do corpus e reparo do reversível (D163).
//!
//! Roda os **13 checks** do `doctor` **mais** a auditoria de integridade/arestas e apresenta um
//! caminho de resolução por achado (`próximos:` e `--explain`). Não é comando de rotina: o custo
//! da validação completa é aceito em troca de **consistência, garantia e resolubilidade**.

mod explain;
mod render;

use knudge_core::Result;
use knudge_core::health::{AuditInput, DoctorInput, audit, doctor, doctor_fix};

use crate::cli::DoctorArgs;
use crate::output::Output;
use crate::session::Session;

/// Idade a partir da qual um lock é considerado abandonado.
const LOCK_STALE_MS: i64 = 30_000;

/// Executa `kd doctor [--fix] [--explain]`.
///
/// # Errors
/// Propaga erros de I/O/leitura do corpus e do derivado.
pub fn run(session: &Session, args: &DoctorArgs) -> Result<Output> {
    let store = session.store();
    let events = session.events();
    let thresholds = session.thresholds()?;
    let root = session.knowledge_dir();
    let project_root = session.project_root();
    let now_ms = session.now_ms();

    let report = {
        let graph = session.graph()?;
        let input = DoctorInput {
            fs: session.fs_dyn(),
            root: &root,
            project_root,
            store: &store,
            events: &events,
            config: session.config(),
            graph: &graph,
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
    let index = session.index()?;
    let graph = session.graph()?;
    let audit = {
        let input = AuditInput {
            fs: session.fs_dyn(),
            root: &root,
            project_root,
            store: &store,
            graph: &graph,
            index: &index,
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
    let data = render::json(&report, &audit, &suggestions, detail);
    Ok(Output::new(text, data).with_warnings(report.warnings))
}
