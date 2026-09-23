//! Subcomandos de `kd task` (hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task`, D93).

use clap::{Args, Subcommand, ValueEnum};

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
    /// Mostra uma ou mais tarefas.
    Show {
        /// Ids.
        #[arg(value_name = "ID", required = true, num_args = 1..)]
        ids: Vec<String>,
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
        /// Motivo do fechamento (entra em `outcomes[].notes`) — T6/D104.
        #[arg(long, value_name = "TXT")]
        note: Option<String>,
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
    /// Renderiza a árvore de um programa externo (`plan/*.md`) ou de um container — D119/D116.
    Graph {
        /// Arquivo do programa.
        #[arg(long, value_name = "PATH")]
        program: Option<String>,
        /// Id do container-raiz.
        #[arg(long, value_name = "ID")]
        root: Option<String>,
    },
    /// Ciclo de vida do plano (`prompt`/`submit`/`adopt`/`reorder`/`release`/`review`).
    Plan(TaskPlanArgs),
}

/// Argumentos de `kd task plan`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct TaskPlanArgs {
    /// Id do plano.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Passos.
    #[arg(long = "step", value_name = "TXT")]
    pub steps: Vec<String>,
    /// Submete o plano.
    #[arg(long)]
    pub submit: bool,
    /// Imprime o prompt do plano (read-only).
    #[arg(long)]
    pub prompt: bool,
    /// Template do plano (`feature`/`bug`/`refactor`).
    #[arg(long, value_name = "NOME")]
    pub template: Option<String>,
    /// Plano preenchido (TOON): `-` lê stdin, senão um arquivo.
    #[arg(long, value_name = "FONTE")]
    pub from: Option<String>,
    /// Adota o plano.
    #[arg(long)]
    pub adopt: bool,
    /// Reordena para a posição 1-based.
    #[arg(long, value_name = "N")]
    pub reorder: Option<u32>,
    /// Libera o plano.
    #[arg(long)]
    pub release: bool,
    /// Marca para revisão.
    #[arg(long)]
    pub review: bool,
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
    /// Com `--blocked`, acrescenta o motivo (`blocked_by=`/`not_before=`/`cycle`);
    /// com `--sort impact`, acrescenta `unblocks=N` (D109).
    #[arg(long)]
    pub explain: bool,
    /// Ordenação derivada do grafo (D109).
    #[arg(long, value_enum, value_name = "CAMPO")]
    pub sort: Option<TaskSort>,
    /// Filtro por tag (repetível; basta uma).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,
    /// Filtro por âncora (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    pub anchor: Vec<String>,
    /// Só o que foi criado a partir do instante (`TS` RFC3339/epoch).
    #[arg(long, value_name = "TS")]
    pub since: Option<String>,
}

/// Campo de ordenação de `kd task list` (D109).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TaskSort {
    /// Impacto de desbloqueio: tarefas abertas que dependem do nó (transitivo).
    Impact,
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
    /// Tags declaradas (repetível).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,
    /// Proveniência (`source`) — ex.: o arquivo do programa.
    #[arg(long, value_name = "FONTE")]
    pub source: Option<String>,
    /// Agendamento (`not_before`).
    #[arg(long, value_name = "TS")]
    pub not_before: Option<String>,
    /// Expiração.
    #[arg(long, value_name = "TS")]
    pub expires_at: Option<String>,
}
