//! Argumentos de `kd write`, `kd forget` e `kd sync`.

use clap::Args;

/// Argumentos de `kd write`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct WriteArgs {
    /// Corpo (Markdown); `-` lê stdin; vazio + pipe também lê stdin.
    #[arg(value_name = "BODY")]
    pub body: Vec<String>,
    /// Afirmação (chave TOON `statement`).
    #[arg(long, value_name = "TXT")]
    pub summary: Option<String>,
    /// Tipo da nota (default: `fact`).
    #[arg(long = "type", value_name = "TIPO")]
    pub note_type: Option<String>,
    /// Tags.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Âncoras (repetível; aceita lista com vírgula: `--anchor a,b`).
    #[arg(long = "anchor", value_name = "PATH", value_delimiter = ',')]
    pub anchors: Vec<String>,
    /// Limpa todas as âncoras (com `--update`).
    #[arg(long, conflicts_with = "anchors")]
    pub clear_anchors: bool,
    /// Classificação.
    #[arg(long = "class", value_name = "CLASSE")]
    pub class: Option<String>,
    /// Status inicial.
    #[arg(long, value_name = "STATUS")]
    pub status: Option<String>,
    /// Aresta explícita `ARESTA:ID` a partir da nota.
    #[arg(long, value_name = "ARESTA:ID")]
    pub edge: Vec<String>,
    /// Id da nota (usado com `--outcome`).
    #[arg(long, value_name = "ID")]
    pub id: Option<String>,
    /// Atualiza a nota existente.
    #[arg(long, value_name = "ID")]
    pub update: Option<String>,
    /// Cria aresta `FROM:ARESTA:TO`.
    #[arg(long, value_name = "FROM:ARESTA:TO")]
    pub link: Option<String>,
    /// Simula sem gravar.
    #[arg(long)]
    pub dry_run: bool,
    /// Aplica um lote de rascunhos JSONL (`-` lê stdin).
    #[arg(long, value_name = "FONTE")]
    pub batch: Option<String>,
    /// Objeto JSON de um rascunho (`-` lê stdin) — D147.
    #[arg(long, value_name = "JSON")]
    pub params: Option<String>,
    /// Anexa um resultado (`success|partial|failure|abandoned`) a uma nota existente.
    #[arg(long, value_name = "OUTCOME")]
    pub outcome: Option<String>,
    /// Texto do resultado (usado com `--outcome`).
    #[arg(long, value_name = "TXT")]
    pub note: Option<String>,
    /// Claim atômica `SUJEITO:RELAÇÃO:OBJETO` (repetível; D207).
    #[arg(long, value_name = "S:R:O")]
    pub claim: Vec<String>,
    /// Agente da proveniência — quem produziu a nota (D207).
    #[arg(long, value_name = "NOME")]
    pub agent: Option<String>,
    /// Atividade da proveniência — como a nota foi produzida (D207).
    #[arg(long, value_name = "NOME")]
    pub activity: Option<String>,
}

/// Argumentos de `kd forget`.
#[allow(clippy::struct_excessive_bools, reason = "flags de CLI")]
#[derive(Debug, Args)]
pub struct ForgetArgs {
    /// Id da nota.
    #[arg(long = "id", value_name = "ID")]
    pub id: String,
    /// Restaura em vez de esquecer.
    #[arg(long)]
    pub restore: bool,
    /// Remove fisicamente após a retenção.
    #[arg(long)]
    pub purge: bool,
    /// Com `--purge`, ignora a retenção e purga uma nota já aposentada (`forgotten`/`superseded`).
    #[arg(long)]
    pub force: bool,
}

/// Argumentos de `kd sync`.
#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Mensagem de commit.
    #[arg(long)]
    pub message: Option<String>,
}
