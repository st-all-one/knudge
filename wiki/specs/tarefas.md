# Tarefas, planos e fluxo

Como o knudge representa **trabalho** — o modelo, por que ele é assim, como funciona e quais
decisões cabem dentro dele. Em uma frase: **tarefa é uma view derivada de notas**, não uma base de
dados paralela.

- Código: `crates/knudge-core/src/task/`
- Decisões: D52/D53, D93/D104/D105, D113–D116, D119/D120, D125–D127, D134–D141, D149, D205
- Fórmulas: [`matematica.md`](matematica.md) §10 (PERT/CPM).
- Bordas: [`DIVERGENCES.md`](DIVERGENCES.md) #37/#41/#108

## 1. O que é o modelo de tarefas

Toda tarefa, épico ou issue é uma **nota comum** em `notas/<tipo>/<id>.md` — o mesmo formato, o
mesmo contrato de bytes, o mesmo store. O que a distingue é o campo fechado **`scope`**:

- `scope = epic` → um **grupo** (nó de planejamento). Não tem `type` (o tipo efetivo `epic` é
  derivado — D149).
- `scope = issue` / `scope = task` → um **item de trabalho** (`type = task` ou uma espécie de
  trabalho — D113).

Não existe um "banco de tarefas". `kd task` e `kd write` operam sobre as mesmas notas; o que muda
é **por qual porta** (o `write` rejeita `task`/`epic` — D93).

### Por que é assim (racional)

| Motivo | Consequência |
|---|---|
| **Uma fonte da verdade.** A nota é o dado; qualquer projeção (árvore, progresso, modo) é recomputada. | nada duplica estado; `sync`/git funcionam igual para conhecimento e tarefa (D32/D153). |
| **Legível por humano e LLM sem ferramenta.** O pai vive no **corpo** (marcador), não só no grafo. | um agente que lê o arquivo entende a hierarquia; o grafo é conveniência (D93). |
| **Sem chaves novas para semântica de gestão.** Papel, modo, progresso, impacto e fluxo são **derivados**. | evita "campos de status que mentem"; nada para migrar (D115/D116/D127). |
| **Determinismo.** Toda derivação é função pura da árvore/grafo/log. | a mesma árvore dá o mesmo papel/modo/progresso; testável com fakes (D92). |
| **Fronteira clara com conhecimento.** `ask` responde *o que se sabe*; `task` responde *o que fazer*. | `ask` filtra só conhecimento por padrão (D146). |

### As duas camadas da hierarquia

1. **Relação durável** — o **marcador de pai** no corpo (D52/D93):
   `<!-- knudge:parent <id> blocks <n> -->`. É texto, sobrevive a qualquer rebuild, é lido sem o
   grafo.
2. **Projeção derivada** — a aresta `results_in` do pai para o filho, recriada a partir do
   marcador quando o grafo é construído (D49). `expand`, `children`, `progress` e `context` leem
   a aresta; o marcador é a verdade.

## 2. Ontologia: `scope`, `type` e `kind`

- **`scope` = nível** (onde o item vive na árvore): `epic`, `issue`, `task` (D134).
- **`type` = espécie** (que natureza tem): `task` por default; `error`/`question`/`risk`/`decision`
  para itens de trabalho "não triviais" (D113). O conjunto válido é `WORK_KINDS` (5).

| `scope` | `type` default | `--kind` aceita | papel (depth 0/1/≥2) |
|---|---|---|---|
| `epic` | *(omitido → epic)* | — | **Epic** |
| `issue` | `task` | `task`/`error`/`question`/`risk`/`decision` | Feature (com filhos) / Story (folha) |
| `task` | `task` | idem | Sub-task |

`validate_kind(scope, kind)` recusa `--kind` em `epic` e aceita só `WORK_KINDS` em `issue`/`task`.

## 3. Hierarquia: regras e validação

A hierarquia é `epic ⊃ { issue ⊃ task | task }` — mas o **`issue` é opcional** (D134):

- `epic` é a **raiz** e **não tem pai** (`validate_parent` recusa filho `epic`).
- O pai pode ser **qualquer ancestral de rank estritamente menor** (`epic < issue < task`), então
  `epic → task` direto é válido.
- **O nível não é intrínseco ao `scope`** — é a **profundidade na árvore**, o que permite pular o
  `issue`.
- `blocks` é **1-based** e exige `parent` (`validate_blocks`; `blocks = 0` é erro).
- A relação é de **pai único** (um marcador por nota).
- Ancorar o épico a um arquivo `plan/*.md` é **recomendado** — o `doctor` emite *warning*, não erro
  (D119/D134).

Ao criar um filho, `submit` valida o escopo do pai **lendo o pai no store** (não confia só no
argumento) e, depois de gravar o filho, adiciona a aresta `results_in` no pai e incrementa a
`revision` dele (D21).

