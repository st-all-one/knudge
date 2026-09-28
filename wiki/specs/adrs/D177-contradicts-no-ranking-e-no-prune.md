# D177 — contradicts no ranking e no prune

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**`contradicts` no ranking e no prune.** O lado **perdedor** de uma aresta `contradicts` declarada (menor confiança derivada — Beta + idade + tarefa) é rebaixado no `rank`/`recall` pela `CONTRADICTION_PENALTY` (0,1) e proposto no `prune` com `DemotionReason::Contradicted` (só propõe — D112; membros de ciclo protegidos — D45). Empate **não** elege perdedor (determinístico). `retrieval/contradiction.rs` pré-indexa por id (O(C)); `rank` passa a receber `&Graph`. `DIVERGENCES.md` #104. Fecha E16-T07 (o check `doctor` `CheckId::Contradictions` fica como follow-up).

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #104.
- Fecha E16-T07 (o check `doctor` `CheckId::Contradictions` fica como follow-up.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
