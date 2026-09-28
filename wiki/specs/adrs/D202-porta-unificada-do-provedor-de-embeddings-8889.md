# D202 — Porta unificada do provedor de embeddings = 8889

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D101/D182.

## Decisão

**Porta unificada do provedor de embeddings = `8889`.** O default de `embeddings.endpoint` (`config/schema/keys_embeddings.rs`), o fallback de runtime (`adapters::http::DEFAULT_ENDPOINT`), o `--port` de `kd drain service` (`cli/health.rs`) e o `DEFAULT_PORT` do worker (`scripts/knudge-idle.sh`) passam a ser **8889** (`http://127.0.0.1:8889/v1/embeddings`). Elimina a divergência worker (8999) × config (8080) que causava degradação silenciosa. Revisa D101/D182.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
