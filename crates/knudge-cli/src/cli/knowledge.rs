//! Subcomandos de `kd knowledge` — mapa/ranking/vocabulário de conhecimento (D128/D146).

use clap::{Args, Subcommand};

/// Subcomandos de conhecimento.
#[derive(Debug, Subcommand)]
pub enum KnowledgeCommand {
    /// Mapa de conhecimento: clusters por eixo estrutural (fase 1) e, com `--semantic`,
    /// semântico (fase 2).
    Map(KnowledgeMapArgs),
    /// Digere o conteúdo num vetor (fila de embeddings; ex-`maintenance index`, D145).
    Digest(KnowledgeDigestArgs),
    /// Notas mais confiáveis, sem pergunta textual (ex-`ask --rank`, D146).
    Rank(KnowledgeRankArgs),
    /// Vocabulário de tags (`tag|count`, `count` desc — ex-`ask --tags`, D146).
    Tags(KnowledgeTagsArgs),
    /// Sugestões semânticas de aresta/contradição entre notas (read-only; D158).
    Suggest(KnowledgeSuggestArgs),
    /// Promove conhecimento a regras governadas no `AGENTS.md` (D157).
    Promote {
        /// Subcomando de promoção.
        #[command(subcommand)]
        command: PromoteCommand,
    },
}

/// Subcomandos de `kd knowledge promote` (D157).
#[derive(Debug, Subcommand)]
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

/// Argumentos de `kd knowledge promote recommend`.
#[derive(Debug, Args)]
pub struct PromoteRecommendArgs {
    /// Varredura do projeto inteiro (sem filtro de trabalho).
    #[arg(long)]
    pub universe: bool,
    /// Limite de candidatas.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd knowledge promote approve|remove`.
#[derive(Debug, Args)]
pub struct PromoteTargetArgs {
    /// Id da nota.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Varredura do projeto inteiro (sem filtro de trabalho).
    #[arg(long)]
    pub universe: bool,
}

/// Argumentos de `kd knowledge promote edit`.
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

/// Argumentos de `kd knowledge digest`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "`--drain`/`--status` são ações mutuamente exclusivas da CLI"
)]
#[derive(Debug, Args)]
pub struct KnowledgeDigestArgs {
    /// Drena um lote da fila agora (repita para drenar mais).
    #[arg(long)]
    pub drain: bool,
    /// Mostra o estado da fila (default).
    #[arg(long)]
    pub status: bool,
}

/// Argumentos de `kd knowledge map`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct KnowledgeMapArgs {
    /// Restringe a um eixo: `anchor`/`type`/`classification`/`scope`.
    #[arg(long, value_name = "EIXO")]
    pub axis: Option<String>,
    /// Restringe aos membros de um escopo (épico).
    #[arg(long, value_name = "ESCOPO")]
    pub scope: Option<String>,
    /// Roda a fase 2 semântica dentro dos clusters (requer embeddings).
    #[arg(long)]
    pub semantic: bool,
    /// Inclui os membros de cada cluster.
    #[arg(long)]
    pub members: bool,
    /// Materializa o mapa: `notas/MAP.md` + uma nota-hub (`references`) por cluster (D150).
    #[arg(long)]
    pub write: bool,
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

/// Argumentos de `kd knowledge rank`.
#[derive(Debug, Args)]
pub struct KnowledgeRankArgs {
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
    /// Varredura explícita do projeto inteiro.
    #[arg(long)]
    pub universe: bool,
    /// Limite de resultados.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd knowledge tags`.
#[derive(Debug, Args)]
pub struct KnowledgeTagsArgs {
    /// Limite de tags.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Argumentos de `kd knowledge suggest` (D158).
#[derive(Debug, Args)]
pub struct KnowledgeSuggestArgs {
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
