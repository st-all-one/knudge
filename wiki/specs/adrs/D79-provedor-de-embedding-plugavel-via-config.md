# D79 — Provedor de embedding plugável via config

- **Status:** Aceita
- **Categoria:** M. Arquitetura e distribuição

## Contexto

Bloco **M. Arquitetura e distribuição**.

## Decisão

**Provedor de embedding plugável via `config.toml`** (`local`/`http`/`none`); modelo default **`ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, cosseno nativo, multilíngue — **D123**), com alternativas por config (`msmarco-MiniLM-L12-cos-v5`, `paraphrase-multilingual-MiniLM-L12-v2`, `embeddinggemma-300m`). `revision` pinada e re-embed quando o modelo muda. Ver `04_embeddings.md`. (**D101** fixa a execução em **HTTP local**, sem inferência in-process.)

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
