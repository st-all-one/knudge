# D181 — Stream do worker

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Stream do worker.** O `knudge-idle.sh` cobre cada passo via `log()` (stderr); o wrapper (`commands/script.rs`) faz **stream do stderr** e mantém stdout como dados (R20). Entregue por E18-T01; prova em `cli::drain_service_streams_stderr_and_keeps_stdout_as_data`. Fecha E17-T03.

## Impacto

- Fecha E17-T03.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
