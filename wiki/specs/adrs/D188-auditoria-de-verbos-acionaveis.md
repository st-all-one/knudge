# D188 — Auditoria de verbos acionáveis

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Auditoria de verbos acionáveis.** Permanecem **nativos** `init`/`sync`/`doctor --fix`/`self setup`/`self completions`/`hooks`/`forget`/`prune` (domínio com portas, `--json` e determinismo); só orquestração de SO/rede (`knudge-idle.sh`, `kd-upgrade.sh`, `install.sh`) e utilitários de dev vivem em `scripts/`. Nenhuma migração adicional. Fecha E18-T06.

## Impacto

- Fecha E18-T06.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
