# D131 — Auto-drain ocioso no lazy; sem eager

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Auto-drain ocioso no `lazy`; sem `eager`.** `embeddings.mode` passa a aceitar só `lazy` (default) e `manual` — `eager` é rejeitado (`config`=7). Em `lazy`, ao fim de cada invocação não-`maintenance`, o CLI drena **um lote** (`embeddings.batch`) *best-effort*, **depois** de emitir a saída; nunca altera exit code nem `warnings[]` (não interage com `strict`). O `--drain` explícito continua valendo nos dois modos (coexistem). `KNUDGE_NO_IDLE` desliga o caminho ocioso. O worker contínuo (timer/systemd) fica fora do binário, via `scripts/knudge-idle.sh`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
