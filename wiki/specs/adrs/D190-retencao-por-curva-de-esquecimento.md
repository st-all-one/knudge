# D190 — Retenção por curva de esquecimento

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Retenção por curva de esquecimento (FSRS-like).** O shelf-life deixa de ter prazo fixo: cada `outcome` de sucesso estende o prazo em `retention.growth_percent` (default 50 %) da base e **reseta o relógio** (`origin = max(created, último ensaio, último uso se `renew_on_use`)`), substituindo E16/D178. A retenção `R(t)=limiar^(t/ttl)` (`lifecycle/retention.rs`, pura) aparece no `prune --json` (`retention`); o cruzamento `R=limiar` é o prazo. `foundational` segue nunca expirando. `DIVERGENCES.md` #99. Fecha E19-T02.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #99.
- Fecha E19-T02.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
