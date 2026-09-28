# D175 — Idade no ranking sem query

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Idade no ranking sem query.** Em `confidence_score` (D87/D189), quando não há similaridade textual (`similarity = 0`, o caso de `knowledge rank`), a idade entra de forma **aditiva**: `recency = AGE_WEIGHT (0,05) · age_factor · (1 − similarity)`. Antes o `age_factor` era multiplicado por `similarity = 0` e sumia, deixando o desempate só no id. A evidência (limite inferior Beta) continua dominando — o termo é ≤ 0,05 e só desempata. `DIVERGENCES.md` #103. Fecha E16-T06.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #103.
- Fecha E16-T06.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
