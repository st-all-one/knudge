# upgrade_plan.md — implementação das melhorias de conhecimento e tarefas

> **Status (implementado):** K7 (D102), K1 (D103), T1 (D104) e PR0.5 (D119). O restante segue
> pendente; ver `CHANGELOG.md` e as decisões em `plan/03_decisoes-fechadas.md`.
>
> Plano de execução do documento [`melhorias_conhecimento_tarefas.md`](melhorias_conhecimento_tarefas.md).
> Cada item traz **CLI**, **core**, **contrato/config**, **determinismo**, **erros**, **testes** e
> **aceite**. Regras invioláveis do projeto (AGENTS.md): `make check` verde; arquivos de `src/`
> ≤ 300 linhas; sem `unwrap/expect/panic/unsafe`; sem chave TOON nova (tudo deriva); sem
> dependência nova; núcleo puro via portas; `#[allow]` com `reason`.

---

## 0. Escopo

14 itens em 4 fases. Nenhum adiciona chave canônica ao TOON (as 28 de D95/D98/D100 permanecem).
Toda mudança de contrato vira uma decisão `Dxx` (§5) e um golden.

| Item | Fase | Verbo afetado | Nova chave config | Novo `Dxx` |
|---|---|---|---|---|
| K7 canal vetorial no `ask` | 0 | `ask` | `recall.semantic`, `recall.semantic_top_k` | D102 |
| K1 `--outcome` em notas | 0 | `write` | — | D103 |
| T1 `--ready/--blocked` | 0 | `task list` | — | D104 |
| **PR0.5 Programas externos** | **0** | `task graph`/`doctor` | `programs.glob` | **D119** |
| T3 plan prompt/template | 1 | `task plan` | — | D105 |
| T4 `next` no `rewind` + K6 frescor | 1 | `rewind` | — | D106 |
| K3 `ask --tags` | 1 | `ask` | — | D107 |
| X1 feedback tarefa→conhecimento | 1 | `ask`/`rewind` | — | D108 |
| T2/X3 impacto derivado | 2 | `task list` | — | D109 |
| K2 `ask --rank` | 2 | `ask` | — | D107 |
| K4 `write --batch` | 2 | `write` | `write.batch_max` | D110 |
| T5 filtros + `show` multi | 2 | `task list`/`show` | — | D104 |
| X2 `learn` sinal de tarefa | 2 | `maintenance learn` | — | D111 |
| T6 `close --note` | 3 | `task close` | — | D104 |
| K5 `maintenance prune` | 3 | `maintenance` | — | D112 |

**Ordem de merge sugerida:** PR1 = K7; PR2 = K1; **PR0.5 = programas externos (D119)**; PR3 = T1;
PR4 = T3+T4+K6; PR5 = K3+X1; PR6 = T2+K2+K4+T5+X2; PR7 = T6+K5. Cada PR fecha com `make check`
verde.

---

## 1. Fase 0 — correções e ganhos baratos

### K7 — ligar o canal vetorial ao `ask` (D102)

**Motivação.** `commands/ask.rs` monta o `RecallQuery` e nunca preenche `query.vector`
(`retrieval/mod.rs:65`), embora `recall()` já fusione o canal (`retrieval/mod.rs:127`) e o
índice vetorial exista (E11). `kd ask "fuso horário"` não acha a nota de UTC.

**CLI.** Sem flag nova (D94: comportamento é config). `kd ask` passa a incluir o canal quando
`recall.semantic = true` e o embedder está disponível. Fallback silencioso → BM25.

**Core (puro).**
- `embeddings/semantic.rs`: nova função
  ```rust
  /// Ranqueia ids do índice por similaridade com `query`, decrescente (empate: id asc).
  pub fn rank_query(index: &EmbeddingIndex, query: &[f32], top_k: usize, min_score: f64) -> Vec<String>
  ```
  reusa `vector::similarity`; filtra `min_score`; corta em `top_k`.
