//! Argumentos dos verbos de saúde/fila do `kd`: `doctor` (D163) e `drain` (D170/D186).

use clap::{Args, Subcommand};

/// Argumentos de `kd doctor`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Corrige o que for reversível (idempotente).
    #[arg(long)]
    pub fix: bool,
    /// Detalha cada achado (`esperado` × `encontrado` × `ação`).
    #[arg(long)]
    pub explain: bool,
}

/// Argumentos de `kd drain`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "ações mutuamente exclusivas da CLI (D170)"
)]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true, args_conflicts_with_subcommands = true)]
pub struct DrainArgs {
    /// Digere a fila em lotes até esvaziar/estagnar (log mínimo).
    #[arg(long, conflicts_with = "status")]
    pub digest: bool,
    /// Mostra o estado rico da fila, sem drenar.
    #[arg(long)]
    pub status: bool,
    /// Com `--digest`: apaga `.idx/` (derivado) e redigeri tudo do zero (último recurso).
    #[arg(long, requires = "digest")]
    pub force: bool,
    /// Subcomando `service`: worker de auto-drain (agendador + servidor de embeddings).
    #[command(subcommand)]
    pub command: Option<DrainCommand>,
}

/// Subcomandos de `kd drain`.
#[derive(Debug, Subcommand)]
pub enum DrainCommand {
    /// Gerencia o worker de auto-drain ocioso (D186).
    Service(WatchServiceArgs),
}

/// Argumentos de `kd drain service`.
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
    /// SHA-256 (hex) do script remoto (`--url`); obrigatório para baixar (D184).
    #[arg(long, value_name = "HEX")]
    pub sha256: Option<String>,
}
