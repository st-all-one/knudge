# D81 — recall funde canais por RRF

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**`recall` funde canais por RRF** (`1/(k+rank+1)`, k=60) com **tie-break determinístico `(score desc, id asc)`** e **degradação graciosa** — canal ausente/falho (âncoras, vetor) cai para o lexical sem quebrar a busca.

## Impacto

- `recall` ganha **fusão RRF determinística** + degradação graciosa.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