- Nada em `retrieval/` (o campo e a fusão já existem).

**Borda (`commands/ask.rs`).**
- `embedder::build(session)` → se `Some`, `embed(&[text])`, `EmbeddingIndex::load(...)`,
  `rank_query(...)` → `query.vector = Some(ids)`.
- Falha do provedor: `query.channel_warnings.push("vetorial: <erro>")` (degradação graciosa,
  R33) — nunca aborta; `strict` promove a erro.

**Config (`config/schema/keys.rs`).**
```rust
KeySpec { key: "recall.semantic", kind: Kind::Bool, default: Default::Bool(true) },
KeySpec { key: "recall.semantic_top_k", kind: Kind::Int, default: Default::Int(50) },
```

**Determinismo.** Ordenação `(similaridade desc, id asc)`; `EmbeddingIndex::ids()` já é ordenado.

**Erros.** Provedor indisponível ⇒ warning, não `Error` (salvo `strict`). Índice ausente ⇒ canal
`None`.

**Testes.**
- Unidade: `rank_query` com vetores sintéticos (id asc no empate).
- Proptest: canal ausente não altera a saída lexical.
- Integração: com `FakeEmbedder` + `MemFs`, query PT casa a nota esperada.
- Golden: `ask --json` inclui `why` sem novo valor (o `Why` continua fechado).

**Aceite.** `kd ask "fuso horário"` retorna `fact_*` de UTC quando há vetores; sem vetores, o
resultado é idêntico ao de hoje.

**Dependências.** Nenhuma. **Risco.** `embed()` bloqueante na borda é aceitável (HTTP local,
timeout); não introduz async (R16/R43).

---

### K1 — `kd write --outcome` em qualquer nota (D103)

**Motivação.** `outcomes[]` só é gravado por `kd task close` (`task/lifecycle.rs:88`, que faz
`ensure_task`). Uma convenção nunca recebe feedback, então a confiança derivada (D87) e o boost
BM25 (E06-T02) ficam inertes para conhecimento.

**CLI.**
```
kd write --outcome <success|partial|failure|abandoned> <ID> [--note <TXT>]
```
- `--outcome` entra em `WriteArgs`; exige exatamente um id posicional (`STATEMENT`).
- Mutuamente exclusivo com `--update`/`--link`/`--dry-run` (erro `invalid_input` se combinados).

**Core.**
- Mover/generalizar `task::lifecycle::outcome` → `write::outcome` **sem** `ensure_task`:
  ```rust
  pub fn outcome(ctx: &WriteContext<'_>, id: &str, status: OutcomeStatus, note: Option<&str>) -> Result<u32>
  ```
  Mantém `task::lifecycle::outcome` como reexport (compat de assinatura) ou delega.
- `OutcomeStatus` passa a viver em `write/` e é reexportado por `task/` (evita ciclo se `task`
  depender de `write` — já depende).

**Contrato.** `outcomes` é chave canônica existente; nada muda no TOON. O evento ganha
`op="outcome"` (já é o rótulo usado).

**Determinismo.** Append puro; `recorded_at` vem de `ctx.now_ms()` (fakes nos testes).

**Erros.** Nota ausente ⇒ `not_found` (3). Status inválido ⇒ `invalid_input` (2). Não-tarefa é
**válido** agora (era `schema`).

**Testes.**
- Unidade: `outcome` em `fact` incrementa `revision` e anexa entrada.
- Golden: reranking — a mesma `ask` muda de ordem após `--outcome success` (usa
  `confidence_score`).
- Integração CLI: `kd write --outcome success <id> --json` → `{action:"outcome", revision}`.

**Aceite.** Uma nota confirmada sobe no `ask`/`rewind` sem alterar a confiança armazenada.

---

### T1 — `kd task list --ready/--blocked` (D104)

**Motivação.** `retrieval/views.rs` computa `ready`/`blocked` e o manifest só mostra contagens;
`task list` filtra apenas scope/status/parent (`commands/task/query.rs`).

