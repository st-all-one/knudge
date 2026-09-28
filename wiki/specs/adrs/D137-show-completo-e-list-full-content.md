# D137 — show completo e list --full-content

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**`show` completo e `list --full-content`.** O `show` passa a emitir no **texto** o que hoje só existe no `--json` (corpo, `checks`, âncoras, tags, `outcomes`, `kind`) e o JSON ganha os campos que faltam; `kd task list --full-content` renderiza cada item filtrado como **bloco completo** (reusa o renderer do `show`), separado por `\n---\n`, compondo com `--scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor/--sort impact`. `--full-content` não é pipe-safe (multilinha, como `ask --with-body`); o pipe enxuto segue o default.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
