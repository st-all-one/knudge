# D01 — ID endereçado por conteúdo

- **Status:** Aceita
- **Categoria:** A. Identidade e IDs

## Contexto

Bloco **A. Identidade e IDs**.

## Decisão

**ID endereçado por conteúdo** — hash curto de `type + statement`. Torna o `write` idempotente sob retry. Se a chave mudar, cria novo id + `superseded_by` (híbrido: identidade pelo conteúdo, linhagem explícita).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
