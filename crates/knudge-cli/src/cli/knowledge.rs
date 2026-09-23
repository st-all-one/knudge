//! Subcomandos de `kd knowledge` — o mapa de conhecimento (D128).

use clap::Subcommand;

/// Subcomandos de conhecimento.
#[derive(Debug, Subcommand)]
pub enum KnowledgeCommand {
    /// Mapa de conhecimento: clusters por eixo estrutural (fase 1) e, com `--semantic`,
    /// semântico (fase 2).
    Map {
        /// Restringe a um eixo: `anchor`/`type`/`classification`/`container`.
        #[arg(long, value_name = "EIXO")]
        axis: Option<String>,
        /// Restringe aos membros de um container.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
        /// Roda a fase 2 semântica dentro dos clusters (requer embeddings).
        #[arg(long)]
        semantic: bool,
        /// Inclui os membros de cada cluster.
        #[arg(long)]
        members: bool,
    },
}
