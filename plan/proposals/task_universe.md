# task_universe.md — fechando o universo de tasks com elegância

> **Status:** C1–C4 (D113–D116) e **PR0.5** (Programa externo, §10.7/D119) implementados —
> `--kind`, `kd task claim`/`--owner`/`--mine`, `task::program`, `kd task graph [--root]`
> (com papel e modo derivados), `task::template`/`plan` (`--prompt`/`--from`), check
> `program-anchor` e config `programs.glob`. C5/C6 ficaram como **documentação** (checks = DoD
> e ledger = eventos + learn no `prime`) — já refletidas em `prime.rs`.
>
> Síntese dos dois brainstorms (WBS/Jira/SAFe e orquestração multi-agente) sobre o modelo de
> tarefas do knudge. **Proposta de desenho**, não decisão fechada. Continua
> [`melhorias_conhecimento_tarefas.md`](melhorias_conhecimento_tarefas.md) e
> [`upgrade_plan.md`](upgrade_plan.md) (que usam D102–D112; aqui seguimos de D113).
>
> **Tese:** o universo de tasks não se fecha adicionando níveis, e sim **separando eixos**.
> O knudge já tem os primitivos; falta expor o eixo *espécie* e o eixo *dono*. Todo o resto —
> Epic, Feature, Story, Bug, Sub-task, DoD, HTN, sequential/concurrent/supervisor/magentic — é
> **projeção derivada**, nunca campo armazenado.

---

## 1. A frase que fecha o universo

> **Uma tarefa é uma nota com `scope`.** Sua **espécie** é `type`; seu **nível e papel** são
> derivados da árvore; seu **dono** é derivado dos eventos; seu **modo** é derivado do grafo;
> seu **pronto** é derivado dos `checks`. Nada além disso é armazenado.

Isso não é retórica: `scope` é opcional no frontmatter (`schema/frontmatter.rs:184`) e só
tarefas/containers o recebem (`task/spec.rs`). Logo **`scope` presente = item de trabalho;
`scope` ausente = conhecimento.** Essa é a fronteira que separa o `kd task` do `kd write`.

O princípio de design: **primitivos fechados, projeções abertas.** O schema não cresce quando
surge um framework novo; cresce uma *view*. É a mesma jogada de D52 (plan/epic são views
derivadas) e D87 (confiança é derivada), estendida a todo o WBS e à orquestração.

---

## 2. Diagnóstico: o que já existe e o que está colapsado

### 2.1 A escada do knudge já é a WBS estendida

| WBS / SAFe / Jira | knudge (D93) | Observação |
|---|---|---|
| Initiative / Theme | `scope=plan` (container sem pai) | nível mais alto |
| Epic | `scope=epic` (container) | |
| Story / Feature | `scope=issue` (work item) | |
| Sub-task | `scope=task` (work item folha) | |

A "estrutura estendida" do brainstorm tem **exatamente 4 níveis** — os mesmos do knudge. Não
falta nível. Falta **eixo**.

### 2.2 Dois eixos colapsados em `scope`

O Jira separa **nível** (Epic) de **espécie** (Story/Task/Bug, irmãos). O knudge forçou
`issue`/`task → type=task` (D93, `TaskSpec::note_type`), então **perdeu o eixo espécie**: não há
bug, spike, risco ou história — tudo é `task`.

O `type` já tem o vocabulário certo para a espécie (enum fechado de 11):

| Espécie de trabalho | `type` existente | Exemplo |
|---|---|---|
| Task / Story / Feature | `task` | "Implementar backoff" |
| Bug | `error` | "maxRetries off-by-one" |
| Spike / Investigação | `question` | "Medir latência do canal vetorial" |
| Risco | `risk` | "Provedor de embeddings indisponível" |
| Decisão / ADR de trabalho | `decision` | "Escolher full jitter" |

Ou seja: **a espécie já está no enum; só foi desligada para tarefas.** Não precisamos de tipos
novos (`bug`/`story`/`feature`) — precisamos **reusar os 11** no eixo espécie.

