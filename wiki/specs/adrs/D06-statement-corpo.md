# D06 — statement + corpo

- **Status:** Aceita
- **Categoria:** B. Schema e serialização (contrato de bytes)

## Contexto

Bloco **B. Schema e serialização (contrato de bytes)**.

## Decisão

`body_hash` = hash de **`statement` + corpo**, após **NFC + trim + colapso de whitespace**. Congelado por teste.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
