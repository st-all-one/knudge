# 05 — Escrita, dedup e ciclo de vida de notas

Toda escrita canônica passa pelo protocolo `write`: idempotência por id → dedup por
similaridade → decisão (create/merge/reject). Especificação:
[`../specs/escrita-e-dedup.md`](../specs/escrita-e-dedup.md).

## `WriteContext` e `Draft`

```rust
use knudge_core::write::{Draft, write, thresholds_from_config};
use knudge_core::schema::{NoteType, Classification, EdgeKind, Claim};

let ctx = kd.write_context()?;            // Store + EventLog + Index + now_ms
let thresholds = thresholds_from_config(kd.config())?;

let mut draft = Draft::new(NoteType::Decision, "Cache usa LRU com teto de 32 MiB")
    .with_body("Por quê: ...\nConsequência: ...");
draft.tags = vec!["cache".into()];
draft.anchors = vec!["src/cache.rs".into(), "src/cache/**".into()];
draft.classification = Some(Classification::Tactical);
draft.edges.push((EdgeKind::References, "fact_abc".into()));
draft.claims.push(Claim::new("cache", "usa", "LRU"));

let outcome = write(&ctx, &draft, &thresholds)?;
match outcome.action {
    knudge_core::write::WriteAction::Created  => println!("criada {}", outcome.id),
    knudge_core::write::WriteAction::Merged   => println!("fundida em {}", outcome.id),
    knudge_core::write::WriteAction::Rejected => println!("duplicata forte de {}", outcome.id),
    knudge_core::write::WriteAction::Unchanged => println!("já existia igual"),
    knudge_core::write::WriteAction::Updated  => println!("atualizada"),
}
```

- `Draft::new(type, statement)`; `with_body(...)`; campos públicos para o resto.
- `write` **não** cria `Task`/`Epic` (D93): use `task::submit` (doc 06).
- Mesmo id + mesmo `body_hash` → `Unchanged` (idempotente). Mesmo id + corpo diferente →
  `Conflict`: use `update`.
- Limiares `create_below`/`merge_below` vêm de `[dedup]` (defaults 0.75/0.92).

### Antes de gravar, consulte

O dedup interno compara com o **índice** (`ctx.index()`). Para uma pós-busca mais rica, rode
`recall` antes e ofereça ao usuário; mas o `write` já evita duplicata.

## Actualizações e supersessão

```rust
use knudge_core::write::{Patch, update, UpdateOutcome};

let mut patch = Patch::default();
patch.body = Some("novo corpo".into());
patch.tags = Some(vec!["cache".into(), "lru".into()]);

let result = update(&ctx, "decision_abc", &patch)?;
// revisão no lugar quando a chave de conteúdo (type+statement) não muda;
// muda type/statement → supersede (novo id + replaces/superseded_by).
```

- `Patch` só mexe no que está `Some`; `type`/`statement` diferentes disparam **supersede** (D01).
- `history(&store, id)?` devolve a linhagem de revisões.

## Ciclo de vida (soft-delete e arestas)

```rust
use knudge_core::write::{forget, restore, link, outcome, OutcomeStatus};
use knudge_core::schema::EdgeKind;

forget(&ctx, "fact_x", Some("obsoleto"))?;  // status = Forgotten (não apaga)
restore(&ctx, "fact_x")?;                    // Forgotten -> Active
link(&ctx, "fact_x", EdgeKind::Supports, "fact_y")?;
outcome(&ctx, "task_x", OutcomeStatus::Success, Some("evidência"))?;
```

- `forget`/`restore` são transições de `status` validadas por `validate_transition`.
- Membro de ciclo de supersessão/dependência é **protegido** (D45).
- `outcome` registra evidência e alimenta a confiança derivada (doc 09).

## Lote (JSONL)

```rust
use knudge_core::write::{batch_jsonl, BatchMode};

let out = batch_jsonl(&ctx, &jsonl_text, &thresholds, BatchMode::Apply);
// BatchMode::DryRun | Apply; o resultado agrega created/merged/rejected/errors
```

Cada linha é um rascunho JSON (`DRAFT_KEYS`). Linha inválida não derruba o lote: vira erro
por linha.

## Reconciliação off-path

```rust
use knudge_core::write::propose_merges;

let propostas = propose_merges(&ctx.index(), &thresholds); // só propõe (D47)
```

`propose_merges` usa peneira exata em corpus pequeno e MinHash/LSH em corpus denso grande
(D204).

## Hooks de ciclo de vida

O core define a porta `ports::HookRunner` e o adapter `adapters::ProcessHookRunner`. O
mapeamento evento→config (`hooks.pre_record`, …) e a execução ficam na borda — no core você
injeta a porta se quiser validar/mutar um `Draft` antes de gravar.

## Recomendações

- **Busque antes de gravar.** Evita duplicata e alimenta a ontologia.
- **Uma afirmação por nota.** `statement` curto e autocontido; detalhe vai no `body`.
- **Ancore código.** `--anchor`/`draft.anchors` liga a nota a arquivos e move o recall.
- **Use `update` para corrigir**, não `write` com o mesmo statement e corpo diferente.
- **Serialização concorrente:** lock no id alvo para read-modify-write (doc 03).
- **`strict` muda a escrita?** Não a forma; apenas promove avisos (slots/claims) a erro.
- **Não guarde derivados** (confiança, índice): são recalculados.