**CLI.** Novos modos de `task list` (sem verbo novo):
```
kd task list --ready [--sort id|created|impact] [--limit N]
kd task list --blocked [--explain]
```
- `--ready` e `--blocked` mutuamente exclusivos.
- `--explain` (só com `--blocked`): acrescenta `|blocked_by=<id>` ou `|not_before=<ts>`.

**Core.**
- `retrieval/views.rs`: já expõe `compute_views_at`. Adicionar helper de motivo:
  ```rust
  pub enum BlockReason { Dependency(String), Scheduled, Cycle }
  pub fn block_reason(graph: &Graph, id: &str, now_ms: i64) -> Option<BlockReason>
  ```
- `task/hierarchy.rs`: reusar `children` para o motivo de dependência (menor id pendente).

**Borda (`commands/task/query.rs`).** `list` recebe `view: Option<View>`; monta o `BTreeSet` de
ids permitidos e filtra antes de formatar. Pipe inalterado: `id|scope|status|statement`.

**Determinismo.** `BTreeSet` + ordenação `(id asc)`; `--sort impact` (§T2) com tie-break definido.

**Erros.** `--explain` sem `--blocked` ⇒ `invalid_input`.

**Testes.** Golden do pipe `--ready`/`--blocked`; caso `not_before` futuro ⇒ blocked; ciclo ⇒
blocked; `--explain` determinístico.

**Aceite.** `kd task list --ready` devolve só tarefas desbloqueadas; `--blocked --explain` mostra
o porquê.

---

### PR0.5 — Programas externos (`/plan/*.md`) como raiz (D119)

**Motivação.** O usuário quer o **Programa** (ponto central de uma grande feature) como **arquivo
real** em `/plan/*.md` — git-tracked, legível, autoral — e **não** como `scope`. O knudge cuida da
corrente menor (Épico → User Story → Task). O `scope=plan`/Iniciativa sai de uso.

**O elo (decisão central).** O vínculo entre o arquivo e o grafo é o primitivo **`anchors`**
(D86): já existe, já carrega `content_hash` derivado + **verify-on-hit**, e já alimenta
`rewind --files`. Não é um campo novo nem um scope novo — é o elo mais barato possível.

- **Programa** = `plan/<slug>.md` (markdown livre; o "porquê").
- **Épico-raiz** = `scope=epic`, sem pai, `--anchors plan/<slug>.md` (o `--source` é opcional —
  hoje `kd task new` **não** o aceita; ver core abaixo).
- **`scope=plan`**: **deprecado** (mantido no enum por compat de bytes D95/D93; apenas não usado).
  O papel exibido "Programa" vem do Épico-raiz ancorado (task_universe C3).

**Validado no `knudge-ts`.** `kd task new --scope epic --anchors plan/retry-resilience.md` +
`kd rewind --files plan/retry-resilience.md` já devolve os nós ancorados ao programa. O achado
(a) permanece: `kd task new` não tem `--source`. O achado (b) foi **corrigido**: `kd ask
--anchor <path>` agora alimenta o canal de âncoras (D81) e funciona sem query textual, então é um
resolvedor válido — junto de `rewind --files` / `task graph --program`.

**CLI.**
```
kd task new "<program statement>" --scope epic --anchors plan/<slug>.md [--source plan/<slug>.md]
kd task graph --program plan/<slug>.md     # resolve âncora → épico-raiz → subárvore
kd rewind --files plan/<slug>.md           # já funciona; passa a incluir a subárvore
kd maintenance doctor                      # check program-anchor
```

**Core.**
- `task/program.rs` (novo):
  ```rust
  pub fn root_for_path(index: &Index, graph: &Graph, path: &str, glob: &str) -> Option<String>;
  pub fn program_of(graph: &Graph, epic: &str, glob: &str) -> Option<String>;
  pub fn subtree(graph: &Graph, root: &str) -> Vec<String>;
  ```
  `root_for_path` devolve o **menor id** entre containers-raiz cujo anchor casa o glob;
  `program_of` acha o path do programa (anchor que casa o glob; senão `source`).
