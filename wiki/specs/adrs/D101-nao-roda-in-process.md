# D101 — não roda in-process

- **Status:** Aceita
- **Categoria:** Y. Embeddings via HTTP local (D101)

## Contexto

Bloco **Y. Embeddings via HTTP local (D101)**.

## Decisão

O provedor de embedding **não roda in-process**. `provider` é `http` (default) \| `lightweight` \| `none`; o default consome um **servidor local OpenAI-compatible** — `llama-server -m msmarco-MiniLM-L12-cos-v5.Q5_K_M.gguf --embeddings` (ou TEI/Ollama/vLLM) — via `embeddings.endpoint` (`http://127.0.0.1:8080/v1/embeddings`), com `timeout_ms`, `retries` e `api_key_env`. O cliente é HTTP/1.1 **bloqueante** sobre `std::net` (**sem** `tokio`/`reqwest` — R16/R43) e `https://` exige proxy/TLS terminator. `lightweight` (hash, D89) cobre CI/offline; `none` cai para BM25. Inferência `local` in-process (ONNX/candle/llama.cpp) é **recusada** (R16/R43). O índice é **invalidado** quando modelo/revisão/dimensão mudam (D79).

## Impacto

- Embeddings via **HTTP local** (OpenAI-compatible); `provider = http\|lightweight\|none`; sem inferência in-process.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
