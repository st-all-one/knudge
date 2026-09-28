# D148 — O cache vetorial é versionado

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revê D15/D34/D83.

## Decisão

**O cache vetorial é versionado (caminho B).** O vetor é função pura de `(modelo, body_hash)`, então o cache de embeddings deixa de ser descartável e passa a ser **versionado** (opt-in `embeddings.version_cache`), com `merge=union` (D31) + dedup e **sem eviction** quando versionado; o `embeddings.jsonl` se reconstrói de notas + cache **sem chamar o modelo**. O cache versionado mora em **`.knudge/emb_cache.jsonl`** (fora do `.idx/`, que segue 100% derivado). Sacrifica espaço (barato) por velocidade de dev (caro). Revê D15/D34/D83.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