- `task/spec.rs` + `cli/task.rs`: acrescentar `source: Option<String>` a `TaskSpec` e `--source` a
  `TaskNewArgs` (hoje ausentes). Sem chave nova (`source` já é canônica).
- `handoff/`/`retrieval/`: `rewind --files <programa>` resolve o Épico-raiz e inclui a subárvore
  (hoje `in_mode(Files)` casa só os nós ancorados, não os filhos não ancorados).
- `health/`: check **`program-anchor`** (warn, não erro): (a) Épico-raiz sem programa; (b) programa
  órfão (`plan/*.md` sem Épico-raiz); (c) `content_hash` divergente (D86) ⇒ "programa mudou;
  re-sincronize o corpo".

**Config.** `programs.glob` (Text, default `"plan/*.md"`) define o que é um programa.

**Elo documental.** `plan/README.md` (opcional) lista os programas → Épico-raiz → status, mantido
por `kd task graph --program` / `doctor`. O arquivo continua sendo a verdade do "porquê"; o knudge
guarda o "como".

**Contrato.** Nenhuma chave TOON nova (`anchors`/`source` já são canônicas). **D119.**

**Determinismo.** Menor id no match; DFS com tie-break `(blocks, created_at, id)`.

**Erros.** `--program` sem arquivo ⇒ `not_found` (3). Arquivo sem Épico-raiz ⇒ resultado parcial +
warning (R33).

**Testes.** `root_for_path` com 2 épicos ancorados ao mesmo doc (menor id); `subtree` DFS
determinística; doctor warn para órfão e para épico sem programa; alterar o doc muda o
`content_hash` ⇒ doctor sinaliza.

**Aceite.** `kd task graph --program plan/foo.md` imprime a árvore inteira do programa; `doctor`
aponta órfãos e drift; **nenhum `scope=plan` é necessário**.

**Interação com task_universe (C1–C4).** O papel "Programa" (C3) é exibido no Épico-raiz ancorado;
`task graph` (C4) mostra o path do programa como raiz da árvore; o `--kind` (C1) e o dono (C2)
seguem valendo para os filhos.

---

## 2. Fase 1 — paridade com seeds/mulch + diferencial

### T3 — `kd task plan --prompt/--from` com templates (D105)

**Motivação.** O seeds materializa um plano preenchido por LLM (`plan prompt` → `plan submit`).
O knudge só aceita `--step` por flag; falta o envelope preenchível e o template.

**CLI.**
```
kd task plan <id> --prompt [--template feature|bug|refactor]
kd task plan <id> --submit --from -      # lê o plano preenchido (TOON) do stdin
```
- `--prompt` é **read-only** (derivado). `--submit --from -` substitui o loop de `--step` quando
  presente; `--step` continua funcionando.

**Formato do prompt (TOON, derivado).**
```
template: feature
seed: <id>
sections: context,approach,steps,acceptance
required: context,approach,steps,acceptance
min_steps: 2
min_acceptance: 1
```
**Formato do plano submetido (TOON).**
```
template: feature
name: <texto opcional>
sections:
  context: <texto>
  approach: <texto>
  steps: <lista de {title,type,priority,blocks,labels,existing_seed,plan_template}>
  acceptance: <lista>
```

**Templates.** `.knudge/templates.toml` no **subset TOML próprio (D97)** — coerente com
`validators.toml` (D99). Built-ins embutidos (`feature`/`bug`/`refactor`) carregados do binário;
o arquivo do projeto **sobrepõe**. Estrutura:
```toml
[templates.feature]
sections = ["context", "approach", "steps", "acceptance"]
required = ["context", "approach", "steps", "acceptance"]
min_steps = 2
min_acceptance = 1
```

