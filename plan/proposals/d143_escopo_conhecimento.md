# D143 — escopo de conhecimento: ponto de partida no `knowledge map` e no `rewind`

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão de `docs/07-manutencao.md`. Revisa **D128/D129**.

## 1. Diagnóstico (evidência)

- **`kd knowledge map` não tem ponto de partida.** `--scope` aceita **só container**
  (`scope_clusters` filtra por `belongs_to`, via `results_in`/`depends_on`); conhecimento sem
  hierarquia de tarefa fica sem escopo. `--axis` agrupa por `anchor`/`type`/`classification`/
  `container`, mas sobre **todo** o corpus (`docs=N`) — com 10k notas, o universo inteiro é ruído.
- **`kd rewind` escopa por container/arquivo** (`--scope`/`--files`), mas não por tag/âncora — o
  mesmo espaço de conhecimento não é acessível.

## 2. Decisões

1. **`knowledge map` ganha filtros de corpus** aplicados **antes** de clusterizar:
   `--tag T...`, `--anchor PATH...` (glob), `--type T...`, `--class C...` (repetíveis, como no
   `ask`). O `--scope <CONTAINER>` continua.
2. **Vizinhança:** `--around <ID> --depth N` (expansão pelo grafo, como `ask --around`) limita o
   corpus à vizinhança de uma nota — outro ponto de partida.
3. **`--universe`:** visualiza o projeto **inteiro** de forma explícita. **Sem filtro, sem
   `--around` e sem `--universe` → erro** (`invalid_input`, D130) — o default deixa de despejar o
   universo como ruído e passa a exigir um escopo.
4. **`rewind` ganha os mesmos filtros** (`--tag`/`--anchor`/`--type`/`--class` e `--around`) para
   escopar o working set/handoff ao espaço de conhecimento. O manifest default segue o estado do
   projeto; os filtros restringem o que entra.

### 2.1 Princípio geral: nada de operação sem escopo

**A IA declara explicitamente o que quer.** Toda operação que **varre o corpus** e pode despejar o
universo exige escopo explícito. Aplicado agora ao `knowledge map` (item 3); **candidatos a
revisar** sob o mesmo princípio (decisão à parte):

- `kd maintenance learn` / `compact` / `prune` (hoje escopáveis por `--scope`, mas rodam sobre o
  corpus todo quando ausente);
- `kd task list` sem filtro (lista todas as tarefas).

## 3. Consequências

- O filtro roda sobre o `Index` antes de `structural_clusters`; `--semantic` (fase 2) segue
  **dentro** do subconjunto já escopado.
- `--axis` + `--universe` = agrupamento do projeto todo (ex.: `--universe --axis type`).
- `rewind` reusa o mesmo predicado de filtro (uma fonte de verdade com `ask`).
- Revisa **D128/D129**; alinha com a tese "custo é contexto" (nunca despejar o universo).

## 4. Raio de alcance (quando executar)

- `cli/knowledge.rs` (`Map` args), `commands/knowledge/map.rs` (filtro antes de clusterizar),
  `cli/mod.rs` (`RewindArgs`), `commands/rewind.rs`, reuso do filtro de `retrieval::Filter`,
  `docs/07-manutencao.md`, `prime.rs`, goldens/testes.

## 5. Aceite

- [x] `knowledge map --tag X` / `--anchor 'src/**'` / `--type T` / `--class C` restringem o corpus.
- [x] `knowledge map --around <ID> --depth N` mostra a vizinhança.
- [x] `knowledge map` sem escopo e sem `--universe` → exit 2 (com orientação).
- [x] `knowledge map --universe` mostra o projeto inteiro.
- [x] `rewind --tag/--anchor/--type/--class` escopam o handoff.
- [x] Nenhuma operação varre o corpus sem escopo explícito (princípio §2.1).
- [x] `make check` verde; docs/`prime`/goldens atualizados.
