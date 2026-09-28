# D96 — Evento

- **Status:** Aceita
- **Categoria:** T. Persistência e eventos (D96)

## Contexto

Bloco **T. Persistência e eventos (D96)**.

## Decisão

**Evento** = `{id, op, note_id?, at, actor?, data?}` em `eventos/events.jsonl`; `id = evt_<base36(8)>` derivado do conteúdo (D95) e usado como chave de **dedup on-read**. Leitura tolerante (linha ruim → skip + warning). Rotação por tamanho: segmento ativo `events.jsonl` → `events-NNNN.jsonl`; checkpoint derivado em `.idx/events.checkpoint`. `revision` é **contador de versões** (default 1; cada `update` incrementa), não CAS.

## Impacto

- Fixa o registro de evento (`id` derivado + dedup on-read), rotação por tamanho e a semântica de `revision`.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