**Core.**
- `task/plan.rs` (novo): `prompt(ctx, id, template) -> PlanPrompt` e
  `submit_plan(ctx, id, PlanSpec) -> Vec<SubmitOutcome>`.
- `submit_plan` **valida tudo antes de escrever** (atomicidade lógica): hierarquia, `blocks`
  1-based sem self-ref, ciclo (D53), ids colidindo. Só então chama `task::submit` por filho.
- Reusa `task::membership` para o marcador de pai e `graph::link` para `depends_on`.
- `templates` (novo em `task/` ou `config/`): parse do subset TOML + merge builtin/projeto.

**Contrato.** Nada em `notas/` muda; o prompt é derivado. Registra `D105` e um golden do TOON.

**Determinismo.** Ordem dos filhos = ordem dos `steps`; `blocks` explícito.

**Erros.** Plano inválido ⇒ `schema`/`invalid_input` e **nenhuma** nota criada. Template
desconhecido ⇒ `invalid_input`. `--prompt` com id não-container ⇒ `schema`.

**Testes.**
- Golden do TOON do prompt (builtin + template do projeto).
- Plano inválido não escreve (assert no store vazio).
- `blocks` com ciclo rejeitado.
- Submissão cria N filhos com pai/arestas corretos.

**Aceite.** `prompt → preencher → submit` cria a árvore; reenviar o mesmo plano colide por id
(idempotência de conteúdo).

---

### T4 + K6 — `rewind` com `next` e frescor (D106)

**Motivação.** O manifest mostra `ready=3 blocked=1` mas não **quais**; e não há sinal de frescor
(equivalente ao `ml status`).

**CLI.** Sem flags novas; o manifest ganha 2 linhas dentro do orçamento:
```
notes=11 ready=3 blocked=1 containers=2 dirty
next: task_00ej81yt|Confiabilidade de retry task_01dja2yi|Persistir fila em JSONL
fresh: stale=2 expiring=1 pending=0
```
- `next:` = até K tarefas `ready` ordenadas por impacto (§T2) — `K` derivado do orçamento.
- `fresh:` = `stale` (fora de shelf-life), `expiring` (vence em ≤ grace), `pending` (fila de
  embeddings). Reusa `lifecycle::shelf_life` + `lifecycle::retire` + `embeddings::pending`.

**Core.**
- `handoff/manifest.rs`: `manifest_text` recebe `now_ms` e `&[Note]`/índice; adiciona as linhas.
  Manter a função atual (estática, D57) para o `prime` e criar `manifest_text_at(...)` para o
  `rewind` dinâmico.
- `lifecycle/shelf_life.rs`: helper `freshness(notes, now_ms, policy) -> Freshness`.

**Contrato.** O manifest **não** é o `prime`; o `prime` continua byte-idêntico (D57). Golden do
manifest do `rewind`.

**Determinismo.** `next` por `(impact desc, created asc, id asc)`; `fresh` por contagem.

**Erros.** Nenhum novo; tudo derivado.

**Testes.** Golden do manifest; `--budget` pequeno trunca `next` com `dropped` explícito;
`fresh` reflete `not_before`/expirados.

**Aceite.** Uma sessão retomada vê a próxima tarefa e o estado de frescor sem `task list`.

---

### K3 — `kd ask --tags` (D107)

**Motivação.** Não há como descobrir o vocabulário de tags (seeds tem `label list-all`).

**CLI.**
```
kd ask --tags [--limit N]        # tag|count  (count desc, tag asc)
```

**Core.** `retrieval/mod.rs` (ou `index.rs`): `pub fn tag_counts(index: &Index) -> Vec<(String, usize)>`
usando `BTreeMap` (determinístico). Ignora notas `forgotten`/`superseded`.

**Borda.** `AskArgs` ganha `--tags`; `ask::run` trata antes de `recall`.

**Determinismo.** `(count desc, tag asc)`.

**Erros.** Nenhum.

