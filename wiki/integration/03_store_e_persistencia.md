# 03 — Store e persistência

`notas/` é a **verdade**; `eventos/` é auditoria; `.idx/`, `cache/` e `.locks/` são derivados
descartáveis (D20/D21/D84). Especificação: [`../specs/persistencia.md`](../specs/persistencia.md).

## `Store`

```rust
use knudge_core::store::{Store, LockPolicy, LockGuard, acquire, Staging, purge_derived};

let store = kd.store();                 // Store<'_> ancorado no knowledge_dir
store.ensure_dirs()?;                   // cria notas/ e pastas por tipo (D150)

store.exists("fact_01m81b6h");          // canônico ou legado
let note = store.read("fact_01m81b6h")?;
let maybe = store.read_optional("fact_x")?;   // None para inválida/ausente
let ids = store.list_ids()?;            // ordenado e determinístico
store.write(&note)?;                    // tmp + rename (D20)
store.sync("fact_01m81b6h")?;           // fsync explícito
store.remove("fact_01m81b6h")?;         // canônico + purga do derivado (D84)
let revision = store.update("fact_x", |note| { /* muta */ Ok(()) })?;
```

- `Store::new(fs, root)` aceita **qualquer** raiz — inclusive um layout alternativo (D214).
- `note_path(id)` = `notas/<type_dir>/<id>.md`; `resolve_path` tolera o layout plano legado
  (D150) para `doctor --fix` migrar.
- Escrita canônica é **atômica**; `write` não registra evento (quem registra é `commit`).

## Ordem de commit (D20/D21)

```rust
use knudge_core::store::{commit, Event};

let event = Event::new("write", now_ms).with_note_id(note.id()?);
commit(&store, &kd.events(), &note, &event)?;
```

**Nota primeiro, evento depois.** Um crash entre os dois deixa a nota visível e sem evento — a
reconstrução tolera isso. Nunca inverta a ordem.

## Lock advisory (D23–D25)

Protege dois processos (CLI + MCP) escrevendo o mesmo alvo. O `LockGuard` é RAII (libera no
`Drop`).

```rust
use knudge_core::store::{LockPolicy, acquire};

let lock = acquire(
    kd.fs_dyn(),
    &knudge_core::adapters::SystemClock::new(),
    &mut knudge_core::adapters::ThreadRng::new(),
    &kd.knowledge_dir().join(".locks").join("fact_x.lock"),
    LockPolicy::default(),   // stale 30s, 50 tentativas, jitter 20ms
)?;
// ... escreve ...
drop(lock); // remove o arquivo
```

- Lock **stale** (idade > `stale_ms`) é reclamado por `rename` atômico.
- `doctor --fix` também reclaimed locks; a varredura de resíduos **não** toca `*.lock` (D160).

## Eventos (auditoria)

```rust
use knudge_core::store::{Event, EventLog};

let log = kd.events(); // EventLog::new(fs, knowledge_dir, DEFAULT_MAX_BYTES)
log.append(&Event::new("task", now_ms).with_note_id(id))?;

let (events, warnings) = log.read_all()?;      // tolerante (linha ruim vira aviso)
let hist = log.history(id)?;                  // eventos da nota
let imported = log.read_since_checkpoint()?;  // incremental
log.mark_checkpoint(last_event_id)?;
```

- Segmentos rotacionam acima de `max_bytes` (`events.jsonl` → `events-NNNN.jsonl`).
- `Event` tem `{id, op, note_id?, at, actor?, data?}`; o `id` deriva do conteúdo.

## Rebuild e descarte

```rust
use knudge_core::store::Staging;

let staging = Staging::begin(kd.fs_dyn(), kd.knowledge_dir().join(".idx"))?;
staging.write("retrieval.jsonl", bytes)?;   // grava no `.idx.new/`
staging.commit()?;                          // fsync + rename atômico (D22/D27)

knudge_core::store::purge_derived(kd.fs_dyn(), &kd.knowledge_dir(), id)?;
```

`Staging` é o double-buffer: um leitor nunca vê índice parcial.

## Resíduos no start (R10/D160)

```rust
let logger = MeuLogger; // implementa ports::Logger
let warnings = kd.sweep_residues(&logger); // remove *.tmp/*.stale antigos
```

Best-effort (R33): falha vira aviso. Nunca toca `*.lock`.

## Recomendações

- **Não escreva direto no FS** o `.md`: use `Store::write`/`write::write`. Você perde atomicidade,
  idempotência e evento.
- **Sempre passe por `commit`** ao criar/alterar nota canônica.
- **Lock para read-modify-write concorrente.** Se dois agentes fazem `update`, adquira o lock do id
  alvo antes.
- **Purge é explícito.** Remover nota não limpa sozinho todo derivado; use `purge_derived`
  (ou o fluxo de `forget`).
- **Não versionar derivado.** `.idx/`, `cache/`, `.locks/` são reconstruíveis; o `onboard` cuida do
  `.git/info/exclude` (doc 11).
