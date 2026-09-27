//! `kd prime` — protocolo estático, byte-idêntico por versão (D57, E12-T01).
//!
//! É o "help da IA": tipos, tools, regras, orçamento e formato de saída, **sempre a mesma
//! resposta** para uma dada versão do binário (cacheável). `prime` é **compacto por padrão**
//! (D166); `--long` anexa a gramática TOON e o schema completo.

use knudge_core::schema::{CANONICAL_KEYS, NoteType};
use serde_json::json;

use crate::output::Output;

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocolo **compacto** (default): o essencial para operar, byte-idêntico por versão.
const PRIME_COMPACT: &str = "\
knudge (kd) — memória por projeto, otimizada para LLM.
TIPOS: {TYPES}
CLASSIFICAÇÃO: foundational, tactical, observational
STATUS: active, in_progress, blocked, closed, superseded, forgotten

CICLO: kd ask (buscar) → kd write (gravar) → kd task (executar) → kd sync (commit).
  Sempre busque antes de gravar (evita duplicata); use --limit N e --brief para gastar menos.

GUIA (o que usar, quando e quando NÃO usar):
  kd ask <QUERY>      RECUPERAR antes de agir (read-only); --anchor/--type/--limit/--brief/--full-content.
  kd write <...>      GRAVAR fato/decisão/erro/risco/pergunta; ancore o código com --anchor PATH.
  kd task new <...>   PLANEJAR/EXECUTAR trabalho (epic ⊃ {issue ⊃ task | task}); NÃO use p/ conhecimento.
  kd rewind           RECONSTRUIR contexto/handoff no início de sessão (orçamento de tokens).
  kd doctor           SAÚDE do corpus: 13 checks + auditoria (--fix corrige; --explain detalha).
  kd drain            FILA: --status (estado) | --digest [--force] (digerir); service = worker.
  kd forget --id <ID>  SOFT-DELETE (--restore; --purge [--force] após retenção).
  kd sync             COMMIT de notas/ + eventos/ (--message).
  kd init|config|self|prime|help — fundação, ajustes, binário, protocolo e ajuda.

ESSENCIAL:
  kd ask \"<QUERY>\" [--anchor PATH...] [--type T...] [--limit N] [--brief] [--full-content]
  kd write --summary \"<afirmação>\" [<corpo>|-] --type <fact|decision|error|risk|question> [--tag T...] [--anchor PATH...]
  kd task new --summary \"<...>\" --scope <epic|issue|task> [--parent ID] [--anchor PATH...]
  ÂNCORAS: --anchor PATH liga a nota/tarefa ao código (repetível; aceita vírgula; glob `src/**`).
  CORPO: escreva quando o statement sozinho NÃO permite agir (Por quê / Evidência / Consequência).

ID: <tipo>_<base36(8)> = hash(type + U+001F + normalize(statement)); reclassificar não reescreve o id.
REGRAS: nunca invente id; statement curto e autocontido; ancore código com --anchor PATH.
SAÍDA: stdout = dados, stderr = logs. --json = {success, command, data?, error{code,message,retryable}, warnings?}
EXIT: 2 invalid, 3 not_found, 4 conflict, 5 io, 6 timeout, 7 config, 8 schema, 70 internal
";

/// Protocolo **completo** (`--long`): todos os verbos e detalhes. `{TYPES}` vem do schema.
const PRIME_FULL: &str = "\
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
  kd map              VISÃO de clusters (estrutural/semântico); exige escopo ou --universe.
                     NÃO use para achar 1 nota (use kd ask) nem para ranking (kd ask --rank).
  kd doctor          SAÚDE do corpus (13 checks + auditoria; reparo com --fix, detalhe com --explain).
  kd maintenance ..  LIMPEZA (compact/learn/prune; só PROPÕEM e exigem escopo ou --universe).
  kd forget --id <ID>  SOFT-DELETE (--restore; --purge só após retenção, [--force]).
  kd sync            COMMIT de notas/ + eventos/ (--message).
  kd init|config|self|prime — fundação, ajustes, binário e este protocolo.

ÂNCORAS (--anchor PATH) — o que liga a memória ao código (alto valor):
  Toda nota/tarefa que fala de um arquivo ou módulo deve ser ancorada:
    kd write --summary \"Cache usa LRU\" --type decision --anchor src/cache.rs --anchor src/cache/**
    kd task new --summary \"Migrar cache\" --scope task --anchor plan/016.md
  Repetível; aceita vírgula (`--anchor a,b --anchor c`). Glob (`src/**`) casa subárvores.
  `kd ask --anchor PATH` busca só o ancorado, mesmo sem query textual (canal de âncoras).
  Onde NÃO usar: nota de conceito global sem arquivo, ou path que ainda não existe.
  Manutenção: `kd doctor` lista âncoras quebradas (arquivo removido).

