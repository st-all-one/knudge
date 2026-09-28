# D179 — Fusão recalibrada por medição

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D81/D123/D151.

## Decisão

**Fusão recalibrada por medição** (revisa D81/D123/D151). A bancada de qualidade ganha famílias multi-canal (`working-set` com o canal de âncoras; `sinonimo` com um canal vetorial sintético) e `make bench-sweep` varre `rrf_k` × pesos (`bench/t09_fusao.md`). Medição: `rrf_k` é **inerte** no corpus rotulado (`10`–`60` dão métricas idênticas — mudança **rejeitada**); o lever é `anchor_weight`: `1.0`/`k=60` faz um match exato de âncora **empatar** com um rank-1 lexical (família `working-set`, nDCG@5 87,7 %), `2.0` corrige (100 %). Adota `DEFAULT_ANCHOR_WEIGHT = 2.0` (config `recall.anchor_weight`); mantém `rrf_k=60` e `semantic_weight=30` (medidos ≥ baseline). `DIVERGENCES.md` #105. Fecha E16-T09.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #105.
- Fecha E16-T09.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
