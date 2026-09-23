//! Subcomandos de `kd maintenance`, `kd config` e `kd self`.

use clap::{Args, Subcommand};

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
        /// Drena um lote da fila agora (repita para drenar mais).
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
    /// Propõe aposentadoria (`forget`) por shelf-life/decay — nunca age (D112).
    Prune {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
        /// Proposta é sempre read-only (D47); a flag existe por paridade.
        #[arg(long)]
        dry_run: bool,
    },
    /// Gerencia o worker de auto-drain ocioso: `--install`/`--subscribe`/`--unsubscribe`/`--status`/`--uninstall`.
    WatchService(WatchServiceArgs),
}

/// Argumentos de `kd maintenance watch-service`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct WatchServiceArgs {
    /// Instala o agendador (systemd/launchd), o servidor de embeddings persistente e cadastra
    /// o projeto atual. Baixa llama.cpp + GGUF se ausentes (use `--no-deps` para pular).
    #[arg(long, group = "action")]
    pub install: bool,
    /// Cadastra o projeto atual (multi-projeto; exige o sistema instalado).
    #[arg(long, group = "action")]
    pub subscribe: bool,
    /// Descadastra o projeto atual (mantém o sistema instalado).
    #[arg(long, group = "action")]
    pub unsubscribe: bool,
    /// Mostra a saúde atual (agendador, servidor, fila por projeto). É o default.
    #[arg(long, group = "action")]
    pub status: bool,
    /// Remove o sistema (agendador + servidor + config + binário).
    #[arg(long, group = "action")]
    pub uninstall: bool,
    /// Não pergunta: assume que sim.
    #[arg(long, short = 'y')]
    pub yes: bool,
    /// Só mostra o plano (não baixa nem executa).
    #[arg(long)]
    pub dry_run: bool,
    /// Período do drain (ex.: `1h`, `15min`).
    #[arg(long, value_name = "DUR", default_value = "1h")]
    pub every: String,
    /// Porta do servidor de embeddings local.
    #[arg(long, value_name = "N", default_value_t = 8999)]
    pub port: u16,
    /// Caminho do modelo GGUF (default: ao lado do config.toml global).
    #[arg(long, value_name = "PATH")]
    pub model: Option<String>,
    /// Não instala dependências (llama.cpp + GGUF) no `--install`; falha se faltarem.
    #[arg(long)]
    pub no_deps: bool,
    /// Usa um script local em vez do embutido (offline/testes).
    #[arg(long, value_name = "PATH")]
    pub script: Option<String>,
    /// Baixa o script de uma URL (HTTPS) em vez de usar o embutido.
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,
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
