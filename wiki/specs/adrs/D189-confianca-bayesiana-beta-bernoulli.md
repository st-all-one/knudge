# D189 — Confiança bayesiana Beta-Bernoulli

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D87.

## Decisão

**Confiança bayesiana Beta-Bernoulli.** `outcomes` viram ensaios de Bernoulli: `success`=1, `partial`=0,5, `failure`/`abandoned`=1 falha. Posterior `Beta(1+s, 1+f)` (revisa D87): a **média** alimenta o canal `stars` e o **limite inferior** de 95 % (Wilson) é a confiança conservadora (1 sucesso ≈0,21 × 20 ≈0,84; `lifecycle/beta.rs`, puro, sem dep). `Meta` ganha `failures` (`filter.rs`; `INDEX_FORMAT` → `retrieval-v3`). `confidence_score` troca `FEEDBACK_WEIGHT·confirmação` por `base + lower_bound` (o feedback explícito continua 0,2). Absorve E16/D174 no que tange `feedback` (falhas); a persistência de `drift` fica pendente. `DIVERGENCES.md` #98. Fecha E19-T01 (fórmula).

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #98.
- Fecha E19-T01 (fórmula.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
