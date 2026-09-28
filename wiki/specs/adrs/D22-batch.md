# D22 — batch

- **Status:** Aceita
- **Categoria:** D. Escrita, atomicidade e crash

## Contexto

Bloco **D. Escrita, atomicidade e crash**.

## Decisão

`fsync` em **batch** (write em page cache; fsync no rebuild/sync).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