### 2.3 Falta o eixo dono (multi-agente)

O seeds tem `assignee`; o mulch não tem tarefas; o knudge não tem dono. Mas eventos já carregam
`actor` (`store/events/event.rs:49`). Logo o dono pode ser **derivado de eventos** — sem chave
canônica nova. Sem isso, `handoff` e `supervisor` (do brainstorm de orquestração) não têm onde
existir.

### 2.4 Ordem ≠ dependência (dois primitivos distintos)

`blocks` no knudge é **posição 1-based dentro do pai** (`task/spec.rs`), não bloqueio; bloqueio é
`depends_on`. O seeds mistura os dois em `blocks`. Manter separado é uma vantagem: **ordem**
(sequência planejada) e **dependência** (pré-requisito real) são coisas diferentes.

---

## 3. Os 7 primitivos (a álgebra)

| # | Primitivo | Onde mora | Derivado? |
|---|---|---|---|
| 1 | **Nó** — `type` (espécie) + `scope` (item de trabalho / nível) | frontmatter | não |
| 2 | **Aresta** — 8 `EdgeKind` em 3 famílias (composição, ordem, evidência) | frontmatter | não |
| 3 | **Agenda** — `not_before`, `expires_at` | frontmatter | não |
| 4 | **Estado** — `status` | frontmatter | não |
| 5 | **Evidência** — `checks` (DoD executável) + `outcomes` | frontmatter | não |
| 6 | **Âncora** — `anchors` (liga ao código) | frontmatter | não |
| 7 | **Claim** — evento `{op:claim, note_id, actor, at}` | `eventos/` | **sim** (dono = último claim) |

As 3 famílias de aresta:
- **Composição**: `results_in` (pai→filho) — a WBS.
- **Ordem**: `depends_on` (bloqueio) + posição `blocks` (sequência).
- **Evidência**: `supports`, `contradicts`, `extends`, `replaces`, `rejects`, `references`.

**Toda view é uma função determinística desses 7.** Nenhum conceito de framework vira campo.

---

## 4. Mapeamento completo dos brainstorms

### 4.1 Estruturas de decomposição (WBS / Jira / SAFe)

| Conceito | Como o knudge expressa | Status |
|---|---|---|
| Initiative / Theme | container `scope=plan` sem pai (raiz) | derivável |
| Epic | container `scope=epic` | armazenado |
| Feature | container com filhos (`results_in`) abaixo de epic | derivável |
| Story | `scope=issue` + `type=task` (+ tag `story` se quiser) | derivável |
| Task | `scope=issue`/`task` + `type=task` | armazenado |
| **Bug** | `scope=issue` + `type=error` | **falta habilitar** |
| **Spike** | `scope=issue` + `type=question` | **falta habilitar** |
| **Risco** | `scope=issue` + `type=risk` | **falta habilitar** |
| Sub-task | `scope=task` sob um `issue` | armazenado |
| WBS (árvore) | `results_in` transitivo | derivável |
| DoD | `checks` (validators executáveis, D54/D99) | existe |
| Acceptance Criteria | seções `acceptance` do plano (upgrade_plan T3) + `checks` | planejado |

### 4.2 Frameworks de agentes e orquestração

