# D94 — strict é config de projeto

- **Status:** Aceita
- **Categoria:** R. Superfície CLI v2 (D93–D94)

## Contexto

Bloco **R. Superfície CLI v2 (D93–D94)**.

## Decisão

**`strict` é config de projeto** (`[behavior] strict = false` em `.knudge/config.toml`), **não** flag de CLI nem subcomando; vale para warnings de leitura/retrieval/embeddings.

## Impacto

- `strict` vira config de projeto (`[behavior] strict`), sem flag.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
