# D182 — Reconciliação e probe do worker

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Reconciliação e probe do worker (no script).** `knudge-idle.sh` compara `embeddings.endpoint`/`embeddings.model` efetivos (projeto > global) com o worker instalado (`http://127.0.0.1:$PORT/v1/embeddings`, `ibm-granite/granite-embedding-97m-multilingual-r2`): divergência ⇒ `warn` + comando `kd config set …` exato; a ação **`kd drain service --reconcile`** aplica e reindexa (`kd drain --digest`). `--status` faz probe (`divergente`/`ok`/`fora`) e o wrapper expõe `endpoint` no `--json` (aditivo). O binário só invoca (wrapper fino). Fecha E17-T01/T02.

## Impacto

- Fecha E17-T01/T02.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
