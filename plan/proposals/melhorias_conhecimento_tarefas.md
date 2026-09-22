# Propostas — conhecimento e tarefas no knudge

> Documento de **proposta** (não é decisão fechada). Origem: avaliação comparativa
> `knudge × seeds × mulch` feita no subprojeto `knudge-ts`
> (`knudge-ts/eval/REPORT.md`). Cada item respeita as restrições do projeto: enum fechado,
> 28 chaves canônicas (D95/D98/D100), TOON como contrato (D95), determinismo, núcleo puro via
> portas, derivação em vez de armazenamento (D87), poucos verbos (D93), sem infra nova
> (`tokio`/`reqwest`/DB — R16/R43) e "propor, nunca agir em silêncio" (D47).
>
> **Filtro central:** copiar a *ergonomia* de seeds/mulch, **não** o *modelo de dados*. O knudge
> recusa `custom_types` (enum aberto) e labels livres justamente porque já fixou o vocabulário;
> a vantagem dele é o **grafo + confiança derivada**, que nenhum dos dois tem.

---

## 0. O que a avaliação mediu (evidência)

| Observação | Evidência |
|---|---|
| `ready`/`blocked` são computados no core (`retrieval/views.rs`) e o `rewind` mostra só as **contagens** (`ready=3 blocked=1`) | `handoff/manifest.rs:121-144` |
| `kd task list` filtra apenas `--scope/--status/--parent`; não há `--ready/--blocked`, `--tag`, `--anchor`, `--sort` | `commands/task/query.rs` |
| `outcomes[]` só é gravado por `kd task close`; não há como **confirmar/refutar uma nota** de conhecimento | `task/lifecycle.rs:88`; `write_cmd.rs` sem `--outcome` |
| Sem descoberta do vocabulário de tags (seeds tem `sd label list-all`; mulch filtra por tag) | `AskArgs` só tem `--tag` de filtro |
| Escrita é 1 nota por invocação; mulch tem `--batch`/`--stdin` | `write_cmd.rs` |
| Embeddings são indexados mas o `ask` **não usa** o canal vetorial | `commands/ask.rs` não popula `query.vector` |

---

## 1. Tarefas (o que o seeds faz melhor)

### T1. Expor `ready`/`blocked` como modos de `kd task list` — **P0**

O core já calcula as views; falta a superfície. Sem verbo novo (D93), são modos:

```
kd task list --ready
kd task list --blocked --explain      # por que está bloqueada (dependência pendente ou not_before)
```

- **Saída** (pipe, 1 linha/tarefa): `id|scope|status|statement` — idêntica a `list` hoje, então o
  contrato não muda.
- **Core:** `list` recebe um filtro `View ∈ {all, ready, blocked}` e usa
  `compute_views_at(graph, now)` (dinâmico, considera `not_before` — D56/D100).
- **`--explain`:** anexa `|blocked_by=<id>` ou `|not_before=<ts>`; determinístico (menor id).
- **Testes:** golden do pipe; caso `not_before` futuro → `blocked`; caso ciclo → `blocked`.

### T2. Prioridade **derivada** por impacto de desbloqueio — **P0**

O seeds armazena `priority 0-4`; o knudge não tem a chave (e adicioná-la seria uma 29ª chave —
custo alto). Em vez disso, **derivar** (D87): quanto uma tarefa desbloqueia.

```
kd task list --ready --sort impact
```

- `impact(t) = |{ t' | t ∈ depends_on⁺(t') e status(t') ∉ {closed, superseded} }|`
  (nº de tarefas abertas que só esperam por `t`). Tie-break `(impact desc, created_at asc, id asc)`.
- **Core:** `task::impact(graph, id)` puro; `list` ganha `--sort {id|created|impact}`.
- **Por que é melhor que prioridade estática:** não exige disciplina do agente nem nova chave;
  reflete o grafo real. Casa com a tese "o índice é derivado".
- **`--explain`:** `...|unblocks=3`.

### T3. `kd task plan` com *prompt* preenchível e template — **P1**