**Testes.** Golden `tag|count`; `--json` devolve `tags: [{tag,count}]`.

**Aceite.** O agente descobre tags antes de `write --tag`.

**Bônus (opcional).** `rewind` inclui as 5 tags mais frequentes no manifest (≤10 tokens).

---

### X1 — feedback tarefa → conhecimento (D108)

**Motivação.** Nem seeds nem mulch conectam tarefas a conhecimento. O knudge tem o grafo
(`results_in`/`depends_on` + `anchors` + `outcomes`) para derivar isso sem escrever.

**Regra (derivada, tempo de consulta — D87).**
```
boost(nota) = Σ over tarefas t com outcome success
              se share_anchor(t, nota): + W / (1 + distância no grafo)
```
- `share_anchor` = interseção de `anchors` (glob) entre nota e tarefa.
- `W` = novo `recall.confirmation_from_tasks` (float, default 0.1) — **config**, não chave.
- Só `success` conta; `failure` **não** pune (evita overfit).

**Core.**
- `lifecycle/confidence.rs`: `ConfidenceInput` ganha `task_confirmation: f64`; `confidence_score`
  soma `FEEDBACK_WEIGHT * task_confirmation` (ou peso próprio).
- `retrieval/mod.rs`: no cálculo do hit, computar `task_confirmation` a partir do grafo/índice
  (função pura `confidence::from_tasks(doc, graph, index)`).
- `handoff/manifest.rs`: `trust_score` já dá `star`; notas promovidas por tarefa passam a
  `star` naturalmente (via `meta.confirmation`? — não; a confirmação por tarefa é **derivada**,
  não armazenada). Para o manifest, adicionar tier `Star` quando `task_confirmation > 0`.

**Determinismo.** Soma ordenada por id; sem float não-determinístico (mesma entrada → mesma
saída).

**Erros.** Nenhum.

**Testes.** Proptest: monotonicidade (mais tarefas `success` ⇒ confiança ≥). Golden: fechar T1
com `--outcome success --anchors src/retry.ts` promove a decisão de jitter no `ask`.

**Aceite.** Implementar uma tarefa melhora o ranking das notas do mesmo arquivo, sem `write`.

---

## 3. Fase 2 — ergonomia e paridade

### T2 + X3 — impacto derivado (D109)

**CLI.**
```
kd task list --ready --sort impact [--explain]
```
**Core.** `task/hierarchy.rs` (ou `task/impact.rs` novo):
```rust
pub fn impact(graph: &Graph, id: &str) -> usize;   // tarefas abertas que dependem de id (transitivo)
```
**Determinismo.** `(impact desc, created_at asc, id asc)`.
**Testes.** Grafo em cadeia A→B→C: `impact(A)=2`, `impact(B)=1`, `impact(C)=0`; proptest de
monotonicidade sob adição de aresta. **Aceite.** A lista pronta é o caminho crítico.

### T5 — filtros e `show` múltiplo (D104)

**CLI.**
```
kd task list [--tag T] [--anchor PATH] [--since TS] [--ready|--blocked]
kd task show <id> [<id2> ...]
```
**Core.** Reusar `retrieval::Filter`; `show` itera e separa com `\n---\n`; `--json` devolve
`tasks: [...]`. **Testes.** Golden do separador; ids ausentes ⇒ `not_found` sem derrubar os
demais (resultado parcial + warning).

### K2 — `kd ask --rank` (D107)

**CLI.** `kd ask --rank [--type T] [--class C] [--limit N]` → `id|statement|confidence|why`.
**Core.** Reusa `confidence_score` e `tag_counts`; ordena `(confidence desc, id asc)`.
**Testes.** Golden; determinismo com empate.

### K4 — `kd write --batch -` (D110)

