# D95 — Hash curto = SHA-256 truncado aos 4 primeiros bytes

- **Status:** Aceita
- **Categoria:** S. Contrato de bytes (D95)

## Contexto

Bloco **S. Contrato de bytes (D95)**.

## Decisão

**Hash curto = SHA-256 truncado aos 4 primeiros bytes** (`u32` big-endian). `body_hash = hex8(normalize(statement) + LF + normalize(body))` (D06); `id = <prefixo>_<base36(8)>(type + U+001F + normalize(statement))` (D01). O `id` é **histórico**: reclassificar o `type` não o reescreve (D02). `normalize` = NFC + trim + colapso de whitespace. A **gramática TOON v1** (raw UTF-8, ordem canônica, inteiros sem `.0`, lista vazia omitida, newline `LF`) está em `TOON.md`.

## Impacto

- Fixa o hash curto (SHA-256→u32), a chave do `id` e a gramática TOON v1 (`TOON.md`).

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
