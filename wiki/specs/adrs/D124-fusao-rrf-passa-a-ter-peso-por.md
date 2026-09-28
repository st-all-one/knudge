# D124 — Fusão RRF passa a ter peso por canal

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revê D81.

## Decisão

**Fusão RRF passa a ter peso por canal** (`recall.lexical_weight`/`anchor_weight`/`semantic_weight`; revê D81). Default `semantic_weight=30` **Pareto-domina** o neutro no corpus PT-BR (R@1/R@5/MRR/nDCG@5 ≥ 1:1) e `≥80` converge para o ranking vetorial puro; corrige a diluição do canal vetorial por votos lexicais.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