**CLI.** `kd write --batch - [--dry-run]` lê **JSONL de drafts** (parser JSON já existe p/ eventos).
```
# saída: uma linha por draft
created|fact_xxx
unchanged|decision_yyy
rejected|decision_zzz
```
**Core.** `write::batch(ctx, drafts: &[Draft], thresholds) -> Vec<WriteOutcome>`; item inválido
⇒ `warnings[]` e continua (R33); teto `write.batch_max` (int, default 100) ⇒ `invalid_input`.
**Config.** `write.batch_max` int 100.
**Testes.** Lote com item inválido grava os válidos; `--dry-run` não grava; ordem preservada.

### K6 — ver T4.

### X2 — `learn` com sinal de tarefa (D111)

**Motivação.** O `learn` já detecta write-gap por arquivo tocado; estender ao ciclo de vida.
**Core.** `maintenance/learn.rs`: novo sinal `kind=create_note`, `why="tarefa fechada sem nota"`,
quando `outcome success` + `anchors` sem nota ancorada. Continua read-only (D47).
**Testes.** Golden da proposta; proptest de que nenhuma proposta é emitida com nota existente.

---

## 4. Fase 3 — polimento

### T6 — `kd task close --note` (D104)

`--note <TXT>` entra em `outcomes[].notes` (já suportado por `write::outcome`). Sem core novo.

### K5 — `kd maintenance prune --dry-run` (D112)

**CLI.** `kd maintenance prune [--scope C] [--dry-run]` → `forget|id|motivo`.
**Core.** Reusa `lifecycle::plan::demotion_candidates` + `lifecycle::retire`; emite propostas.
Aplicação só via `kd forget` (D47). **Testes.** Golden; membros de ciclo nunca propostos (D45).

---

## 5. Decisões a registrar (`plan/03_decisoes-fechadas.md`)

| Dxx | Conteúdo |
|---|---|
| D102 | Canal vetorial entra no `ask` via `recall.semantic`; `rank_query`; degradação graciosa |
| D103 | `outcomes[]` vale para qualquer nota; `write --outcome`; confirmação derivada |
| D104 | Views `ready`/`blocked` e filtros como **modos** de `task list`; `show` multi-id; `close --note` |
| D105 | `task plan --prompt/--from` + `templates.toml` (subset TOML); submissão atômica |
| D106 | `rewind` emite `next:` e `fresh:`; `prime` permanece estático (D57) |
| D107 | `ask --tags` e `ask --rank` (ranking sem query) |
| D108 | Feedback derivado tarefa→conhecimento; `recall.confirmation_from_tasks` |
| D109 | Impacto de desbloqueio derivado como `--sort`; sem chave de prioridade |
| D110 | `write --batch` (JSONL de drafts), degradação graciosa, `write.batch_max` |
| D111 | `learn` com sinal de tarefa (write-gap de tarefa→conhecimento) |
| D112 | `maintenance prune` propõe aposentadoria; nunca age |
| D118 | Nomenclatura exibida = role derivado (`Programa/Épico/User Story/Task`); `scope` armazenado inalterado (task_universe §10) |
| D119 | **Programa = arquivo externo `plan/*.md` ancorado ao Épico-raiz**; `scope=plan` deprecado; `kd task graph --program` |

---

## 6. Novas chaves de config

| Chave | Tipo | Default | Item |
|---|---|---|---|
| `recall.semantic` | Bool | `true` | K7 |
| `recall.semantic_top_k` | Int | `50` | K7 |
| `recall.confirmation_from_tasks` | Float | `0.1` | X1 |
| `write.batch_max` | Int | `100` | K4 |
| `programs.glob` | Text | `"plan/*.md"` | PR0.5 |

Adicionar em `crates/knudge-core/src/config/schema/keys.rs` (ordem canônica D63) + defaults no
`config.toml` global. Nenhuma afeta o `prime`.

---

## 7. Contratos, goldens e divergências

- **TOON:** sem chave nova. Atualizar `TOON.md` só se algum modo novo mudar a emissão — não muda.
- **Pipe:** novos formatos (`--tags`, `--rank`, `--blocked --explain`, `next:`, `fresh:`) precisam
  de golden em `crates/*/tests/` e linha em `plan/implementation/17_matriz_aceitacao.md`.
