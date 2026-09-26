//! Subcomandos de `kd maintenance`, `kd config` e `kd self`.

use clap::{Args, Subcommand};

/// Subcomandos de manutenção.
#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum MaintenanceCommand {
    /// Propõe consolidação (merge/supersede), nunca em silêncio.
    Compact {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
        /// Avalia o portão de evidência sobre cada proposta (read-only) — D156.
        #[arg(long)]
        verify: bool,
        /// Filtros de corpus (D144).
        #[command(flatten)]
        corpus: CorpusArgs,
    },
    /// Sugere notas/links/merges a partir de eventos e âncoras.
    Learn {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
        /// Avalia o portão de evidência sobre cada proposta (read-only) — D156.
        #[arg(long)]
        verify: bool,
        /// Filtros de corpus (D144).
        #[command(flatten)]
        corpus: CorpusArgs,
    },
    /// Propõe aposentadoria (`forget`) por shelf-life/decay — nunca age (D112).
    Prune {
        /// Container/domínio de escopo.
        #[arg(long, value_name = "CONTAINER")]
        scope: Option<String>,
        /// Proposta é sempre read-only (D47); a flag existe por paridade.
        #[arg(long)]
        dry_run: bool,
        /// Filtros de corpus (D144).
        #[command(flatten)]
        corpus: CorpusArgs,
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

/// Filtros de corpus compartilhados por `learn`/`compact`/`prune` (D143/D144).
#[derive(Debug, Args)]
pub struct CorpusArgs {
    /// Filtro por tipo (repetível).
    #[arg(long = "type", value_name = "TIPO")]
    pub types: Vec<String>,
    /// Filtro por classificação (repetível).
    #[arg(long = "class", value_name = "CLASSE")]
    pub classes: Vec<String>,
    /// Filtro por tag (repetível; basta uma).
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Filtro por âncora (repetível; aceita lista com vírgula).
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    pub anchor: Vec<String>,
    /// Ponto de partida: vizinhança de uma nota pelo grafo.
    #[arg(long, value_name = "ID")]
    pub around: Option<String>,
    /// Profundidade da vizinhança de `--around`.
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub depth: u8,
    /// Varredura explícita do projeto inteiro (sem filtro).
    #[arg(long)]
    pub universe: bool,
}

/// Subcomandos de `kd config`.
#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ConfigCommand {
    /// Lê uma chave.
    Get {
        /// Chave.
        #[arg(long = "key", value_name = "CHAVE")]
        key: String,
        /// Usa o config global.
        #[arg(long)]
        global: bool,
    },
    /// Define uma chave (valida contra o schema).
    Set {
        /// Chave.
        #[arg(long = "key", value_name = "CHAVE")]
        key: String,
        /// Valor.
        #[arg(long = "value", value_name = "VALOR")]
        value: String,
        /// Grava no config global.
        #[arg(long)]
        global: bool,
    },
    /// Remove uma chave.
    Unset {
        /// Chave.
        #[arg(long = "key", value_name = "CHAVE")]
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
#[command(arg_required_else_help = true)]
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