| Conceito | Como o knudge expressa | Status |
|---|---|---|
| WBS para agentes (escopo/guardrails/aprovação) | `checks` + `strict` + `outcomes` | existe |
| HTN (métodos/precondições) | templates de plano (métodos) + `checks` (precondições) | T3 |
| Behavior Tree | DAG + ordem; `sequence`/`parallel`/`fallback` por arestas | projeção |
| **Sequential** | filhos em cadeia `depends_on` (ou ordem `blocks`) | derivável |
| **Concurrent** | filhos independentes simultaneamente `ready` | derivável |
| **Handoff** | >1 evento `claim` no mesmo nó (troca de dono) | derivável (novo) |
| **Supervisor** | container com dono e filhos com donos distintos | derivável (novo) |
| **Magentic** | ledger = `eventos/` + `learn`; filhos criados incrementalmente | derivável |
| ReAcTree | árvore de containers com dono por nível | derivável |
| CrewAI (papéis) | dono/claim por nó | derivável (novo) |
| AutoGen (planner) | `task plan --prompt/--submit` | T3 |
| LangGraph (nós/arestas) | o próprio grafo (8 arestas) | existe |
| Task ledger | `eventos/` + `maintenance learn` | existe |
| Guardrails / approvals | `checks` + `behavior.strict` | existe |
| Escalation / fallback | `status=blocked` + `learn` + `type=question` | existe |

**Conclusão do mapeamento:** dos ~24 conceitos, **20 já são projeções dos 7 primitivos** e 3
exigem apenas *habilitar o eixo espécie* (bug/spike/risco). Só **1** exige primitivo novo:
**claim/dono**. Esse é o tamanho real do fechamento.

---

## 5. Propostas de fechamento

### C1 — Habilitar o eixo espécie (relaxar D93) — **P0**

`scope` = **nível** (onde na decomposição); `type` = **espécie** (o que é o item). Um item de
trabalho pode carregar qualquer um dos tipos "de trabalho": `task`, `error`, `question`, `risk`,
`decision`.

```
kd task new "maxRetries off-by-one" --scope issue --kind error
kd task new "Medir latencia do ask" --scope issue --kind question
```

- **Core:** `TaskSpec` ganha `kind: Option<NoteType>`; `note_type()` usa `kind` quando presente,
  senão o default por escopo (mantém compat). `--kind` valida contra o subconjunto de trabalho.
- **Contrato:** nenhuma chave nova (`type` já é canônica). `scope` continua presente.
- **Regra derivada:** `é_item_de_trabalho(nota) = scope.is_some()`. Isso permite consultas como
  `kd ask --scope issue --kind error` (bugs) sem namespace de tags.
- **Determinismo:** inalterado.

### C2 — Dono derivado de eventos (`claim`) — **P0**

```
kd task claim <ID> --by <agente>      # registra claim
kd task claim <ID> --release          # libera
kd task list --owner <agente> | --mine
```

- **Core:** função pura `ownership(events, id) -> Option<String>` = `actor` do último `claim`
  não seguido de `release`/`close`. Nada no frontmatter (D96: eventos são a verdade da auditoria).
- **Handoff** = novo `claim` por outro agente; **supervisor** = dono do container + donos dos
  filhos; **concurrent** = claims distintos em `ready` distintos.
- **Contrato:** evento novo `op=claim`; nenhuma chave canônica. `outcomes[].agent` já existe para
  atribuição no fechamento.
- **Por que derivado e não campo:** dois agentes em branches paralelas que reivindicam a mesma
  task resolvem por `merge=union` + dedup de eventos (D96), sem conflito de campo — exatamente a
  vantagem que o seeds perde ao armazenar `assignee`.

### C3 — Papel e nível derivados da árvore — **P1**

O papel exibido (Initiative/Epic/Feature/Story/Sub-task) é uma função
`f(tem_filhos, scope, type, profundidade)` — nunca um campo.

- **Core:** `task/role.rs` (novo): `pub fn role(graph, id) -> Role`.
- Mantém a escada de 4 níveis (D93) como *rótulo*, mas o **papel** é derivado: um `epic` com
  filhos `issue` é "Epic"; um `issue` com `type=error` é "Bug"; um `issue` com filhos é
  "Feature"; um `task` sob `issue` é "Sub-task".
- **Contrato:** view nova, sem storage.

### C4 — Modo de execução derivado do grafo — **P1**

`kd task graph` (ou `kd task list --tree`) projeta a álgebra num WBS legível por agente:

```
plan_00268n8q  Initiative  Entregar fila com retry resiliente        owner=sup  mode=supervisor
  epic_00zpn61q  Epic        Resiliencia de execucao
    issue_00ej81yt Story       Confiabilidade de retry
      task_00vzuh51 [x] Task   Implementar backoff com full jitter    owner=agent-a
      task_01dja2yi [ ] Bug    Persistir fila em JSONL                owner=agent-b  blocked_by=...
      task_01flvckk [ ] Spike  Tratar dead-letter                     owner=-        ready
```

- **Core:** `task/mode.rs` (novo): classificador determinístico
  `mode(container) -> Mode` a partir das arestas/claims/eventos.
  - `sequential`: filhos formam cadeia (`depends_on` ou ordem estrita).
  - `concurrent`: filhos sem dependências mútuas.
  - `supervisor`: container com dono e ≥2 filhos com donos distintos.
  - `handoff`: algum nó com ≥2 claims ao longo do tempo.
  - `magentic`: filhos criados após o submit inicial (eventos de criação espalhados) ou `learn`
    com propostas pendentes para o container.
- **Contrato:** pipe `indent|id|role|kind|status|owner|mode|statement` (golden). Nada armazenado.

### C5 — DoD e critérios de aceite como checks — **P1 (reuso)**

Já existe: `checks` é o DoD executável (D54/D99) e `outcomes` é a evidência. O upgrade_plan T3
acrescenta `acceptance` como seção de plano. **Nada novo no modelo** — apenas documentar no
`prime` que `checks` = DoD e que `close` só declara com evidência (D55).

### C6 — Ledger magentic = eventos + learn — **P2**

O "task ledger" do padrão Magentic é `eventos/` (histórico) + `maintenance learn` (propostas de
novos nós/links). `rewind` entrega a fatia atual. Nada a implementar além de rotular no `prime`.

---

## 6. Por que isso é *elegante* (e não só completo)

1. **Não cresce o schema.** Espécie reusa `type`; dono reusa eventos; papel/modo são views. As 28
   chaves canônicas (D95/D98/D100) ficam intactas. Um framework novo = uma view nova.
2. **Fecha a fronteira.** `scope` presente/ausente separa trabalho de conhecimento — uma regra,
   não uma convenção.
3. **Multi-agente é nativo.** Handoff/supervisor/concurrent caem de graça dos eventos e do grafo,
   e sobrevivem a `merge=union` melhor que um campo `assignee` (seeds).
4. **WBS e orquestração são a mesma coisa.** A árvore de composição (`results_in`) e o grafo de
   ordem (`depends_on`) já são o WBS e o LangGraph; os modos são leituras deles.
5. **Determinismo preservado.** Toda projeção tem tie-break explícito; o `prime` continua estático
   (D57) e as views dinâmicas ficam no `rewind`/`task graph`.

---

## 7. Decisões a registrar (continuação de D112)

| Dxx | Conteúdo |
|---|---|
| D113 | `scope` = nível; `type` = espécie. `kd task new --kind`; itens de trabalho podem ser `error`/`question`/`risk`/`decision`. |
| D114 | Dono **derivado de eventos** `claim`/`release`; `kd task claim`; `--owner/--mine`. Sem chave canônica. |
| D115 | Papel (Initiative/Epic/Feature/Story/Sub-task) derivado de `(filhos, scope, type, profundidade)`. |
| D116 | Modo de execução (sequential/concurrent/supervisor/handoff/magentic) derivado do grafo/eventos; `kd task graph`. |
| D117 | `é_item_de_trabalho(n) ⇔ n.scope.is_some()`; consultas filtram por `--scope`. |

---

## 8. O que NÃO fazer (anti-elegância)