O seeds ganha muito com `sd plan prompt` → `sd plan submit`: o LLM preenche um JSON estruturado e
o sistema materializa os filhos. O knudge já tem o ciclo (`--step/--submit/--adopt/...`), mas só
por flags; falta o **envelope de prompt** e o **template**.

```
kd task plan <id> --prompt              # emite a requisição (TOON) para o LLM preencher
kd task plan <id> --submit --from -     # lê o plano preenchido (TOON) do stdin
```

- **Prompt (derivado, nada armazenado):** `seed|template|sections[]|validation` em TOON — o
  formato já é o contrato do projeto. Built-ins `feature`, `bug`, `refactor` (espelha seeds).
- **Templates:** `.knudge/templates.toml` no **subset TOML próprio (D97)** — coerente com
  `validators.toml` (D99). O `00_panorama.md` cita `templates.yaml`; a decisão D99 já rejeitou
  YAML, então aqui corrigimos para TOML.
- **Submissão atômica:** valida o plano inteiro **antes** de qualquer `commit` (mesma disciplina do
  `plan edit` do seeds: erro ⇒ nada escrito). Cria N filhos + arestas `depends_on` a partir de
  `blocks` (1-based, sem self-reference, detecção de ciclo — D53) + `not_before`.
- **Idempotência de graça:** ids são endereçados por conteúdo (D95); reenviar o mesmo plano
  colide no id e vira `unchanged`/erro de conflito previsível.
- **Core:** `task::plan::prompt(ctx, id, template)` e `task::plan::submit(ctx, PlanSpec)`;
  nada de I/O no core.
- **Testes:** golden do TOON do prompt; plano inválido não escreve; ciclo em `blocks` rejeitado.

### T4. `rewind` mostra o *ready set*, não só a contagem — **P1**

Hoje o manifest diz `ready=3 blocked=1`, mas quem retoma a sessão não vê **quais** são. O gap de
handoff é justamente "o que fazer agora".

```
kd rewind                       # manifest ganha 1 linha, dentro do orçamento
# notes=11 ready=3 blocked=1 containers=2 dirty
# next: task_00ej81yt|Confiabilidade de retry task_01dja2yi|Persistir fila em JSONL
```

- Respeita o orçamento (`ceil(len/4)`, D40): trunca por `--budget`, com `dropped` explícito.
- A view é **dinâmica** (`compute_views_at`), então `not_before` conta; o `prime` continua
  estático/byte-idêntico (D57) porque o manifest não faz parte do `prime`.
- **Core:** `handoff/manifest.rs` reusa `compute_views`; sem chave nova.

### T5. Paridade de filtros e `show` múltiplo — **P2**

```
kd task list --tag retry --anchor src/retry.ts --since <ts>
kd task show <id> [<id2> ...]        # espelha sd show multi-id
```

- Reusa `Filter` do retrieval (já existe). `show` com N ids: separador `\n---\n`; `--json`
  devolve `tasks: [...]` (mesma convenção do seeds).

### T6. `kd task close --outcome` já roda validators; falta o **resumo de evidência** — **P3**

O seeds `close --reason` grava um motivo. O knudge grava `outcomes[]` mas o texto é opcional.
Sugerir `--note <TXT>` (entra em `outcomes[].notes`) para registrar *por que* fechou — alimenta o
`learn` e o `diff`.

---

## 2. Conhecimento (o que o mulch faz melhor)

### K1. Confirmar/refutar uma nota (outcomes fora de tarefas) — **P0, maior ganho**

O mulch fecha o loop com `ml outcome <domain> <id> --status`, e isso **ranqueia** (boost por
confirmação). No knudge, `outcomes[]` só existe para tarefas; uma convenção/erro nunca recebe
feedback. Generalizar o mecanismo é barato e liga a confiança derivada (D48/D87) que já está no
schema.

```
kd write --outcome success <ID> [--note "aplicado no job de retry"]
kd write --outcome failure <ID> --note "quebrou com relogio mock"
```

- **Core:** generalizar `task::lifecycle::outcome` para `write::outcome` (aceita qualquer nota);
  reusa `OutcomeStatus` (success/partial/failure/abandoned).
