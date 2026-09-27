//! Argumentos e subcomandos do `kd` (superfície v2, ver `plan/implementation/16_cli_surface.md`).

mod ask;
mod health;
mod help;
mod knowledge;
mod maintenance;
mod rewind;
mod task;
mod write;

pub use ask::AskArgs;
pub use health::{DoctorArgs, DrainArgs, DrainCommand, WatchServiceArgs};
pub use help::{render_help, subcommand_help};
pub use knowledge::{
    MapArgs, PromoteCommand, PromoteEditArgs, PromoteRecommendArgs, PromoteTargetArgs, RankArgs,
    SuggestArgs, TagsArgs,
};
pub use maintenance::{ConfigCommand, CorpusArgs, MaintenanceCommand, SelfCommand, UpgradeArgs};
pub use rewind::RewindArgs;
pub use task::{TaskCommand, TaskFlowArgs, TaskListArgs, TaskNewArgs, TaskPlanArgs, TaskSort};
pub use write::{ForgetArgs, SyncArgs, WriteArgs};

use clap::{Args, Parser, Subcommand};

/// Interface de linha de comando do `kd`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags de CLI; bools são o tipo natural"
)]
#[derive(Debug, Parser)]
#[command(
    name = "kd",
    version,
    about = "knudge — memória otimizada para LLM",
    after_help = help::AFTER_HELP
)]
pub struct Cli {
    /// Emite o resultado no envelope JSON (contrato de máquina).
    #[arg(long, global = true)]
    pub json: bool,
    /// Nível de log em stderr (`error|warn|info|debug|trace|off`).
    #[arg(long, global = true, value_name = "NÍVEL", default_value = "warn")]
    pub log_level: String,
    /// Silencia o stderr.
    #[arg(long, global = true)]
    pub quiet: bool,
    /// Subcomando; sem verbo o `kd` mostra o help (D171).
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Verbos de topo.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Funda `.knudge/` no projeto e emite o prompt inicial.
    Init(InitArgs),
    /// Protocolo de uso estático (byte-idêntico por versão).
    Prime(PrimeArgs),
    /// Estado/handoff ponto-no-tempo.
    Rewind(RewindArgs),
    /// Toda pesquisa (recall, get e expand).
    Ask(AskArgs),
    /// Toda escrita (create, update e arestas).
    Write(WriteArgs),
    /// Gestão de tarefas (`plan`/`epic`/`issue`/`task`).
    Task {
        /// Subcomando de tarefa.
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Mapa de conhecimento (clusters estruturais e semânticos).
    Map(MapArgs),
    /// Manutenção (compact, learn, prune; só propõem, nunca agem).
    Maintenance {
        /// Subcomando de manutenção.
        #[command(subcommand)]
        command: MaintenanceCommand,
    },
    /// Saúde do corpus: 13 checks + auditoria, com reparo reversível (D163).
    Doctor(DoctorArgs),
    /// Fila de embeddings (`--status`/`--digest`) e worker (`service`; D170/D186).
    Drain(DrainArgs),
    /// Configuração do projeto.
    Config {
        /// Subcomando de configuração.
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Soft-delete de notas.
    Forget(ForgetArgs),
    /// Commit de `notas/` + `eventos/`.
    Sync(SyncArgs),
    /// Instalação e utilitários do binário.
    #[command(name = "self")]
    SelfCmd {
        /// Subcomando de instalação.
        #[command(subcommand)]
        command: SelfCommand,
    },
}

impl Command {
    /// Nome canônico usado no envelope.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Init(_) => "init",
            Self::Prime(_) => "prime",
            Self::Rewind(_) => "rewind",
            Self::Ask(_) => "ask",
            Self::Write(_) => "write",
            Self::Task { .. } => "task",
            Self::Map(_) => "map",
            Self::Maintenance { .. } => "maintenance",
            Self::Doctor(_) => "doctor",
            Self::Drain(_) => "drain",
            Self::Config { .. } => "config",
            Self::Forget(_) => "forget",
            Self::Sync(_) => "sync",
            Self::SelfCmd { .. } => "self",
        }
    }
}

/// Argumentos de `kd init`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Sobrescreve a configuração existente.
    #[arg(long)]
    pub force: bool,
    /// Não emite o prompt inicial de fundação.
    #[arg(long)]
    pub no_prompt: bool,
}

/// Argumentos de `kd prime`.
#[derive(Debug, Args)]
pub struct PrimeArgs {
    /// Inclui a gramática TOON e o schema completo.
    #[arg(long)]
    pub long: bool,
}