| Anti-padrão | Por quê |
|---|---|
| Adicionar tipos `bug`/`story`/`feature`/`subtask` | o enum de 11 já cobre a espécie; crescer o enum é ruído |
| Armazenar `level`, `role`, `mode`, `assignee` | vira campo a sincronizar; D52/D87 já dizem: derive |
| Criar um 5º nível (Initiative) | os 4 níveis já são a WBS estendida; papel é view |
| Importar Behavior Tree / HTN como engine | o grafo é o substrato; controle é projeção, não runtime |
| Confundir ordem (`blocks`) com dependência (`depends_on`) | são primitivos distintos; manter separados |
| Namespace de tags para espécie (`kind:bug`) | enum fechado > convenção de string |

---

## 9. Impacto no `upgrade_plan.md`

C1–C4 **encaixam** nas fases já planejadas e reduzem trabalho:

- **T1/T2** (`--ready/--blocked`, impacto): passam a filtrar também por `--kind` e `--owner`
  (C1/C2), sem código novo de view.
- **T3** (plan prompt): o template declara `kind` por passo (C1) e o `--prompt` já é o "planner"
  do AutoGen.
- **X1/X2** (feedback, learn): o dono (C2) enriquece o `learn` ("tarefa do agente X fechou sem
  nota") e o `diff`.
- **Novo PR sugerido:** `PR0 = C1 + C2` (habilitar espécie + claim), antes de T1, porque T1/T2/T5
  ganham filtros de graça. Depois `PR4.5 = C3 + C4` (`task graph`).

---

## 10. Nomenclatura proposta — Épico ⊃ User Story ⊃ Task

Proposta do usuário: *User Story* (universo do problema) pertence a 1..N *Épicos* (plano maior),
que contêm N *Tasks*; o **corpo** da nota carrega o detalhamento técnico (sub-tasks, goals,
critérios).

### 10.1 Veredito

**É vantajoso — mas como nomenclatura *exibida* + regra de fronteira, não como mudança de
schema.** O modelo de 3 níveis já é uma instanciação válida do knudge **hoje**: `submit` aceita
`parent: None`, então um Épico pode ser raiz sem `plan`.

### 10.2 Mapeamento (armazenado → rótulo → semântica)

| `scope` (armazenado, contrato) | Rótulo exibido (role derivado, C3) | Semântica | Corpo |
|---|---|---|---|
| `plan` | Programa / Iniciativa *(opcional)* | agrupa épicos | estratégia |
| `epic` | **Épico** / Plano maior | objetivo amplo | contexto |
| `issue` | **User Story** | universo do **problema** (`statement` = "Como X, quero Y para Z") | critérios de aceite |
| `task` | **Task** | execução técnica (`statement` = ação) | **goals, sub-tasks, detalhe** |
| *(corpo da Task)* | Sub-task / Goals | passo local | — (não é nó) |

A separação **problema (Story) × solução (Task)** é o ganho principal: o `statement` da Story é
*o que/por quê*; o da Task é *como*. Casa com a definição de TOON do projeto ("o `statement` é a
afirmação; o corpo é o contexto que ela não diz").

### 10.3 Por que é vantajoso

1. **Legibilidade/ergonomia**: dá âncora semântica a cada nível sem criar vocabulário novo.
2. **Token economy**: sub-tasks no corpo **não viram nós** — menos nós, menos arestas, menos
   superfície de merge. O custo é contexto, não disco (tese central).
3. **Zero migração**: é um role derivado (C3) sobre o enum fechado; as 28 chaves não mudam.

### 10.4 Dois cuidados (os pontos que decidem)

**(a) Contrato de bytes.** `scope` é chave canônica com valores fechados
(`plan|epic|issue|task`). Renomear os **valores armazenados** quebra TOON/D93; renomear os
**rótulos exibidos** é grátis. Portanto: **armazene `issue`, exiba "User Story".**

**(b) N:N Story ↔ Épico.** A membership do knudge é **pai único** (D52, marcador no corpo). Se
uma Story serve a vários Épicos:
- **1 pai "home"** (membership) + **N arestas** `results_in`/`supports` para os demais;
- a view `belongs_to` já percorre `results_in` (`handoff/manifest.rs`), então a Story aparece em
  todos os Épicos sem inventar membership múltiplo.
- **Não** criar membership múltiplo nem tag `epic:<id>` (violaria D52 e o enum fechado).

### 10.5 A regra de fronteira (a parte mais valiosa)

> **O corpo é para detalhe que não precisa de grafo; o nó é para trabalho que precisa ser
> agendado, possuído ou verificado.**

Uma sub-task **vira nó** (`scope=task`) quando precisa de **qualquer** um:
`depends_on` · dono (`claim`, C2) · `checks`/DoD próprio · `not_before`/`expires_at` · âncora de
arquivo distinta.

Caso contrário, **fica no corpo** (checklist/goals). Isso preserva a resolução do grafo onde ela
importa e evita inflar o WBS com passos triviais.

### 10.6 Como encaixa nas decisões

- **C1 (espécie)**: `scope=issue` é a Story; a *espécie* (`type=task|error|question|risk`) segue
  ortogonal — uma "User Story" pode ser um bug (`type=error`).
- **C3 (papel)**: é exatamente onde a nomenclatura vive — `role(scope, type, tem_filhos)`
  devolve "Épico"/"User Story"/"Task"/"Bug"/"Sub-task".
- **C4 (`task graph`)**: renderiza a árvore com esses rótulos e o checklist do corpo.
- **`prime`**: a linha `plan ⊃ epic ⊃ issue ⊃ task` passa a exibir
  `Programa ⊃ Épico ⊃ User Story ⊃ Task` — **sem** mudar o contrato armazenado.

**Decisão nova a registrar (D118):** nomenclatura exibida = role derivado; `scope` armazenado
permanece `plan|epic|issue|task`; sub-task é detalhe de corpo por padrão, promovível a nó pela
regra de fronteira (§10.5).

### 10.7 O Programa é um arquivo externo — o elo (D119)

Decisão do usuário: o **Programa** (ponto central de uma grande feature) vive como **arquivo
real** em `/plan/*.md` — autoral, git-tracked, legível — e o `plan`/Iniciativa **sai do escopo**.
O knudge cuida da corrente menor (Épico → User Story → Task).

**O elo é `anchors` (D86), não um scope nem um campo:**

| Papel | Onde mora |
|---|---|
| Programa (o "porquê") | `plan/<slug>.md` (markdown livre) |
| Épico-raiz (a raiz knudge) | `scope=epic`, sem pai, `--anchors plan/<slug>.md` (`--source` opcional) |
| Corrente (o "como") | `epic --results_in--> story --results_in--> task` |

**Por que `anchors` é o melhor elo:**
1. É primitivo **existente**, com `content_hash` + **verify-on-hit** (D86): um doc que evolui deixa
o Épico "stale" **sinalizado, não apagado** — o comportamento certo para um programa vivo.
2. `rewind --files` já usa o canal; `kd task graph --program plan/<slug>.md` só resolve
âncora → Épico-raiz → subárvore.
3. N programas por projeto sem custo de schema; **zero chave TOON nova**.
4. N:N de graça: o mesmo arquivo pode ancorar mais de um nó; o mesmo Épico pode ancorar vários
docs (ex.: `plan/foo.md` + `plan/foo-api.md`).

**Consequência para C3 (papel):** "Programa" deixa de ser um nível e passa a ser o **rótulo do
Épico-raiz ancorado**. A árvore renderizada começa no path do arquivo, não num scope.

**Consequência para a tabela §10.2:** a linha `plan` → "Programa/Iniciativa" é **substituída**
pelo arquivo externo. O enum mantém `plan` só por compat de bytes (D95/D93); nenhum uso novo.

**Decisão nova a registrar (D119):** Programa = arquivo `plan/*.md` ancorado ao Épico-raiz;
`scope=plan` deprecado; `kd task graph --program`; check `program-anchor` no `doctor`;
config `programs.glob`. Detalhe de implementação em `upgrade_plan.md` §PR0.5.