- **Efeito:** o boost BM25 `score * (1 + 0.1*(success + partial*0.5))` (E06-T02) e a confiança
  derivada em tempo de consulta (D87) passam a evoluir **sem armazenar** confiança. É a mesma
  filosofia do mulch (score derivado), mas sobre o grafo do knudge.
- **Testes:** golden de reranking — a mesma query muda a ordem após `--outcome success`;
  `outcomes[]` é lista no frontmatter (ordem canônica preservada).

### K2. `kd ask --rank` — ranking sem query — **P2**

Espelha `ml rank`: consumidores com contexto curto querem "as notas mais confirmadas" sem
pergunta.

```
kd ask --rank [--type T] [--limit N]     # id|statement|confidence|why
```

- **Core:** reusa o scorer de confiança derivada; ordena `(confidence desc, id asc)`.
- **Custo:** ~15 tok/hit, igual ao pipe do recall.

### K3. Descoberta de vocabulário (tags) — **P1**

Agentes não sabem quais tags existem; hoje só dá para filtrar por uma tag adivinhada. O seeds tem
`sd label list-all`; o mulch filtra por tag.

```
kd ask --tags                 # tag|count, ordenado (count desc, tag asc)
```

- **Core:** agregação determinística sobre o índice (BTreeMap).
- **Bônus barato:** as 5 tags mais frequentes podem entrar no manifest do `rewind` (≈10 tokens),
  para o agente saber o vocabulário antes de `write --tag`.

### K4. Escrita em lote: `kd write --batch -` — **P2**

Fim de sessão: o agente tem 5 aprendizados e faz 5 round-trips. O mulch resolve com
`ml record --batch`/`--stdin`. O knudge pode aceitar **JSONL de drafts** (o parser JSON já existe
para eventos — sem parser novo):

```
kd write --batch - < drafts.jsonl
# action|id  (uma linha por draft; dedup avaliado por item)
```

- **Core:** `write::batch(ctx, drafts)` aplica a máquina de dedup (0.75/0.92) por item; item
  inválido é reportado e **não** derruba o lote (degradação graciosa, R33 — resultado parcial +
  `warnings[]`).
- **`--dry-run`** mostra os vereditos sem gravar (espelha `ml record --batch --dry-run`).

### K5. `kd maintenance prune --dry-run` — propor aposentadoria — **P3**

O knudge tem `retention`/`decay` e `forget`, mas `forget` é manual. O mulch tem
`ml prune --dry-run` que propõe arquivar por shelf-life. Alinhado a D47 (propor, nunca agir):

```
kd maintenance prune [--scope C] [--dry-run]
# forget|id|motivo   (expirado por retention, âncoras decaídas, cluster obsoleto)
```

- **Core:** reusa lifecycle (shelf-life/decay/clusters); emite **propostas**, o agente aplica com
  `kd forget`.

### K6. Linha de frescor no manifest — **P2**

Espelha `ml status` (frescor por domínio). No knudge cabe em ~12 tokens no manifest do `rewind`:

```
# stale=2 expiring=1 pending=0
```

- Derivado de `retention`/`decay`/fila de embeddings (o manifest já mostra `embeddings_pending`).

### K7. Ligar o canal vetorial ao `ask` — **P0 (correção, não feature)**

A avaliação mostrou que `ask` nunca popula `query.vector` (`commands/ask.rs`), apesar de o índice
vetorial existir (E11) e o RRF já suportar o canal (`retrieval/mod.rs:127`). É o maior ganho de
consulta pelo menor esforço — `"fuso horário"` passa a achar a convenção de UTC.

- **Core:** nada novo (o canal já existe); a borda computa o vetor da query via `Embedder` e o
  injeta no `RecallQuery`. Degrada gracioso se o embedder falhar (R33): cai para BM25 + `warn`.
- **Teste:** proptest de que o canal ausente não altera o resultado lexical; golden de que o canal
  presente reordena quando o vetor aproxima.

---

## 3. Alavancas exclusivas do knudge (nem seeds nem mulch têm)

### X1. Feedback tarefa → conhecimento (fusão dos dois mundos) — **P1, criativo**

