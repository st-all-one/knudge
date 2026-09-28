# D151 — ask expõe a contribuição de canal no --json

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D39/D121/D124.

## Decisão

**`ask` expõe a contribuição de canal no `--json`.** Cada hit ganha `channels: {lexical, anchor, semantic, recent, stars}` (parcelas do RRF/boost); o pipe `id\|statement\|score\|why` **não muda**; a confirmação derivada de tarefas (X1/D108) aparece no rótulo. A recalibração de `semantic_weight`/`rrf_k`/`limit` é **offline** (bancada `bench/`), pois o `maintenance eval` saiu (D145). Revisa D39/D121/D124.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