CONHECIMENTO (kd write — fatos, decisões, erros, riscos, perguntas):
  1. kd ask \"<rascunho>\"                # dedup lexical (BM25) antes de criar
  2. score < 0.75 cria | 0.75-0.92 faz merge | >= 0.92 rejeita
  kd write --summary \"<...>\" [<corpo>|-] [--type <fact|decision|error|risk|question>] [--tag T...] [--anchor PATH...]
  kd write --update <ID> --summary \"<...>\"   # versiona; mudar type/statement cria novo id + supersede
  kd write --update <ID> --params '<json>'  # patch (body/tags/anchors/...); --clear-anchors limpa
  kd write --link <FROM:ARESTA:TO>        # aresta explícita (via única; inclui depends_on)
  kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]   # evidência (D55)
  kd write --batch - [--dry-run]          # lote JSONL de rascunhos (D110)
  kd write --params '<json>' [--dry-run]  # item único (D147)
  --type task é rejeitado: use kd task.

CORPO (quando e por quê) — o \"porquê\" que não cabe no statement:
  Escreva corpo quando o statement sozinho NÃO permite agir. Template (2–4 linhas):
    Por quê: <motivo/decisão>
    Evidência: <comando, saída, erro, link>
    Consequência: <o que muda na prática>
  Na prática é esperado em decision/error/risk; dispensável em fact óbvio.
  O `ask` mostra o corpo (1º hit completo, 2–5 truncado); leia com --id/--full-content.

PESQUISA (kd ask — só conhecimento por padrão; --with-task inclui trabalho):
  kd ask <QUERY> [--type T...] [--class C...] [--tag T...] [--status S...] [--scope ID]
        [--anchor PATH...] [--since TS] [--until TS] [--limit N] [--brief] [--full-content]
  kd ask --params '<json>'                # consulta + filtros (D147); '-' lê a query do stdin
  kd ask --id <ID>...                     # corpos por id (inclui trabalho)
  kd ask --around <ID> [--via ARESTA] [--depth N]   # expande o grafo
  Pipe (LLM): id|statement|score|why (1 hit/linha); 1º hit com corpo, 2–5 truncado (D161).
              Corpo completo de todos com --full-content; --brief só id|statement.
  Busca vazia → stdout [no_results] (D152); --json traz channels por hit (D151).
  forgotten/superseded ficam fora do ask por padrão; use --status para incluí-los.
  Rank (`kd ask --rank`) e vocabulário de tags (`kd ask --tags`) — D146/D209.

TAREFAS (kd task — epic ⊃ { issue ⊃ task | task }; epic é a raiz, ancore-o):
  kd task new --summary \"<...>\" [<corpo>|-] --scope <epic|issue|task> [--kind error|question|risk|decision]
        [--parent ID] [--checks NOME] [--tag T] [--anchor PATH] [--params '<json>'|--batch FONTE]
  kd task list <filtro> [--sort impact] [--full-content]   # filtro: --scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor, ou --universe
  kd task show --id <ID> [<ID>...] [--history]   # + corpo/checks/ancoras/tags/outcomes (D125/D127/D137)
  kd task update --id <ID> ...              # edita no lugar (--anchor substitui; --clear-anchors limpa)
  kd task close --id <ID> [--outcome S] [--note TXT]   # só declara com evidência (D55); + épico e progresso (D127)
  kd task graph [--program plan/<slug>.md|--root ID]   # id|kind|status|statement (done/total) (D139)
  kd task plan <ID> --prompt [--template T] | --submit --from -   # plano TOON (D105)

ESTADO / HANDOFF: kd rewind [--scope CONTAINER] [--files PATH...] [--budget N] [--resume ID]
  Filtros de corpus: --tag/--anchor/--type/--class/--around ID (D143).
  Orçamento ceil(len/4) tokens (default 4000); context_id retomável 1:1; next:/fresh: (D106).
MAPA/RANK: kd map ...  ·  kd ask --rank|--tags ...  (map/rank exigem escopo ou --universe; D143/D146)
  kd map [--axis anchor|type|classification|scope] [--scope C] [--semantic] [--members] [--write]
  kd drain --status|--digest  (fila de embeddings: estado rico e digestão; --force redigeri tudo)
  kd drain service ..  (worker de auto-drain: install/subscribe/unsubscribe/status/uninstall)
SAÚDE: kd doctor [--fix] [--explain]  (13 checks + auditoria; reparo reversível; --explain detalha cada achado)
MANUTENÇÃO: kd maintenance compact|learn|prune  (só propõem; exigem escopo ou --universe)
CICLO DE VIDA: kd forget --id <ID>  (soft; --restore; --purge [--force] após retenção)
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
    /// Protocolo compacto (default; byte-idêntico).
    Compact,
    /// Protocolo completo + schema/gramática (`--long`).
    Long,
}

/// Resposta de `prime` (estática; `Long` anexa schema/gramática).
#[must_use]
pub fn run(format: PrimeFormat) -> Output {
    let text = match format {
        PrimeFormat::Compact => prime_text(PRIME_COMPACT),
        PrimeFormat::Long => format!("{}{}", prime_text(PRIME_FULL), long_section()),
    };
    let data = match format {
        PrimeFormat::Compact => json!({ "protocol": &text, "version": VERSION }),
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
fn prime_text(body: &str) -> String {
    let types = NoteType::ALL
        .iter()
        .map(|note_type| note_type.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    body.replace("{TYPES}", &types)
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
