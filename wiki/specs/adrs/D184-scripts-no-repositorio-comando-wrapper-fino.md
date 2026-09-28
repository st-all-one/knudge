# D184 — Scripts no repositório; comando = wrapper fino

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Scripts no repositório; comando = wrapper fino.** `commands/script.rs`: `Source::{Embedded,Local,Remote}`; embutido/local por padrão, `--url` remoto **exige `--sha256`** (SHA-256 verificado antes de executar); `run` faz stream do stderr e captura o stdout, propagando exit. `kd drain service` é wrapper; `scripts/knudge-idle.sh` é a fonte da verdade (sem lógica de agendador em Rust). E18-T01/T03.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
