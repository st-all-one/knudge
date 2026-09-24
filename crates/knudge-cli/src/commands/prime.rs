//! `kd prime` — protocolo estático, byte-idêntico por versão (D57, E12-T01).
//!
//! É o "help da IA": tipos, tools, regras, orçamento e formato de saída, **sempre a mesma
//! resposta** para uma dada versão do binário (cacheável). `kd` sem argumentos = `kd prime`;
//! `--long` anexa a gramática TOON e o schema completo.

use knudge_core::schema::{CANONICAL_KEYS, NoteType};
use serde_json::json;

use crate::output::Output;

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Corpo estático do protocolo. `{TYPES}` é preenchido a partir do schema (fonte única).
const PRIME_BODY: &str = "\
knudge (kd) — memória por projeto, otimizada para LLM.
TIPOS: {TYPES}
CLASSIFICAÇÃO: foundational, tactical, observational
STATUS: active, in_progress, blocked, closed, superseded, forgotten

CICLO: kd ask (buscar) → kd write (gravar conhecimento) → kd task (executar) → kd sync (commit).
  Sempre busque antes de gravar (evita duplicata); para gastar menos, use --limit N e --brief.

GUIA RÁPIDO (o que usar, quando e quando NÃO usar):
  kd ask <QUERY>     RECUPERAR antes de agir; use no início de qualquer tarefa.
                     NÃO use para criar/editar (é read-only) nem como histórico (use rewind).
  kd write <...>     GRAVAR fato/decisão/erro/risco/pergunta. Antes, `kd ask \"<rascunho>\"`.
                     NÃO use para trabalho (`--type task` é rejeitado) — use kd task.
  kd task new <...>  PLANEJAR/EXECUTAR trabalho (epic ⊃ {issue ⊃ task | task}).
                     NÃO use para conhecimento/observação — use kd write.
  kd rewind          RECONSTRUIR contexto no início de sessão (orçamento de tokens).
                     NÃO use como busca dirigida — use kd ask.
  kd knowledge map   VISÃO de clusters (estrutural/semântico); exige escopo ou --universe.
                     NÃO use para achar 1 nota (use kd ask) nem para ranking (kd knowledge rank).
  kd maintenance ..  SAÚDE (doctor/--audit), limpeza (compact/learn/prune).
                     learn/compact/prune só PROPÕEM e exigem escopo (ou --universe).
  kd forget --id <ID>  SOFT-DELETE (--restore; --purge só após retenção).
  kd sync            COMMIT de notas/ + eventos/ (--message).
  kd init|config|self|prime — fundação, ajustes, binário e este protocolo.

ÂNCORAS (--anchor PATH) — o que liga a memória ao código (alto valor):
  Toda nota/tarefa que fala de um arquivo ou módulo deve ser ancorada:
    kd write --summary \"Cache usa LRU\" --type decision --anchor src/cache.rs --anchor src/cache/**
    kd task new --summary \"Migrar cache\" --scope task --anchor plan/016.md
  Repetível; aceita vírgula (`--anchor a,b --anchor c`). Glob (`src/**`) casa subárvores.
  `kd ask --anchor PATH` busca só o ancorado, mesmo sem query textual (canal de âncoras).
  Onde NÃO usar: nota de conceito global sem arquivo, ou path que ainda não existe.
  Manutenção: `kd maintenance doctor --audit` lista âncoras quebradas (arquivo removido).

CONHECIMENTO (kd write — fatos, decisões, erros, riscos, perguntas):
  1. kd ask \"<rascunho>\"                # dedup lexical (BM25) antes de criar
  2. score < 0.75 cria | 0.75-0.92 faz merge | >= 0.92 rejeita
  kd write --summary \"<...>\" [<corpo>|-] [--type <fact|decision|error|risk|question>] [--tag T...] [--anchor PATH...]
  kd write --update <ID> --summary \"<...>\"   # versiona; mudar type/statement cria novo id + supersede
  kd write --link <FROM:ARESTA:TO>        # aresta explícita (via única; inclui depends_on)
  kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]   # evidência (D55)
  kd write --batch - [--dry-run]          # lote JSONL de rascunhos (D110)
  kd write --params '<json>' [--dry-run]  # item único (D147)
  --type task é rejeitado: use kd task.

