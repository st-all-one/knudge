# D134 (proposta) — `epic` é a raiz; `issue` é opcional; `scope=plan` sai

> **Status:** **decisão fechada como D134** e **implementada** (v0.3.0). Conclui a
> depreciação de `scope=plan` já declarada em **D119** e relaxa a hierarquia de D93/D113. Revisa
> D52/D93/D113/D115/D119/D127.

## 1. Decisão

O nível **`plan`** deixa de existir como `scope`. O **`epic`** passa a ser o container-raiz (pode
viver sozinho) e o **`issue`** passa a ser **opcional** entre `epic` e `task`.

```
antes:  plan ⊃ epic ⊃ issue ⊃ task   (adjacência obrigatória, 4 níveis)
depois: epic ⊃ { issue ⊃ task | task }   (epic = raiz; issue opcional)
```

- `epic` **não tem pai**; viver sozinho é válido.
- Ancorar o épico (`--anchor plan.md`, `plan/*.md` ou qualquer arquivo) é **fortemente
  recomendado**, não obrigatório — é o elo com o "porquê" externo (D119).
- O plano global continua sendo um **arquivo `.md` real** (D119); ele **não** é um `scope`.

## 2. Por que (evidência)

- **D119 já deprecou `scope=plan`** e definiu o Programa como `plan/*.md` ancorado ao Épico-raiz.
  D134 só remove a variante que ficou para trás.
- A obrigatoriedade do `issue` vem de `task/hierarchy.rs::validate_parent`, que exige o pai
  **imediatamente externo** (`expected_parent(child) == parent`). Não é a existência do `issue`
  que incomoda — é a adjacência. `epic → task` hoje é **rejeitado** (`Schema`).
- O corpus da bancada usa os 4 scopes (`knudge-ts/.knudge/notas`: 1 plan, 1 epic, 1 issue,
  4 task) — a remoção precisa de destino para as notas `scope: plan`.
- `id = hash(type + U+001F + normalize(statement))` (D95): `plan` e `epic` são ambos
  `type=container`, então **reescrever `scope: plan` → `scope: epic` não muda o id** — a migração
  é estável e endereçável.

## 3. Consequências de projeto

| Tema | Antes | Depois |
|---|---|---|
| Nível | intrínseco ao `scope` (`depth()`) | **derivado da árvore** (profundidade real) |
| Pai válido | exatamente o externo | **qualquer `rank` estritamente menor** (`epic` < `issue` < `task`) |
| Raiz | `plan` | `epic` |
| Papel (D115) | `Initiative` no topo | `Epic` no topo; `Feature`/`Story`/`Sub-task` por profundidade |
| Programa (D119) | épico-raiz ancorado | inalterado |
| `task plan --submit` | filho = `child(parent_scope)` (`plan→epic→issue→task`) | filho = **`task`** (folha executável) |
| `kd write` | rejeita `task`/`container` | inalterado |

## 4. Raio de alcance (arquivos)

### Core

- `schema/types.rs` — `Scope::ALL: [Self; 3] = [Epic, Issue, Task]`; remover `Plan`;
  `depth()` → `rank()` nominal (`Epic=1, Issue=2, Task=3`); `parent()` deixa de ser adjacente
  (ou sai; a validação passa a usar `rank`); `FromStr` rejeita `"plan"`.
- `task/hierarchy.rs` — `validate_parent`: exigir `parent.rank() < child.rank()` e filho ≠ `Epic`.
  `child(Epic)=Task`, `child(Issue)=Task`, `child(Task)=None`; remover `expected_parent`.
- `task/spec.rs` — `note_type()`: `Epic => Container`; `Issue|Task => work`. `validate_kind`
  idem.
- `task/role.rs` — remover `Scope::Plan`; `role` passa a receber a **profundidade** (derivada da
  árvore) para separar `Story` (folha sob épico) de `Sub-task` (folha sob issue) e `Feature`
  (container abaixo do topo). Ajustar assinatura e chamadas.
- `task/progress.rs` — `epic_of` itera no máximo **3** níveis (era 4).
- `graph/mod.rs` — `is_work_item` inalterado (espécie de trabalho + `scope` presente).
- `task/template.rs` / `task/plan.rs` — `submit_plan` usa o novo `child` (folha `task`).

### CLI

- `commands/task/create.rs`, `query.rs` — render/parse do `scope` sem `plan`.
- `commands/task/plan.rs`, `graph.rs` — papel e filho.
- `commands/prime.rs` — `TAREFAS` vira `epic ⊃ {issue ⊃ task | task}` + "epic é raiz; ancore".
- `commands/init.rs` (AGENTS.md embutido) e `docs/` se citarem a cadeia de 4.

### Docs/contrato

- `plan/implementation/16_cli_surface.md`, `17_matriz_aceitacao.md`, `ARCHITECTURE.md`,
  `README.md`, `llms.txt`, `SKILL.md`, `docs/05-task.md`, `TOON.md` (lista de `scope`),
  `plan/00_panorama.md` (tabela do schema), `DIVERGENCES.md` (nova borda + teste).
- `plan/03_decisoes-fechadas.md` — linha **D134** + nota "revisa D52/D93/D113/D115/D119/D127".

### Testes/goldens

- `schema/tests.rs` (`Scope::from_str("plan")` → erro; `ALL` com 3), `task/tests/hierarchy.rs`,
  `role.rs`, `kind.rs`, `plan.rs`, `progress.rs`.
- Goldens: `prime`, `task list/show/graph`, mensagens de erro de hierarquia.
- Novos: `epic → task` direto **aceito**; `epic → issue → task` **aceito**; `task → epic`
  **rejeitado**; `scope=plan` na CLI **rejeitado** (`invalid_input`=2); nota `scope: plan`
  existente → warning/`doctor`.

## 5. Migração das notas `scope: plan` (ponto bloqueante)

Como D14 proíbe alias/retrocompatibilidade, remover a variante faz `Scope::from_str("plan")`
falhar; `Graph::from_notes` propaga o erro e a **leitura tolerante pula a nota** (D18) — o
container raiz some do grafo.

**Recomendação: `kd maintenance doctor --fix` reescreve `scope: plan` → `scope: epic`.** É
lossless (mesmo `type=container`, **mesmo id**) e conclui a depreciação de D119 de forma
explícita e auditável. Alternativa: aceitar o skip com warning e orientar a recriação manual.

## 6. Decisões confirmadas (D134)

| # | Questão | Decisão |
|---|---|---|
| M1 | Destino das notas `scope: plan` | `doctor --fix` reescreve → `scope: epic` (lossless, mesmo id) |
| M2 | Filho padrão do `task plan --submit` | `task` (folha); `issue` só manual |
| M3 | Papel com nível derivado | derivar por profundidade (mantém `Feature/Story/Sub-task`) |
| M4 | Épico sem âncora | **warning** no `doctor`/`task new` (não erro) |

## 7. Aceite

- [ ] `Scope::ALL` = 3; `kd task new --scope plan` → exit 2.
- [ ] `epic → task` e `epic → issue → task` aceitos; `task → epic` e `epic → epic` rejeitados.
- [ ] Épico raiz sem pai é válido; `doctor` avisa se não houver âncora.
- [ ] Goldens e `prime` atualizados; `make check` verde.
- [ ] `DIVERGENCES.md` com a borda e o teste que a trava.
