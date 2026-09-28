# D154 — Renovação de shelf-life por uso

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D44/D135.

## Decisão

**Renovação de shelf-life por uso.** O uso (citação) vira derivado `.idx/usage.jsonl` (`UsageStore`), nunca verdade e nunca evento de auditoria; purgado por `purge_derived` (D84). Com `retention.renew_on_use=true`, a expiração é `max(created_at, last_seen) + prazo` e **só estende** — `prune`/`rewind`/`plan` a respeitam via `is_expired_with`/`freshness_with`. `ask`/`rewind` creditam os ids devolvidos, coalescidos numa escrita por invocação (padrão D85/D131). Default `false` preserva o comportamento byte-a-byte. Revisa D44/D135.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
