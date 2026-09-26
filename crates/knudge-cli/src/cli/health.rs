//! Argumentos dos verbos de saúde/fila do `kd`: `doctor` (D163) e `drain` (D170).

use clap::Args;

/// Argumentos de `kd doctor`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Corrige o que for reversível (idempotente).
    #[arg(long)]
    pub fix: bool,
    /// Detalha cada achado (`esperado` × `encontrado` × `ação`).
    #[arg(long)]
    pub explain: bool,
}

/// Argumentos de `kd drain`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "ações mutuamente exclusivas da CLI (D170)"
)]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct DrainArgs {
    /// Digere a fila em lotes até esvaziar/estagnar (log mínimo).
    #[arg(long, conflicts_with = "status")]
    pub digest: bool,
    /// Mostra o estado rico da fila, sem drenar.
    #[arg(long)]
    pub status: bool,
    /// Com `--digest`: apaga `.idx/` (derivado) e redigeri tudo do zero (último recurso).
    #[arg(long, requires = "digest")]
    pub force: bool,
}
