# D86 — anchors com path + content_hash derivado

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**`anchors` com `path` + `content_hash` derivado** (hashes em `.idx/anchors.jsonl`, **nunca** no frontmatter) e **verify-on-hit**: `cited` invalida, `context` não; nota **stale é sinalizada, não apagada**.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
