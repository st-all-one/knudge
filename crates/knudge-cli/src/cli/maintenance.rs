//! Subcomandos de `kd maintenance`, `kd config` e `kd self`.

use clap::Subcommand;

/// Subcomandos de manutenção.
#[derive(Debug, Subcommand)]
pub enum MaintenanceCommand {
    /// Relatório de saúde (e reparo do reversível com `--fix`).
    Doctor {
        /// Corrige o que for reversível.
        #[arg(long)]
        fix: bool,
        /// Foco em integridade de grafo/arestas.
        #[arg(long)]
        audit: bool,
    },
    /// Propõe consolidação (merge/supersede), nunca em silêncio.
    Compact {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
    },
    /// Métricas de retrieval (Recall@k, nDCG@k, MRR).
    Eval {
        /// Compara dois modelos: `--ab <A> <B>`.
        #[arg(long = "ab", num_args = 2, value_names = ["A", "B"])]
        ab: Vec<String>,
    },
    /// Fila de embeddings.
    Index {
        /// Drena a fila agora.
        #[arg(long)]
        drain: bool,
        /// Mostra o estado da fila.
        #[arg(long)]
        status: bool,
    },
    /// Sugere notas/links/merges a partir de eventos e âncoras.
    Learn {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
    },
}

/// Subcomandos de `kd config`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Lê uma chave.
    Get {
        /// Chave.
        key: String,
        /// Usa o config global.
        #[arg(long)]
        global: bool,
    },
    /// Define uma chave (valida contra o schema).
    Set {
        /// Chave.
        key: String,
        /// Valor.
        value: String,
        /// Grava no config global.
        #[arg(long)]
        global: bool,
    },
    /// Remove uma chave.
    Unset {
        /// Chave.
        key: String,
        /// Usa o config global.
        #[arg(long)]
        global: bool,
    },
    /// Lista as chaves.
    List {
        /// Usa o config global.
        #[arg(long)]
        global: bool,
    },
}

/// Subcomandos de `kd self`.
#[derive(Debug, Subcommand)]
pub enum SelfCommand {
    /// Instala a recipe de um cliente (`claude`/`cursor`/`codex`/`pi`).
    Setup {
        /// Cliente alvo.
        #[arg(value_name = "CLIENTE")]
        client: String,
    },
    /// Gera shell completions.
    Completions {
        /// Shell alvo.
        #[arg(value_name = "SHELL")]
        shell: String,
    },
    /// Atualiza o binário.
    Upgrade,
    /// Mostra a versão.
    Version,
}