## 4. O que é derivado (nunca armazenado)

| Derivação | Função | Regra |
|---|---|---|
| **Papel** (`Role`) | `role(depth, kind, has_children)` | espécie manda (`error`→Bug, `question`→Spike, `risk`→Risk, `decision`→Decision); senão profundidade: `0`→Epic, `1`+filhos→Feature, `1`→Story, `≥2`→SubTask (D115). |
| **Modo** (`Mode`) | `mode(container)` | `incremental`→Magentic; senão todos os filhos (menos o 1º) dependem de um irmão anterior→Sequential; senão Concurrent (D116/D136). |
| **Progresso** | `progress_of(graph, root)` | conta **itens de trabalho folha** no subárvore e quantos estão `closed` (D127). |
| **Épico** | `epic_of(graph, id)` | sobe pelos pais (≤3 níveis) até `scope=epic`. |
| **Impacto** | `impact`/`impacts` | nº de itens **abertos** que dependem transitivamente (D106/D109). |
| **Contexto** | `context_of` | pai/bloqueadores/bloqueados/filhos + épico (D125). |
| **Fluxo** | `task_flows`/`throughput`/`critical_path` | derivado do log de eventos (D205). |

**Por que não armazenar:** papel/modo/owner/progresso como campos ficam desatualizados (a árvore
muda) e criam uma segunda verdade. Derivar garante consistência e é barato (o grafo já está em
memória). O preço é recomputar a cada leitura — aceitável porque tudo é O(N) sobre o grafo.

## 5. A superfície de decisão (o que você decide)

Ao criar/atualizar uma tarefa, estas são as decisões **reais** (o resto é derivado):

| Campo / flag | Valores | Efeito |
|---|---|---|
| `scope` | `epic`/`issue`/`task` | nível na árvore; define o `type` default. |
| `--kind` | `WORK_KINDS` | espécie (relaxa D93): `error`/`question`/`risk`/`decision` contam como item de trabalho (D120). |
| `--parent` | id | pai (marcador + `results_in`); validado por rank. |
| `blocks` | `n ≥ 1` | ordem 1-based entre irmãos (exige pai). |
| `--summary` + corpo | texto | afirmação (≤120) + contexto (o **posicional é o corpo**, D140). |
| `--status` | `active`/`in_progress`/`blocked`/`closed` | estado (máquina própria — §6). |
| `classification` | `foundational`/`tactical`/`observational` | maturidade; rege a retenção (D44). |
| `--checks` | nomes de validator | critérios de aceite (D54/D99). |
| `--anchor` | path/glob | vínculo externo (único — D135). |
| `--tag` | texto | agrupamento. |
| arestas (`--link`) | `depends_on`/`contradicts`/… | via única: `kd write --link` (D126). |

> **Arestas têm via única** (D126): `kd task new` **não** aceita `--depends-on`; o vínculo é
> `kd write --link <FROM:ARESTA:TO>`, reaproveitando o grafo (valida id/auto-aresta, grava evento e
> revisão). `plan submit` cria as dependências dos passos por esse caminho.

## 6. Máquina de status

O status de tarefa é validado por `task::lifecycle::validate_transition` (D53/D138):

| de \ para | `active` | `in_progress` | `blocked` | `closed` |
|---|---|---|---|---|
| `active` | ✓ | ✓ | | ✓ |
| `in_progress` | ✓ | ✓ | | ✓ |
| `blocked` | ✓ | ✓ | | |
| `closed` | | | | ✓ |

Regras: transições de status ficam em `kd task update --status`; o **fechamento** é
`kd task close` (com evidência — D55). `superseded`/`forgotten` são atribuídos por
`update`/`forget`, nunca à mão (ver `write/status.rs`). Um item **aberto** = `active`/`in_progress`/
`blocked`; `is_actionable` exclui `closed`/`superseded`/`forgotten` (usado por `--sort impact` e
`next:`).

## 7. Operações

- **`submit`** — cria a nota, valida o pai e anexa (`results_in` + `revision++`). Grava o evento
  `task/submit`. Idempotência é por conteúdo (D01): a mesma afirmação não duplica.
- **`update`** — altera campos; incrementa `revision`; grava evento.
- **`close`/`review`** — aplica a transição para `closed` **com evidência** (roda validators e
  infere o `outcome` — D55).
- **`outcome`** — registra evidência de execução para **qualquer** nota (D103); alimenta a
  confiança derivada (D87/D189) e a retenção (D190).

## 8. Planos e templates (D105/D138)

- **`kd task plan --prompt`** deriva um **prompt** (TOON, read-only) a partir de um template: nome,
  seed, seções esperadas/obrigatórias, `min_steps`, `min_acceptance`.
