# D82 — Orçamento do prime sem tokenizer

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**Orçamento do `prime` sem tokenizer**: manter a heurística `ceil(len/4)` (D40), aplicada item a item — trunca o último e ignora sobra < 100 tokens. `palavras × 1.3` fica como alternativa só se o corpus exigir.

## Impacto

- Orçamento do `rewind` explicitado **sem tokenizer** (D40 refinado).

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
