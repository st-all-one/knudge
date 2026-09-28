//! Modos de `kd ask` (rank/tags/suggest), `kd map` e `kd config promote` — mapa/ranking/vocabulário de conhecimento (D128/D146).

use clap::{Args, Subcommand};

/// Subcomandos de `kd config promote` (D157).
#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum PromoteCommand {
    /// Recomenda candidatas a regra (read-only).
    Recommend(PromoteRecommendArgs),
    /// Promove uma nota para o bloco governado (respeita o teto).
    Approve(PromoteTargetArgs),
    /// Edita a linha promovida de uma nota (a proveniência continua).
    Edit(PromoteEditArgs),
    /// Remove uma nota do bloco governado (a nota de origem permanece).
    Remove(PromoteTargetArgs),
    /// Lista as notas promovidas.
    List,
}

/// Argumentos de `kd config promote recommend`.
#[derive(Debug, Args)]
pub struct PromoteRecommendArgs {
    /// Varredura do projeto inteiro (sem filtro de trabalho).
    #[arg(long)]
    pub universe: bool,
    /// Limite de candidatas.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd config promote approve|remove`.
#[derive(Debug, Args)]
pub struct PromoteTargetArgs {
    /// Id da nota.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Varredura do projeto inteiro (sem filtro de trabalho).
    #[arg(long)]
    pub universe: bool,
}

/// Argumentos de `kd config promote edit`.
#[derive(Debug, Args)]
pub struct PromoteEditArgs {
    /// Id da nota.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Texto da regra.
    #[arg(long, value_name = "TXT")]
    pub summary: String,
    /// Varredura do projeto inteiro (sem filtro de trabalho).
    #[arg(long)]
    pub universe: bool,
}

/// Argumentos de `kd map`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct MapArgs {
    /// Restringe a um eixo: `anchor`/`type`/`classification`/`scope`.
    #[arg(long, value_name = "EIXO")]
    pub axis: Option<String>,
    /// Restringe aos membros de um escopo (épico).
    #[arg(long, value_name = "ESCOPO")]
    pub scope: Option<String>,
    /// Roda a fase 2 semântica dentro dos clusters (requer embeddings).
    #[arg(long)]
    pub semantic: bool,
    /// Detecta **comunidades** (`GraphRAG`) sobre arestas + âncoras, com resumo local (D193).
    #[arg(long)]
    pub communities: bool,
    /// Inclui os membros de cada cluster.
    #[arg(long)]
    pub members: bool,
    /// Materializa o mapa: `notas/MAP.md` + uma nota-hub (`references`) por cluster (D150).
    #[arg(long)]
    pub write: bool,
    /// Filtro por tipo (repetível; aceita lista com vírgula: `--type a,b`).
    #[arg(long = "type", value_name = "TIPO", value_delimiter = ',')]
    pub types: Vec<String>,
    /// Filtro por classificação (repetível; aceita lista com vírgula).
    #[arg(long = "class", value_name = "CLASSE", value_delimiter = ',')]
    pub classes: Vec<String>,
    /// Filtro por tag (repetível; aceita lista com vírgula).
    #[arg(long = "tag", value_name = "TAG", value_delimiter = ',')]
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

/// Argumentos de `kd ask --rank`.
#[derive(Debug, Args)]
pub struct RankArgs {
    /// Filtro por tipo (repetível; aceita lista com vírgula: `--type a,b`).
    #[arg(long = "type", value_name = "TIPO", value_delimiter = ',')]
    pub types: Vec<String>,
    /// Filtro por classificação (repetível; aceita lista com vírgula).
    #[arg(long = "class", value_name = "CLASSE", value_delimiter = ',')]
    pub classes: Vec<String>,
    /// Filtro por tag (repetível; aceita lista com vírgula).
    #[arg(long = "tag", value_name = "TAG", value_delimiter = ',')]
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
    /// Varredura explícita do projeto inteiro.
    #[arg(long)]
    pub universe: bool,
    /// Limite de resultados.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd ask --tags`.
#[derive(Debug, Args)]
pub struct TagsArgs {
    /// Limite de tags.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd ask --suggest` (D158).
#[derive(Debug, Args)]
pub struct SuggestArgs {
    /// Nº máximo de vizinhos por nota.
    #[arg(long = "top-k", value_name = "N", default_value_t = 5)]
    pub top_k: usize,
    /// Restringe a uma relação: `duplicate`/`contradiction`/`link`.
    #[arg(long, value_name = "RELAÇÃO")]
    pub relation: Option<String>,
    /// Limite de sugestões.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}
