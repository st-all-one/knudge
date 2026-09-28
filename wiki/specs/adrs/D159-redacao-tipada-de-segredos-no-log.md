# D159 — Redação tipada de segredos no log

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Redação tipada de segredos no log.** `Redactor` emite `[REDACTED:<tipo>]` (`authorization`/`token`/`api_key`/`password`/`secret`/`bearer`/`custom`; literais de `[secrets]` → `secret`) em vez do marcador anônimo. Muda só o contrato observável de log (R22) — o knudge não persiste corpo redigido. Allowlist preservada; nenhum valor vaza. Revisa R22.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
