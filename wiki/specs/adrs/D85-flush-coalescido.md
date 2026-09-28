# D85 — Flush coalescido

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**Flush coalescido (debounce)** do `.idx`/embeddings com *dirty flag*, **flush forçado na saída** — rajadas de 10–20 notas causam um único rewrite.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
