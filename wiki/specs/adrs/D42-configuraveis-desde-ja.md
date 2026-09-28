# D42 — configuráveis desde já

- **Status:** Aceita
- **Categoria:** G. Retrieval e ranking

## Contexto

Bloco **G. Retrieval e ranking**.

## Decisão

Embeddings **configuráveis desde já** via `config.toml` (provedor plugável); sempre derivados/opcionais. `enabled=false`/`provider=none` cai para BM25 puro.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
