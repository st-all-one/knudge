//! Subcomandos de `kd task` (hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task`, D93).

use clap::{Args, Subcommand};

/// Subcomandos de tarefa.
#[allow(
    clippy::large_enum_variant,
    reason = "variantes são argumentos de CLI derivados pelo clap, sem Box"
)]
#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Cria uma tarefa.
    New(TaskNewArgs),
    /// Lista tarefas.
    List(TaskListArgs),
    /// Mostra uma tarefa.
    Show {
        /// Id.
        #[arg(value_name = "ID")]
        id: String,
        /// Inclui o histórico de supersessão.
        #[arg(long)]
        history: bool,
    },
    /// Atualiza uma tarefa.
    Update {
        /// Id.
        #[arg(value_name = "ID")]
        id: String,
        /// Nova afirmação.
        #[arg(long, value_name = "TXT")]
        statement: Option<String>,
        /// Novo status.
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        /// Novo pai.
        #[arg(long, value_name = "ID")]
        parent: Option<String>,
        /// Novos checks.
        #[arg(long, value_name = "NOME")]
        checks: Vec<String>,
    },
    /// Fecha uma tarefa (roda validators e grava evidência).
    Close {
        /// Id.
        #[arg(value_name = "ID")]
        id: String,
        /// Resultado: `success|partial|failure|abandoned`.
        #[arg(long, value_name = "OUTCOME")]
        outcome: Option<String>,
    },
    /// Reivindica (`--by`) ou libera (`--release`) um item de trabalho (D114).
    Claim {
        /// Id.
        #[arg(value_name = "ID")]
        id: String,
        /// Agente que assume a tarefa.
        #[arg(long, value_name = "AGENTE")]
        by: Option<String>,
        /// Libera a tarefa (sem dono).
        #[arg(long)]
        release: bool,
    },
    /// Renderiza a árvore de um programa externo (`plan/*.md`) — D119.
    Graph {
        /// Arquivo do programa.
        #[arg(long, value_name = "PATH")]
        program: String,
    },
    /// Ciclo de vida do plano (`submit`/`adopt`/`reorder`/`release`/`review`).
    Plan {
        /// Id do plano.
        #[arg(value_name = "ID")]
        id: String,
        /// Passos.
        #[arg(long = "step", value_name = "TXT")]
        steps: Vec<String>,
        /// Submete o plano.
        #[arg(long)]
        submit: bool,
        /// Adota o plano.
        #[arg(long)]
        adopt: bool,
        /// Reordena para a posição 1-based.
        #[arg(long, value_name = "N")]
        reorder: Option<u32>,
        /// Libera o plano.
        #[arg(long)]
        release: bool,
        /// Marca para revisão.
        #[arg(long)]
        review: bool,
    },
}

/// Argumentos de `kd task list`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct TaskListArgs {
    /// Filtro por escopo.
    #[arg(long, value_name = "ESCOPO")]
    pub scope: Option<String>,
    /// Filtro por status.
    #[arg(long, value_name = "STATUS")]
    pub status: Option<String>,
    /// Filtro por espécie (`type`) — D113.
    #[arg(long, value_name = "ESPECIE")]
    pub kind: Option<String>,
    /// Filtro por dono derivado de `claim` (D114).
    #[arg(long, value_name = "AGENTE")]
    pub owner: Option<String>,
    /// Só o que o ator atual reivindicou (D114).
    #[arg(long)]
    pub mine: bool,
    /// Filtro por pai.
    #[arg(long, value_name = "ID")]
    pub parent: Option<String>,
    /// Só tarefas prontas (dependências resolvidas e `not_before` vencido).
    #[arg(long)]
    pub ready: bool,
    /// Só tarefas bloqueadas.
    #[arg(long)]
    pub blocked: bool,
    /// Com `--blocked`, acrescenta o motivo (`blocked_by=`/`not_before=`/`cycle`).
    #[arg(long)]
    pub explain: bool,
}

/// Argumentos de `kd task new`.
#[derive(Debug, Args)]
pub struct TaskNewArgs {
    /// Afirmação.
    #[arg(value_name = "STATEMENT")]
    pub statement: Vec<String>,
    /// Escopo fechado.
    #[arg(long, value_name = "ESCOPO")]
    pub scope: String,
    /// Espécie (`type`) do item: `task|error|question|risk|decision` (D113).
    #[arg(long, value_name = "ESPECIE")]
    pub kind: Option<String>,
    /// Pai na hierarquia.
    #[arg(long, value_name = "ID")]
    pub parent: Option<String>,
    /// Corpo (`-` lê stdin).
    #[arg(long, value_name = "TXT")]
    pub body: Option<String>,
    /// Checks (validators).
    #[arg(long, value_name = "NOME")]
    pub checks: Vec<String>,
    /// Âncoras.
    #[arg(long, value_name = "PATH")]
    pub anchors: Vec<String>,
    /// Proveniência (`source`) — ex.: o arquivo do programa.
    #[arg(long, value_name = "FONTE")]
    pub source: Option<String>,
    /// Dependências (`depends_on`).
    #[arg(long = "depends-on", value_name = "ID")]
    pub depends_on: Vec<String>,
    /// Agendamento (`not_before`).
    #[arg(long, value_name = "TS")]
    pub not_before: Option<String>,
    /// Expiração.
    #[arg(long, value_name = "TS")]
    pub expires_at: Option<String>,
}
