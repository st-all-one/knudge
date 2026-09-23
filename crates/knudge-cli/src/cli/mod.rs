//! Argumentos e subcomandos do `kd` (superfície v2, ver `plan/implementation/16_cli_surface.md`).

mod knowledge;
mod maintenance;
mod task;

pub use knowledge::KnowledgeCommand;
pub use maintenance::{ConfigCommand, MaintenanceCommand, SelfCommand, WatchServiceArgs};
pub use task::{TaskCommand, TaskListArgs, TaskNewArgs, TaskPlanArgs, TaskSort};

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
    disable_help_subcommand = true
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
    /// Subcomando; sem argumentos equivale a `prime`.
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
    Knowledge {
        /// Subcomando de conhecimento.
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    /// Manutenção (doctor, compact, eval, index, learn, watch-service).
    Maintenance {
        /// Subcomando de manutenção.
        #[command(subcommand)]
        command: MaintenanceCommand,
    },
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
            Self::Knowledge { .. } => "knowledge",
            Self::Maintenance { .. } => "maintenance",
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

/// Argumentos de `kd rewind`.
#[derive(Debug, Args)]
pub struct RewindArgs {
    /// Container/domínio de escopo.
    #[arg(long, value_name = "CONTAINER")]
    pub scope: Option<String>,
    /// Working set por arquivos.
    #[arg(long, value_name = "PATH")]
    pub files: Vec<String>,
    /// Orçamento de tokens.
    #[arg(long, value_name = "N")]
    pub budget: Option<usize>,
    /// Início do intervalo.
    #[arg(long, value_name = "TS")]
    pub since: Option<String>,
    /// Fim do intervalo.
    #[arg(long, value_name = "TS")]
    pub until: Option<String>,
    /// Retoma um contexto por id (handoff 1:1).
    #[arg(long, value_name = "CONTEXT_ID")]
    pub resume: Option<String>,
}

/// Argumentos de `kd ask`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct AskArgs {
    /// Consulta textual (recall completo).
    #[arg(value_name = "QUERY")]
    pub query: Vec<String>,
    /// Recupera os corpos dos ids.
    #[arg(long = "id", value_name = "ID")]
    pub ids: Vec<String>,
    /// Expande o grafo a partir do id.
    #[arg(long, value_name = "ID")]
    pub around: Option<String>,
    /// Aresta do expand.
    #[arg(long, value_name = "ARESTA")]
    pub via: Option<String>,
    /// Profundidade do expand.
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub depth: u8,
    /// Saída mínima (`id|statement`).
    #[arg(long)]
    pub brief: bool,
    /// Lista o vocabulário de tags (`tag|count`, `count` desc).
    #[arg(long = "tags")]
    pub tag_vocab: bool,
    /// Ranqueia por confiança derivada, sem query textual (D107).
    #[arg(long)]
    pub rank: bool,
    /// Inclui o corpo dos hits.
    #[arg(long)]
    pub with_body: bool,
    /// Filtro por tipo.
    #[arg(long = "type", value_name = "TIPO")]
    pub types: Vec<String>,
    /// Filtro por classificação.
    #[arg(long = "class", value_name = "CLASSE")]
    pub classes: Vec<String>,
    /// Filtro por tag.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Filtro por status.
    #[arg(long, value_name = "STATUS")]
    pub status: Option<String>,
    /// Filtro por container.
    #[arg(long, value_name = "ID")]
    pub container: Option<String>,
    /// Filtro por âncora (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    pub anchor: Vec<String>,
    /// Início do intervalo.
    #[arg(long, value_name = "TS")]
    pub since: Option<String>,
    /// Fim do intervalo.
    #[arg(long, value_name = "TS")]
    pub until: Option<String>,
    /// Limite de resultados.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd write`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct WriteArgs {
    /// Afirmação da nota.
    #[arg(value_name = "STATEMENT")]
    pub statement: Vec<String>,
    /// Tipo da nota (default: `fact`).
    #[arg(long = "type", value_name = "TIPO")]
    pub note_type: Option<String>,
    /// Corpo da nota (`-` lê stdin).
    #[arg(long, value_name = "TXT")]
    pub body: Option<String>,
    /// Tags.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Âncoras (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(
        long = "anchor",
        visible_alias = "anchors",
        value_name = "PATH",
        value_delimiter = ','
    )]
    pub anchors: Vec<String>,
    /// Checks (validators).
    #[arg(long, value_name = "NOME")]
    pub checks: Vec<String>,
    /// Proveniência.
    #[arg(long, value_name = "FONTE")]
    pub source: Option<String>,
    /// Classificação.
    #[arg(long = "class", value_name = "CLASSE")]
    pub class: Option<String>,
    /// Status inicial.
    #[arg(long, value_name = "STATUS")]
    pub status: Option<String>,
    /// Expiração.
    #[arg(long, value_name = "TS")]
    pub expires_at: Option<String>,
    /// Confiança declarada.
    #[arg(long, value_name = "F")]
    pub confidence: Option<f64>,
    /// Aresta explícita `ARESTA:ID` a partir da nota.
    #[arg(long, value_name = "ARESTA:ID")]
    pub edge: Vec<String>,
    /// Atualiza a nota existente.
    #[arg(long, value_name = "ID")]
    pub update: Option<String>,
    /// Cria aresta `FROM:ARESTA:TO`.
    #[arg(long, value_name = "FROM:ARESTA:TO")]
    pub link: Option<String>,
    /// Simula sem gravar.
    #[arg(long)]
    pub dry_run: bool,
    /// Aplica um lote de rascunhos JSONL (`-` lê stdin).
    #[arg(long, value_name = "FONTE")]
    pub batch: Option<String>,
    /// Anexa um resultado (`success|partial|failure|abandoned`) a uma nota existente.
    #[arg(long, value_name = "OUTCOME")]
    pub outcome: Option<String>,
    /// Texto do resultado (usado com `--outcome`).
    #[arg(long, value_name = "TXT")]
    pub note: Option<String>,
}

/// Argumentos de `kd forget`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct ForgetArgs {
    /// Id da nota.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Restaura em vez de esquecer.
    #[arg(long)]
    pub restore: bool,
    /// Remove fisicamente após a retenção.
    #[arg(long)]
    pub purge: bool,
}

/// Argumentos de `kd sync`.
#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Mensagem de commit.
    #[arg(long)]
    pub message: Option<String>,
}
