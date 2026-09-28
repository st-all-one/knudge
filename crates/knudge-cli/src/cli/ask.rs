//! Argumentos de `kd ask` — toda pesquisa, incluindo os modos `--rank`/`--tags`/`--suggest`
//! (absorvidos de `kd knowledge`, D209).

use clap::Args;

/// Argumentos de `kd ask`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Clone, Default, Args)]
pub struct AskArgs {
    /// Consulta textual (recall completo).
    #[arg(value_name = "QUERY")]
    pub query: Vec<String>,
    /// Objeto JSON com a consulta e os filtros (`-` lê stdin) — D147.
    #[arg(long, value_name = "JSON")]
    pub params: Option<String>,
    /// Recupera os corpos dos ids (repetível; aceita lista com vírgula: `--id a,b`).
    #[arg(
        long = "id",
        value_name = "ID",
        value_delimiter = ',',
        conflicts_with = "query"
    )]
    pub ids: Vec<String>,
    /// Expande o grafo a partir do id.
    #[arg(long, value_name = "ID", conflicts_with = "query")]
    pub around: Option<String>,
    /// Aresta do expand.
    #[arg(long, value_name = "ARESTA")]
    pub via: Option<String>,
    /// Profundidade do expand.
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub depth: u8,
    /// Saída mínima (`id|statement`).
    #[arg(long)]
    pub brief: bool,
    /// Inclui itens de trabalho (notas com `scope`) — D146.
    #[arg(long)]
    pub with_task: bool,
    /// Inclui o corpo completo dos hits (ex-`--with-body`, D146).
    #[arg(long)]
    pub full_content: bool,
    /// Filtro por tipo (repetível; aceita lista com vírgula: `--type a,b`).
    #[arg(long = "type", value_name = "TIPO", value_delimiter = ',')]
    pub types: Vec<String>,
    /// Filtro por classificação (repetível; aceita lista com vírgula: `--class a,b`).
    #[arg(long = "class", value_name = "CLASSE", value_delimiter = ',')]
    pub classes: Vec<String>,
    /// Filtro por tag (repetível; aceita lista com vírgula: `--tag a,b`).
    #[arg(long = "tag", value_name = "TAG", value_delimiter = ',')]
    pub tags: Vec<String>,
    /// Filtro por status.
    #[arg(long, value_name = "STATUS")]
    pub status: Option<String>,
    /// Filtro por escopo (épico).
    #[arg(long = "scope", value_name = "ID")]
    pub scope: Option<String>,
    /// Filtro por âncora (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    pub anchor: Vec<String>,
    /// Início do intervalo.
    #[arg(long, value_name = "TS")]
    pub since: Option<String>,
    /// Fim do intervalo.
    #[arg(long, value_name = "TS")]
    pub until: Option<String>,
    /// Reconstrói o corpus ativo num instante (RFC3339 ou data) — D155.
    #[arg(long = "as-of", value_name = "TS")]
    pub as_of: Option<String>,
    /// Limite de resultados.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
    /// Modo ranking (sem query): notas mais confiáveis — absorvido de `kd knowledge rank` (D146/D209).
    #[arg(long)]
    pub rank: bool,
    /// Modo vocabulário: contagem de tags (`tag|count`) — absorvido de `kd knowledge tags` (D146/D209).
    #[arg(long = "tags")]
    pub tags_vocab: bool,
    /// Modo sugestões semânticas de aresta/contradição (read-only) — D158.
    #[arg(long)]
    pub suggest: bool,
    /// Varredura explícita do projeto inteiro (modo `--rank`).
    #[arg(long)]
    pub universe: bool,
    /// Nº máximo de vizinhos por nota (modo `--suggest`).
    #[arg(long = "top-k", value_name = "N", default_value_t = 5)]
    pub top_k: usize,
    /// Restringe a uma relação: `duplicate`/`contradiction`/`link` (modo `--suggest`).
    #[arg(long, value_name = "RELAÇÃO")]
    pub relation: Option<String>,
}
