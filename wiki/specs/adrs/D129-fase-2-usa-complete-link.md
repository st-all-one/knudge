# D129 — Fase 2 usa complete-link

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Fase 2 usa complete-link** (`cluster_by_similarity`): um id só entra num cluster se for similar (`>= clusters.similarity_threshold`) a **todos** os membros. Corrige o efeito do leader/single-link em que um item central puxa vizinhos dissimilares entre si (com threshold 0.5, 31 itens viravam um só grupo).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
