//! Subcomandos de `kd maintenance`, `kd config` e `kd self`.
//!
//! `kd maintenance` só revisa (`compact`/`learn`/`prune`); o worker de auto-drain vive em
//! `kd drain service` (D186).

use clap::{Args, Subcommand};

use super::knowledge::PromoteCommand;

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
    /// Promove conhecimento a regras governadas no `AGENTS.md` (D157).
    Promote {
        /// Subcomando de promoção.
        #[command(subcommand)]
        command: PromoteCommand,
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
    /// Atualiza o binário pelo instalador oficial (release + checksum; D187).
    Upgrade(UpgradeArgs),
    /// Mostra a versão.
    Version,
}

/// Argumentos de `kd self upgrade` (D187).
#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Só mostra o plano (não baixa nem executa).
    #[arg(long)]
    pub dry_run: bool,
    /// Versão/tag alvo (default: `latest`).
    #[arg(long, value_name = "TAG")]
    pub version: Option<String>,
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
