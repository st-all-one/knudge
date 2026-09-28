# D192 — Autoridade no grafo

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Autoridade no grafo (PageRank/PPR).** `graph/rank.rs` (puro, sem dep) calcula `PageRank` global e **PPR** semeado pelo *working set* sobre as arestas de autoridade (`references`/`supports`/`extends`/`replaces`), por iteração de potência determinística (damping 0,85, ≤32 iterações, tolerância L1 1e-8, ordem canônica dos ids). Vira o canal `ppr` da fusão RRF (`FusionWeights.ppr`, config `recall.ppr_weight`), **default 0,0** (desligado — só compensa em corpora com arestas); filtrado por `allowed`. O `--json` do `ask` ganha `channels.ppr` (aditivo) e `Why` (conjunto fechado, D39) **não muda**. `DIVERGENCES.md` #101. Fecha E19-T04.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #101.
- Fecha E19-T04.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
