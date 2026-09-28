# D186 — watch-service sob drain; maintenance só revisão

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D170.

## Decisão

**`watch-service` sob `drain`; `maintenance` só revisão.** `kd drain service <ação>` (`--install`/`--subscribe`/`--unsubscribe`/`--status`/`--uninstall`); `kd maintenance watch-service` ⇒ uso (2); `maintenance` = `compact`/`learn`/`prune`. `DrainArgs` ganha o subcomando opcional `service` (`args_conflicts_with_subcommands`). Revisa D170. Fecha E18-T04.

## Impacto

- Fecha E18-T04.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
