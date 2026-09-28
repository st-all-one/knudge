# D183 — Supply-chain do worker

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Supply-chain do worker.** O GGUF vem de uma **revisão pinada** (`MODEL_REVISION`, nunca `/resolve/main/`) e é verificado por **SHA-256** (`MODEL_SHA256`); o instalador oficial do llama.cpp é baixado e verificado por SHA-256 (`LLAMA_INSTALL_SHA256`) **antes** de executar (nunca `curl … \| sh` cego), com fallback para gestor de pacotes. `file://` habilita mirror local/air-gapped. Fecha E17-T05.

## Impacto

- Fecha E17-T05.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
