//! Subcomandos de `kd task` (hierarquia fechada `epic ⊃ { issue ⊃ task | task }`, D93/D134).

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
        #[arg(
            long = "id",
            value_name = "ID",
            required = true,
            num_args = 1..,
            value_delimiter = ','
        )]
        ids: Vec<String>,
        /// Inclui o histórico de supersessão.
        #[arg(long)]
        history: bool,
    },
    /// Atualiza uma tarefa.
    Update {
        /// Id.
        #[arg(long = "id", value_name = "ID")]
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
        /// Novas âncoras (repetível; aceita lista com vírgula). Substitui as existentes.
        #[arg(
            long = "anchor",
            visible_alias = "anchors",
            value_name = "PATH",
            value_delimiter = ',',
            conflicts_with = "clear_anchors"
        )]
        anchor: Vec<String>,
        /// Limpa todas as âncoras.
        #[arg(long)]
        clear_anchors: bool,
    },
    /// Fecha uma tarefa (roda validators e grava evidência).
    Close {
        /// Id.
        #[arg(long = "id", value_name = "ID")]
        id: String,
        /// Resultado: `success|partial|failure|abandoned`.
        #[arg(long, value_name = "OUTCOME")]
        outcome: Option<String>,
        /// Motivo do fechamento (entra em `outcomes[].notes`) — T6/D104.
        #[arg(long, value_name = "TXT")]
        note: Option<String>,
    },
    /// Renderiza a árvore de um programa externo (`plan/*.md`) ou de um escopo — D119/D116.
    Graph {
        /// Arquivo do programa.
        #[arg(long, value_name = "PATH")]
        program: Option<String>,
        /// Id do escopo-raiz (épico).
        #[arg(long, value_name = "ID")]
        root: Option<String>,
    },
    /// Plano: `--prompt` (read-only) ou `--submit` (`--step`/`--from`) — D105/D138.
    Plan(TaskPlanArgs),
}

/// Argumentos de `kd task plan`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "`--prompt` e `--submit` são modos mutuamente exclusivos do plano"
)]
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
    /// Filtro por pai.
    #[arg(long, value_name = "ID")]
    pub parent: Option<String>,
    /// Só tarefas prontas (dependências resolvidas).
    #[arg(long)]
    pub ready: bool,
    /// Só tarefas bloqueadas.
    #[arg(long)]
    pub blocked: bool,
    /// Com `--blocked`, acrescenta o motivo (`blocked_by=`/`cycle`);
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
    /// Renderiza cada item como bloco completo (multilinha; não é pipe-safe) — D137.
    #[arg(long)]
    pub full_content: bool,
    /// Panorama geral explícito: lista tudo sem exigir filtro (D144).
    #[arg(long)]
    pub universe: bool,
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
    /// Corpo (Markdown); `-` lê stdin; vazio + pipe também lê stdin.
    #[arg(value_name = "BODY")]
    pub body: Vec<String>,
    /// Afirmação (chave TOON `statement`).
    #[arg(long, value_name = "TXT")]
    pub summary: Option<String>,
    /// Escopo fechado (obrigatório fora de `--params`/`--batch`).
    #[arg(long, value_name = "ESCOPO")]
    pub scope: Option<String>,
    /// Espécie (`type`) do item: `task|error|question|risk|decision` (D113).
    #[arg(long, value_name = "ESPECIE")]
    pub kind: Option<String>,
    /// Pai na hierarquia.
    #[arg(long, value_name = "ID")]
    pub parent: Option<String>,
    /// Checks (validators).
    #[arg(long, value_name = "NOME")]
    pub checks: Vec<String>,
    /// Âncoras (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(
        long = "anchor",
        visible_alias = "anchors",
        value_name = "PATH",
        value_delimiter = ','
    )]
    pub anchors: Vec<String>,
    /// Tags declaradas (repetível).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,
    /// Objeto JSON de uma tarefa (`-` lê stdin) — D141/D147.
    #[arg(long, value_name = "JSON")]
    pub params: Option<String>,
    /// Lote JSONL de operações (`-` lê stdin) — D141.
    #[arg(long, value_name = "FONTE")]
    pub batch: Option<String>,
    /// Só avalia o lote, sem gravar — D141.
    #[arg(long)]
    pub dry_run: bool,
}
