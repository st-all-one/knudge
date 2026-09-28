# D106 — `rewind` emite `next

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

`rewind` emite `next:` (tarefas `ready` abertas por impacto) e `fresh:` (`stale`/`expiring`/`pending`); `prime` permanece estático (D57). `impact` = tarefas abertas com `depends_on` reverso.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
