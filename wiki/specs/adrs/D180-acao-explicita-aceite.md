# D180 — Ação explícita = aceite

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Ação explícita = aceite.** O worker (`kd drain service`) não pergunta mais: `Action::asks()`/`question()`/`confirm()` e `WatchServiceArgs.yes` saem (D14). `--install`/`--uninstall`/`--subscribe`/`--unsubscribe` executam direto; testes migram para `--dry-run`/`--script` fake. Fecha E17-T04.

## Impacto

- Fecha E17-T04.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
