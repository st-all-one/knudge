//! Argumentos de `kd rewind` (E12-T01, D57/D143).

use clap::Args;

/// Argumentos de `kd rewind`.
#[derive(Debug, Args)]
pub struct RewindArgs {
    /// Container/domínio de escopo.
    #[arg(long, value_name = "CONTAINER")]
    pub scope: Option<String>,
    /// Working set por arquivos.
    #[arg(long, value_name = "PATH")]
    pub files: Vec<String>,
    /// Orçamento de tokens.
    #[arg(long, value_name = "N")]
    pub budget: Option<usize>,
    /// Início do intervalo.
    #[arg(long, value_name = "TS")]
    pub since: Option<String>,
    /// Fim do intervalo.
    #[arg(long, value_name = "TS")]
    pub until: Option<String>,
    /// Retoma um contexto por id (handoff 1:1).
    #[arg(long, value_name = "CONTEXT_ID")]
    pub resume: Option<String>,
    /// Filtro por tipo (repetível) — D143.
    #[arg(long = "type", value_name = "TIPO")]
    pub types: Vec<String>,
    /// Filtro por classificação (repetível) — D143.
    #[arg(long = "class", value_name = "CLASSE")]
    pub classes: Vec<String>,
    /// Filtro por tag (repetível; basta uma) — D143.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Filtro por âncora (repetível; aceita lista com vírgula) — D143.
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    pub anchor: Vec<String>,
    /// Ponto de partida: vizinhança de uma nota pelo grafo — D143.
    #[arg(long, value_name = "ID")]
    pub around: Option<String>,
    /// Profundidade da vizinhança de `--around` — D143.
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub depth: u8,
}
