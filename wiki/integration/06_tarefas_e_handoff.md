# 06 — Tarefas e handoff

Trabalho é modelado como view derivada: `epic ⊃ {issue ⊃ task | task}` (D52/D53/D93/D149). O
`epic` é `scope=epic` com `type` omitido. Especificação: [`../specs/tarefas.md`](../specs/tarefas.md)
e [`../specs/handoff.md`](../specs/handoff.md).

## Criar tarefas (`TaskSpec` + `submit`)

```rust
use knudge_core::task::{TaskSpec, submit};
use knudge_core::schema::{Scope, NoteType, Status};

let ctx = kd.write_context()?;
let spec = TaskSpec {
    scope: Scope::Task,
    kind: None,                       // default por escopo
    statement: "Migrar o cache para LRU".into(),
    body: "Contexto...".into(),
    parent: Some("issue_abc".into()), // membership por marcador + results_in
    checks: vec!["cargo test".into()],
    anchors: vec!["src/cache.rs".into()],
    tags: vec!["cache".into()],
    blocks: None,                     // ordem 1-based no pai
    classification: None,
    status: Some(Status::Active),
};
let out = submit(&ctx, &spec)?;
println!("tarefa {} (pai {:?})", out.id, out.parent);
```

Use `submit` (não `write`) para `task`/`epic` — `write` rejeita (D93). Hierarquia e `blocks` são
validados; pai inválido vira `Schema`.

## Navegar o programa

```rust
use knudge_core::task::{context_of, progress_of, epic_of, impact, impacts, subtree, program_of};
use knudge_core::task::flow::{task_flows, durations_from_flows, critical_path};

let graph = kd.graph()?;
let store = kd.store();
let ctx_task = context_of(&store, &graph, "task_x")?;   // pai, blocked_by, blocks, filhos, épico
let progress = progress_of(&graph, "epic_x");           // contadores derivados
let peso = impact(&graph, "task_x");                    // nº de dependentes
let tudo = impacts(&graph);                             // mapa id -> impacto
let nos = subtree(&graph, "epic_x");                    // árvore do programa
let prog = program_of(&note, "src/**")?;                // programa por caminho/glob
```

## Fluxo, throughput e caminho crítico (D205)

```rust
let (events, _warnings) = kd.events().read_all()?;
let flows = task_flows(&events);
let duracoes = durations_from_flows(&flows, kd.now_ms());
let caminho = critical_path(&graph, &duracoes);
```

- `cycle_ms` (review − primeiro evento) e `lead_ms` (close/agora − create).
- `critical_path` é o maior caminho ponderado do DAG `depends_on` (empate determinístico).

## Handoff (`rewind`)

O `rewind` monta o contexto de início de sessão respeitando orçamento e grava um `context_id`
endereçável (D57/D88).

```rust
use knudge_core::handoff::{rewind, ContextStore, RewindInput, RewindRequest, RewindMode};
use knudge_core::lifecycle::Freshness;
use std::collections::BTreeMap;

let index = kd.index()?;
let graph = kd.graph()?;
let (events, _) = kd.events().read_all()?;
let changed = vec!["src/cache.rs".to_string()];

let input = RewindInput {
    index: &index,
    graph: &graph,
    events: &events,
    changed_paths: &changed,
    freshness: Freshness::default(),
    bodies: &BTreeMap::new(),
    scope: Default::default(),            // CorpusScope (filtro + vizinhança)
    task_confirmation_weight: 0.1,
    now_ms: kd.now_ms(),
};

let request = RewindRequest {
    mode: RewindMode::Auto,               // Manifest | Scope(id) | Files(paths) | Auto
    budget: knudge_core::handoff::DEFAULT_BUDGET, // ~4000 tokens
    ..RewindRequest::default()
};

let contexts = ContextStore::new(kd.fs_dyn(), kd.knowledge_dir());
let out = rewind(&input, &request, &contexts)?;

println!("context_id={} truncado={} descartados={}", out.context_id, out.truncated, out.dropped);
// próxima sessão: RewindRequest { resume: Some(out.context_id), .. } reproduz bytes idênticos (D88)
```

| Modo | Conteúdo |
|---|---|
| `Manifest` | ~30 tokens: contadores + recentes + dirty |
| `Scope(id)` | working set de um container/domínio |
| `Files(paths)` | notas ancoradas aos arquivos |
| `Auto` | deriva dos arquivos; vira manifest em corpus grande |

- `budget` é em tokens aproximados (`ceil(chars/4)`), sem tokenizer (D40/D82).
- `bodies` anexa corpo de `foundational`/`decision`; `freshness` informa fila de embeddings.

## Recomendações

- **Uma nota por unidade de trabalho.** `statement` = objetivo; corpo = critérios/contexto.
- **Ancore tarefas.** `anchors` liga ao código e aparece no `rewind`/`map`.
- **Feche com evidência.** Use `health::close_task`/`accept` (doc 08), que gravam `evidence` e
  inferem `outcome`.
- **`rewind` no início de sessão, `ask` para busca dirigida.** Não use `rewind` como busca.
- **Reaproveite `Index`/`Graph`** ao montar `RewindInput`; não recarregue por chamada.