Quando uma tarefa fecha com sucesso, as notas ancoradas nos mesmos arquivos ganham **confirmação
derivada** — sem escrever em `outcomes[]`:

```
confiança_derivada(nota) += f(outcomes de tarefas que compartilham âncoras)
```

- Determinístico, em tempo de consulta (D87), sem chave nova.
- Efeito prático: implementar o backoff (tarefa fechada, `--anchors src/retry.ts`) promove
  automaticamente a decisão de full jitter e a falha de off-by-one nas próximas consultas.
- **Core:** `confidence::derive(note, graph, outcomes)`; entra no boost do BM25 e no `--rank`.
- É a resposta direta à pergunta "melhor resultado": o knudge **conecta** o que seeds (tarefas) e
  mulch (conhecimento) mantêm separados.

### X2. `learn` com sinal de tarefa — **P2**

O `learn` já detecta "atividade sem registro" (arquivos tocados sem nota). Estender ao ciclo de
vida: **tarefa fechada com sucesso + âncora sem nota** ⇒ proposta `create_note` (write-gap de
tarefa→conhecimento). Continua read-only (D47).

### X3. `kd task list --ready --sort impact` como "caminho crítico" — **P2**

Combina T1+T2: a lista pronta ordenada por impacto é, na prática, o caminho crítico do plano.
`--explain` mostra `unblocks=N`. Substitui a noção de prioridade estática do seeds por uma
**derivada do grafo**.

---

## 4. Priorização (menor caminho → maior impacto)

| Ordem | Item | Por quê | Custo |
|---|---|---|---|
| **P0** | K7 (canal vetorial no `ask`) | correção de bug; maior ganho de consulta | baixo (borda) |
| **P0** | K1 (`--outcome` em notas) | fecha o loop; liga confiança derivada | baixo |
| **P0** | T1 (`--ready/--blocked`) | views já prontas no core; falta expor | baixo |
| **P1** | T3 (plan prompt/template) | paridade com o melhor do seeds | médio |
| **P1** | T4 (`next` no rewind) | fecha handoff | baixo |
| **P1** | K3 (`--tags`) | descoberta de vocabulário | baixo |
| **P1** | X1 (feedback tarefa→conhecimento) | diferencial único | médio |
| **P2** | T2/T5, K2/K4/K6, X2/X3 | ergonomia | médio |
| **P3** | T6, K5 | polimento | baixo |

**Sequência sugerida:** P0 (3 itens, quase todos na borda) → T3+T4+K3 → X1. Isso já cobre os
gaps observados e entrega a vantagem que os concorrentes não têm.

---

## 5. O que **não** copiar (e por quê)

| Do seeds/mulch | Recusa | Motivo |
|---|---|---|
| `custom_types` (mulch) | ❌ | enum aberto; viola "não criar tipos abertos onde o projeto fixou enum fechado" |
| labels livres (seeds) | ❌ | knudge tem `tags` filtradas; vocabulário fechado é feature, não limite |
| domínio = arquivo (mulch) | ❌ | knudge já usa `container`/`anchors`; arquivo-por-domínio fragmenta o grafo |
| `--force` para duplicar (mulch) | ❌ | a **idempotência** do knudge (`unchanged`) é vantagem; não abrir exceção |
| `priority 0-4` armazenada (seeds) | ⚠️ | prefira **impacto derivado** (T2/X3); chave nova custa contrato |
| `plans.jsonl` separado (seeds) | ❌ | no knudge o plano é view derivada (D52); não materializar verdade |

---

## 6. Contratos a atualizar se as propostas avançarem

- `plan/implementation/16_cli_surface.md`: novos modos de `task list`/`ask`/`write`.
- `plan/implementation/17_matriz_aceitacao.md`: linhas de aceite por modo.
- `plan/03_decisoes-fechadas.md`: um `Dxx` por mudança de contrato (T1/T3/K1/X1 no mínimo).
- `TOON.md`: só se algum campo novo for canônico — **nenhuma proposta aqui adiciona chave**
  (todas derivam), então o contrato de bytes fica intacto.
- `DIVERGENCES.md`: ordem/tie-break dos novos rankings (impacto, tags, rank).
