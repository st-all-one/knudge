//! Subcomandos de `kd task` (hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task`, D93).

use clap::Subcommand;

/// Subcomandos de tarefa.
#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Cria uma tarefa.
    New {
        /// Afirmação.
        #[arg(value_name = "STATEMENT")]
        statement: Vec<String>,
        /// Escopo fechado.
        #[arg(long, value_name = "ESCOPO")]
        scope: String,
        /// Pai na hierarquia.
        #[arg(long, value_name = "ID")]
        parent: Option<String>,
        /// Corpo.
        #[arg(long, value_name = "TXT")]
        body: Option<String>,
        /// Checks (validators).
        #[arg(long, value_name = "NOME")]
        checks: Vec<String>,
        /// Âncoras.
        #[arg(long, value_name = "PATH")]
        anchors: Vec<String>,
        /// Dependências (`depends_on`).
        #[arg(long = "depends-on", value_name = "ID")]
        depends_on: Vec<String>,
        /// Agendamento (`not_before`).
        #[arg(long, value_name = "TS")]
        not_before: Option<String>,
        /// Expiração.
        #[arg(long, value_name = "TS")]
        expires_at: Option<String>,
    },
    /// Lista tarefas.
    List {
        /// Filtro por escopo.
        #[arg(long, value_name = "ESCOPO")]
        scope: Option<String>,
        /// Filtro por status.
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        /// Filtro por pai.
        #[arg(long, value_name = "ID")]
        parent: Option<String>,
    },
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
