# D88 — rewind emite context_id endereçável

- **Status:** Aceita
- **Categoria:** Q. Extrações do arags (D81–D92)

## Contexto

Bloco **Q. Extrações do arags (D81–D92)**.

## Decisão

**`rewind` emite `context_id` endereçável**; `kd rewind --resume <id>` devolve o **mesmo contexto 1:1**, sem re-busca — handoff reprodutível entre agentes/rodadas.

## Impacto

- `rewind` emite **`context_id`** para handoff 1:1 (`--resume`).

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
