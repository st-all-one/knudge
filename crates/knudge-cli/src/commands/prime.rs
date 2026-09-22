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

ESCRITA (duas fases, idempotente por conteúdo):
  1. kd ask \"<rascunho>\"                # dedup lexical (BM25) antes de criar
  2. score < 0.75 cria | 0.75-0.92 faz merge | >= 0.92 rejeita
     kd write --type <T> \"<...>\" [--body -|TXT] [--tag T...] [--anchors PATH...]
     kd write --update <ID> \"<...>\"    # versiona, não sobrescreve
     kd write --link <FROM:ARESTA:TO>    # aresta explícita
     kd write --outcome <S> <ID>         # evidência: success|partial|failure|abandoned
     --type task|container é rejeitado: use kd task

PESQUISA (uma tool):
  kd ask [QUERY] [--id ID...] [--around ID] [--via ARESTA] [--depth N]
         [--type T...] [--class C...] [--tag T...] [--status S...] [--container ID]
         [--anchor PATH...] [--since TS] [--until TS] [--limit N] [--brief] [--with-body]
  Pipe (LLM): id|statement|score|why  (1 hit por linha). Corpo só com --id/--with-body.
  forgotten/superseded ficam fora do ask por padrão; use --status para incluí-los.

ESTADO / HANDOFF:
  kd rewind [--scope CONTAINER] [--files PATH...] [--budget N] [--resume ID]
  Orçamento ceil(len/4) tokens (default 4000); emite context_id retomável 1:1.

TAREFAS: kd task new|list|show|update|close|graph|plan  (plan ⊃ epic ⊃ issue ⊃ task, máx. 4)
  kd task list --ready|--blocked [--explain]   kd task graph --program plan/<slug>.md
MANUTENÇÃO: kd maintenance doctor|audit|compact|eval|index|learn  (learn/compact só propõem)
CICLO DE VIDA: kd forget <ID>  (soft; --restore; --purge após retenção)
CONFIG: kd config get|set|unset|list [--global]  (strict é config, não flag)
INIT: kd init  (funda .knudge/ + AGENTS.md)

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
