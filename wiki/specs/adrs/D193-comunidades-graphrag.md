# D193 — Comunidades + GraphRAG

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Comunidades + GraphRAG (soft).** `graph/communities.rs` (puro) implementa Louvain determinístico (*local moving* + agregação; ordem canônica, ≤8 níveis, ≤64 passos, empate mantém a comunidade corrente) sobre grafo ponderado não-dirigido. `lifecycle/communities.rs` monta o grafo a partir das **arestas explícitas** (peso 1) + **âncoras compartilhadas** (clique; estrela acima de 64 membros) e produz `Community { members, terms }` com resumo local (`content_terms`, top 8). Exposto em `knowledge map --communities` (off-path; JSON aditivo `data.communities`) e materializado em `--write` (`## Comunidades` no `MAP.md` + hub `meta`). Sem chave nova e sem bump de `schema_version`. `DIVERGENCES.md` #102. Fecha E19-T05.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #102.
- Fecha E19-T05.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
