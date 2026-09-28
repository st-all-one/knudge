# D153 — Sincronização multi-dev

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Complementa D26/D28/D31/D148/D150.

## Decisão

**Sincronização multi-dev.** Verdade = **notas** (endereçadas por conteúdo, arquivo-por-nota → merge natural); índice é **derivado** (reconstruir); **eventos e cache vetorial** por `merge=union` + dedup. Chave lógica do cache = `(body_hash, model)`; loader **idempotente**, **model-aware** e com **desempate determinístico** (`created_ms`). Conflito de nota (mesma `statement`, corpos divergentes) é pulado e reportado pelo `doctor` — nunca auto-mergeado. Sem lock distribuído/CRDT (o git sincroniza). Premissa: mesmo modelo/config. Complementa D26/D28/D31/D148/D150.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
