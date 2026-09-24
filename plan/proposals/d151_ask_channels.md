# D151 — `ask` expõe a contribuição de canal no `--json`

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: item 2 do `rascunho_melhorias.md`. Revisa **D39/D121/D124**. Sem execução.

## 1. Decisão

- O `--json` do `ask` inclui, por hit, a **contribuição de cada canal**:
  `channels: { lexical, anchor, semantic, recent, stars }` (parcelas do RRF/boost).
  O **pipe** (`id|statement|score|why`) **não muda**.
- A **confirmação derivada de tarefas** (X1/D108) aparece no `why`/`channels` (hoje entra no
  score mas não no rótulo).
- A **recalibração** de `semantic_weight`/`rrf_k`/`limit` é **offline** (bancada `bench/`), pois o
  `maintenance eval` saiu (D145); documentar o procedimento e o valor medido.

## 2. Consequências

- O agente vê *por que* um hit subiu, não só o rótulo dominante (`why`).
- Sem artefato novo; sem mudança no pipe. `--json` ganha `channels`.
- Ordenação default **não** muda nesta decisão (só expõe); mudança de pesos exige golden + `Dxx`.

## 3. Raio de alcance (quando executar)

- `retrieval/rrf.rs` + `retrieval/pipeline.rs` (expor as parcelas por hit), `retrieval/why.rs`
  (confirmação de tarefas), `commands/ask/query.rs` (`hit_json` com `channels`),
  `docs/03-ask.md`, `prime.rs`, goldens/testes.

## 4. Aceite

- [ ] `ask --json` traz `channels` por hit; o pipe segue `id|statement|score|why`.
- [ ] A confirmação de tarefas é rotulada.
- [ ] Doc descreve a recalibração offline.
- [ ] `make check` verde; goldens atualizados.
