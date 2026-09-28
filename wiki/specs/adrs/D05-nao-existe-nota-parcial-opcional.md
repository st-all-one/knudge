# D05 — Não existe nota parcial/opcional

- **Status:** Aceita (com ponto de atenção)
- **Categoria:** B. Schema e serialização (contrato de bytes)

## Contexto

Bloco **B. Schema e serialização (contrato de bytes)**.

## Decisão

**Não existe nota parcial/opcional.** Ou a nota é válida e existe, ou não existe. Campos opcionais são **omitidos**, nunca `null`/vazio. Sem `draft`, sem nota incompleta.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
