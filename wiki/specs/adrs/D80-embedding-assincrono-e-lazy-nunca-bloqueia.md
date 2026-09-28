# D80 — Embedding assíncrono e lazy; nunca bloqueia

- **Status:** Aceita
- **Categoria:** M. Arquitetura e distribuição

## Contexto

Bloco **M. Arquitetura e distribuição**.

## Decisão

**Embedding assíncrono e lazy; nunca bloqueia.** `write`/`recall`/rebuild seguem sem esperar o modelo; notas recém-criadas ficam **“dark”** no espaço vetorial até serem digeridas por uma fila (gap tolerado em rajadas de 10–20). Dedup no write é **lexical**; o semântico é **eventual** (reconciliação). Estado `embedded\|pending\|stale` é derivado, em `.idx/`; `prime` reporta `embeddings_pending`.

## Impacto

- Embedding **assíncrono/lazy**: retrieval e write nunca bloqueiam; gap vetorial tolerado.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