- **`DIVERGENCES.md`:** registrar os tie-breaks novos (impacto, tags, rank, feedback) e o teste
  que os trava.
- **MCP:** espelhar os modos novos em `crates/knudge-mcp/src/tools.rs` (16 §13).
- **`prime`:** inalterado por construção; um teste de golden deve falhar se qualquer PR o tocar.

---

## 8. Estratégia de testes

1. **Unidade** em `src/<mod>/tests.rs` com fakes (`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`,
   `FakeEmbedder`, `RecordingLogger`).
2. **Proptest** para funções puras: `rank_query` (determinismo), `impact` (monotonicidade),
   `confidence_score` (monotonicidade), `tag_counts` (soma = nº de notas).
3. **Golden** para todo pipe novo (bytes exatos) e para o `prime` (não pode mudar).
4. **Integração do binário** (`env!("CARGO_BIN_EXE_kd")`): `--help`, `--json` válido, exit codes,
   EPIPE→0, e cada modo novo.
5. **Sem `unwrap`/`expect`/`panic`** em testes; usar `?`/`matches!`/`ok()`.

---

## 9. Riscos e mitigações

| Risco | Mitigação |
|---|---|
| Canal vetorial adiciona latência ao `ask` | config `recall.semantic=false`; timeout do embedder; fallback BM25 |
| `outcomes[]` em nota crescer sem limite | teto `outcomes` por nota (ex.: 32) + `compact` merge_outcomes (D47) |
| `templates.toml` virar YAML disfarçado | parser = subset TOML próprio (D97); rejeitar `[[...]]`/multilinha |
| Submissão de plano parcial | validar tudo **antes** do primeiro `commit`; teste de "nada escrito em erro" |
| Feedback tarefa→conhecimento virar overfit | só `success`, peso baixo por config, sem punir `failure` |
| `--sort impact` custo O(V+E) por lista | cache no índice derivado (`.idx/`), reconstruível (D15) |
| Mudança acidental no `prime` | golden do `prime` no CI + `DIVERGENCES.md` |

---

## 10. Rollout e compatibilidade

- Tudo é **aditivo**: flags/modos novos; defaults preservam o comportamento atual, exceto
  `recall.semantic=true` (K7), que altera ranking — documentar no `CHANGELOG` e permitir
  desligar por config.
- Sem bump de `schema_version` (nenhuma chave nova; D15).
- `make install` após cada fase; `kd maintenance doctor` deve seguir verde.

---

## 11. Docs e changelog por PR

- `CHANGELOG.md`: uma entrada por item concluído.
- `plan/implementation/16_cli_surface.md` e `17_matriz_aceitacao.md`: novos modos.
- `AGENTS.md` (do projeto-alvo, template do `kd init`) só se o fluxo do agente mudar (T1/T4/K1
  mudam o fluxo: consultar `next`, confirmar notas).
- `MODULE.md` de `task/`, `handoff/`, `retrieval/`, `write/` se um módulo novo for criado
  (`task/plan.rs`, `task/impact.rs`, `task/templates.rs`).

---

## 12. Definition of Done (por item)

- [ ] `make check` verde (fmt + clippy `-D warnings` + test + gate 300 linhas).
- [ ] Teste novo cobrindo o comportamento (unidade/proptest/golden/integração) — sem `unwrap`.
- [ ] Pipe novo com golden; `prime` intacto (golden).
- [ ] Linha em `DIVERGENCES.md` se houver borda nova (tie-break/ordem).
- [ ] `Dxx` registrado e propagado; `17_matriz_aceitacao.md` atualizada.
- [ ] `CHANGELOG.md` atualizado; `MODULE.md` se criou módulo.
- [ ] Nenhuma chave TOON nova; nenhum arquivo de `src/` > 300 linhas.
