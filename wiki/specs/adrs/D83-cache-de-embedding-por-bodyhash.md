# D83 — Cache de embedding por body_hash

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**Cache de embedding por `body_hash`** + estado derivado `indexed\|pending\|stale` (extensão de D80); falha de embedding **marca `pending`, nunca descarta a nota**; worker de reconcile re-embeda do corpo.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