PESQUISA (kd ask — só conhecimento por padrão; --with-task inclui trabalho):
  kd ask <QUERY> [--type T...] [--class C...] [--tag T...] [--status S...] [--scope ID]
        [--anchor PATH...] [--since TS] [--until TS] [--limit N] [--brief] [--full-content]
  kd ask --params '<json>'                # consulta + filtros (D147); '-' lê a query do stdin
  kd ask --id <ID>...                     # corpos por id (inclui trabalho)
  kd ask --around <ID> [--via ARESTA] [--depth N]   # expande o grafo
  Pipe (LLM): id|statement|score|why  (1 hit por linha). Corpo só com --id/--full-content.
  Busca vazia → stdout [no_results] (D152); --json traz channels por hit (D151).
  forgotten/superseded ficam fora do ask por padrão; use --status para incluí-los.
  Rank (--rank) e vocabulário de tags (--tags) agora em kd knowledge rank|tags (D146).

TAREFAS (kd task — epic ⊃ { issue ⊃ task | task }; epic é a raiz, ancore-o):
  kd task new --summary \"<...>\" [<corpo>|-] --scope <epic|issue|task> [--kind error|question|risk|decision]
        [--parent ID] [--checks NOME] [--tag T] [--anchor PATH] [--params '<json>'|--batch FONTE]
  kd task list <filtro> [--sort impact] [--full-content]   # filtro: --scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor, ou --universe
  kd task show --id <ID> [<ID>...] [--history]   # + corpo/checks/ancoras/tags/outcomes (D125/D127/D137)
  kd task update --id <ID> ...              # edita campos no lugar
  kd task close --id <ID> [--outcome S] [--note TXT]   # só declara com evidência (D55); + épico e progresso (D127)
  kd task graph [--program plan/<slug>.md|--root ID]   # id|kind|status|statement (done/total) (D139)
  kd task plan <ID> --prompt [--template T] | --submit --from -   # plano TOON (D105)

ESTADO / HANDOFF: kd rewind [--scope CONTAINER] [--files PATH...] [--budget N] [--resume ID]
  Filtros de corpus: --tag/--anchor/--type/--class/--around ID (D143).
  Orçamento ceil(len/4) tokens (default 4000); context_id retomável 1:1; next:/fresh: (D106).
MAPA/RANK: kd knowledge map|digest|rank|tags ...  (map/rank exigem escopo ou --universe; D143/D145/D146)
  kd knowledge map [--axis anchor|type|classification|scope] [--scope C] [--semantic] [--members] [--write]
  kd knowledge digest --status|--drain  (digere o conteúdo num vetor; ex-maintenance index)
MANUTENÇÃO: kd maintenance doctor [--audit]|compact|learn|prune|watch-service  (learn/compact/prune só propõem; exigem escopo ou --universe)
CICLO DE VIDA: kd forget --id <ID>  (soft; --restore; --purge após retenção)
CONFIG: kd config get --key K | set --key K --value V | unset --key K | list [--global]  (strict é config, não flag)
INIT/SYNC: kd init  (funda .knudge/ + AGENTS.md)  ·  kd sync  (commit de notas/ + eventos/)

ID: <tipo>_<base36(8)> = hash(type + U+001F + normalize(statement)); reclassificar não reescreve o id.
TOON: frontmatter em ordem canônica; opcionais omitidos, nunca null.
REGRAS: nunca invente id; statement curto e autocontido; ancore código com --anchor PATH.
SAÍDA: stdout = dados, stderr = logs. --json = {success, command, data?, error{code,message,retryable}, warnings?}
EXIT: 2 invalid, 3 not_found, 4 conflict, 5 io, 6 timeout, 7 config, 8 schema, 70 internal
";

/// Formato de `prime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimeFormat {
    /// Protocolo curto (byte-idêntico).
    Short,
    /// Protocolo + schema/gramática.
    Long,
}

/// Resposta de `prime` (estática; `Long` anexa schema/gramática).
#[must_use]
pub fn run(format: PrimeFormat) -> Output {
    let text = match format {
        PrimeFormat::Short => prime_text(),
        PrimeFormat::Long => format!("{}{}", prime_text(), long_section()),
    };
    let data = match format {
        PrimeFormat::Short => json!({ "protocol": &text, "version": VERSION }),
        PrimeFormat::Long => json!({
            "protocol": &text,
            "version": VERSION,
            "types": NoteType::ALL.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            "keys": CANONICAL_KEYS,
        }),
    };
    Output::new(text, data)
}

/// Protocolo com os tipos vindos do schema (fonte única).
fn prime_text() -> String {
    let types = NoteType::ALL
        .iter()
        .map(|note_type| note_type.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    PRIME_BODY.replace("{TYPES}", &types)
}

fn long_section() -> String {
    let types: Vec<&str> = NoteType::ALL.iter().map(|t| t.as_str()).collect();
    let keys: Vec<&str> = CANONICAL_KEYS.to_vec();
    format!(
        "\ntipos: {}\nchaves canônicas ({}): {}\n",
        types.join(", "),
        keys.len(),
        keys.join(", ")
    )
}