- **`--submit --from`** materializa o plano: valida **por inteiro antes de qualquer escrita**
  (atomicidade lógica) e cria os passos como folhas `task`, ligando as dependências via `write::link`.
- **Templates** vivem em `.knudge/templates.toml` (subset TOML próprio, D97); os built-ins
  (`feature`/`bug`/`refactor`) vêm do binário e o arquivo do projeto **sobrepõe por nome**.
- Saíram `--adopt`/`--release`/`--review`/`--reorder` (D138): transição é `task update --status`;
  fechamento é `task close`; a ordem vem do próprio plano.

## 9. Lote (D141)

`kd task new --batch FILE|-` cria/atualiza itens a partir de **JSONL** (ou `--params '{...}'` para
um item). Cada linha é uma operação (`TaskOp`):

- `key` local permite referenciar itens entre linhas (`parent`/`depends_on` por `key`).
- `id` presente = **atualiza**; ausente = **cria**.
- Carrega as 7 arestas explícitas (ids ou `key`s) — permite **re-parentar** e criar vínculos no
  mesmo lote.
- Processa **em ordem** (pai antes do filho), **best-effort** com `warnings[]` (R33), `--dry-run`,
  teto `task.batch_max`.
- A saída lista o que foi criado/atualizado (`key|id|scope|status|statement`) e o `--json` traz
  `key`→id e as arestas resolvidas.

## 10. Programas externos (D119/D139)

Um **Programa** é um arquivo `plan/*.md` real (o "porquê", git-tracked); o knudge cuida da corrente
menor (Épico → Story → Task). O elo é a **âncora** (D86) — nenhum `scope` novo, nenhuma chave nova.

- `roots_for_path` resolve **todos** os Épicos-raiz (sem pai) ancorados a um path — `plan.md` pode
  ancorar **vários** épicos (floresta).
- `subtree` devolve a árvore em pré-ordem determinística.
- `kd task graph --program` renderiza a floresta; o check `program-anchor` do `doctor` avisa quando
  falta a âncora (warn — D134).

## 11. Fluxo (D205)

`task/flow.rs` deriva do **log de eventos** (sem gravar verdade):

- **Cycle time** = `review − primeiro evento`; **Lead time** = `close (ou agora) − create`.
- **Throughput** = fechamentos por janela (`div_euclid`).
- **Caminho crítico** (PERT/CPM) = maior caminho ponderado do DAG `depends_on` (peso = lead time);
  empate pelo caminho mais longo, depois menor id. Fórmulas em [`matematica.md`](matematica.md) §10.

Expõe `kd task flow` (texto `resumo|`/`throughput|`/`critico|` + `--json`) e `rewind --json`
(`data.flow`, aditivo).

## 12. Relação com conhecimento

- `kd ask` responde **o que se sabe** e, por padrão, devolve só conhecimento (`scope` ausente);
  itens de trabalho entram só com `--with-task` (D146).
- `kd task list` responde **o que fazer** e exige escopo/filtro (D144).
- A ponte é a **âncora** (D86) e a confirmação **tarefa→conhecimento** (D108): uma tarefa com
  `outcomes` de sucesso que compartilha âncoras **confirma** a nota correspondente — sem `write`.

## 13. Anti-padrões e invariantes

- **Não** criar tarefa por `kd write` — use `kd task` (o `write` rejeita `task`/`epic`).
- **Não** guardar papel, modo, progresso, dono ou prioridade — todos são derivados.
- **Não** usar `scope=plan` — saiu (D134); `doctor --fix` migra para `epic`.
- **Não** criar dependência por flag em `task new` — use `write --link` (D126).
- **Não** fechar sem evidência — `task close` roda validators (D55).
- **Não** misturar `ask` e `task list` — superfícies separadas (D146).

## 14. Onde vive

| Aspecto | Arquivo |
|---|---|
| `submit`, `parent_of`, `is_task`, `attach` | `task/mod.rs` |
| Especificação/espécies (`WORK_KINDS`) | `task/spec.rs` |
| Hierarquia/validação | `task/hierarchy.rs` |
| Filiação (marcador) | `task/membership.rs` |
| Contexto estrutural | `task/context.rs` |
| Progresso/épico | `task/progress.rs` |
| Ciclo de vida/status | `task/lifecycle.rs` |
| Impacto | `task/impact.rs` |
| Papel/modo | `task/{role,mode}.rs` |
| Plano/templates | `task/{plan,template}.rs` |
| Lote | `task/batch/` |
| Programa | `task/program.rs` |
| Fluxo | `task/flow.rs` |

## 15. Testes

`task/tests/` (batch, context, flow, hierarchy, impact, kind, lifecycle, mode, plan, program,
progress, role, submit, template). `DIVERGENCES.md` #37 (impacto vs `--ready`), #41 (item de
trabalho), #108 (flow/caminho crítico).
