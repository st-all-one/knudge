# D173 — Termos de alta frequência

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Termos de alta frequência.** `STOPWORDS` ganha as formas dobradas do PT (`ja`, `sao`, `nao`, `tambem`, `ate`, `apos`, `entao`, `porem`, `voce`, …). O canal lexical descarta termos com `df/N ≥ recall.max_term_ratio` (default `0.9`) via `Index::score_with`/`term_ratio`, **desligado para corpora < `MIN_CUTOFF_CORPUS` (64 notas)** — abaixo disso `df/N` é alto para quase todo termo; `0` desliga e ajusta por projeto. O IDF já atenua termos comuns, mas o RRF usa *ranks*: o corte evita um rank-1 espúrio. `DIVERGENCES.md` #97. Fecha E16-T04.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #97.
- Fecha E16-T04.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
