# D142 — Poda de kd write

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D87.

## Decisão

**Poda de `kd write`: `--checks` e `--confidence` saem.** `checks` é conceito de tarefa (D54) — some de `write` (nunca era executado: `validators::run` só roda no `task close`), é documentado em `task` e entra no `--params`/`--batch` (D141). `confidence` sai por completo (flag + chave; 26→25): a confiança derivada (D87) não a lia e o merge só mantinha o maior — no-op com o default 0.7. `--outcome` permanece com o nome atual (D103; fiel à chave `outcomes` — `--evidence` colidiria com a chave `evidence` dos validators). Revisa D87.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
