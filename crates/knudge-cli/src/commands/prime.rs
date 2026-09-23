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

CONHECIMENTO (kd write — fatos, decisões, erros, riscos, perguntas):
  1. kd ask \"<rascunho>\"                # dedup lexical (BM25) antes de criar
  2. score < 0.75 cria | 0.75-0.92 faz merge | >= 0.92 rejeita
  kd write --type <fact|decision|error|risk|question> \"<...>\" [--body -|TXT] [--tag T...] [--anchors PATH...]
  kd write --update <ID> \"<...>\"       # versiona; mudar type/statement cria novo id + supersede
  kd write --link <FROM:ARESTA:TO>        # aresta explícita (via única; inclui depends_on)
  kd write --outcome <success|partial|failure|abandoned> <ID> [--note TXT]   # evidência (D55)
  kd write --batch - [--dry-run]          # lote JSONL de rascunhos (D110)
  --type task|container é rejeitado: use kd task.

PESQUISA (kd ask — uma tool para tudo):
  kd ask <QUERY> [--type T...] [--class C...] [--tag T...] [--status S...] [--container ID]
        [--anchor PATH...] [--since TS] [--until TS] [--limit N] [--brief] [--with-body]
  kd ask --id <ID>...                     # corpos por id
  kd ask --around <ID> [--via ARESTA] [--depth N]   # expande o grafo
  kd ask --rank                           # mais confiáveis, sem query (D107)
  kd ask --tags                           # vocabulário de tags
  Pipe (LLM): id|statement|score|why  (1 hit por linha). Corpo só com --id/--with-body.
  forgotten/superseded ficam fora do ask por padrão; use --status para incluí-los.

TAREFAS (kd task — plan ⊃ epic ⊃ issue ⊃ task, máx. 4):
  kd task new \"<...>\" --scope <plan|epic|issue|task> [--kind error|question|risk|decision]
        [--parent ID] [--checks NOME] [--body TXT] [--tag T] [--anchor PATH]
  kd task list --ready|--blocked [--explain] [--sort impact] [--kind K] [--tag T] [--owner A|--mine]
  kd task show <ID> [<ID>...] [--history]   # + contexto (parent/blocked_by/children) e épico (D125/D127)
  kd task update <ID> ...                   # edita campos no lugar
  kd task close <ID> [--outcome S] [--note TXT]   # só declara com evidência (D55); + épico e progresso (D127)
  kd task claim <ID> --by <agente>|--release       # dono derivado (D114)
  kd task graph [--program plan/<slug>.md|--root ID]   # role|kind|status|owner|mode|progresso (D116/D127)
  kd task plan <ID> --prompt [--template T] | --submit --from -   # plano TOON (D105)

ESTADO / HANDOFF: kd rewind [--scope CONTAINER] [--files PATH...] [--budget N] [--resume ID]
  Orçamento ceil(len/4) tokens (default 4000); context_id retomável 1:1; next:/fresh: (D106).
MAPA: kd knowledge map [--axis anchor|type|classification|container] [--scope C] [--semantic] [--members]
MANUTENÇÃO: kd maintenance doctor [--audit]|compact|eval|index|learn|prune  (learn/compact/prune só propõem)
CICLO DE VIDA: kd forget <ID>  (soft; --restore; --purge após retenção)
CONFIG: kd config get|set|unset|list [--global]  (strict é config, não flag)
INIT/SYNC: kd init  (funda .knudge/ + AGENTS.md)  ·  kd sync  (commit de notas/ + eventos/)

ID: <tipo>_<base36(8)> = hash(type + U+001F + normalize(statement)); reclassificar não reescreve o id.
TOON: frontmatter em ordem canônica; opcionais omitidos, nunca null.
REGRAS: nunca invente id; statement curto e autocontido; ancore código com --anchors PATH.
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
