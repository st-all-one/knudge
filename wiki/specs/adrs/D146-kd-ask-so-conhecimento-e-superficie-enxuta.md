# D146 — kd ask só conhecimento e superfície enxuta

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D39/D107/D121.

## Decisão

**`kd ask` só conhecimento e superfície enxuta.** Por padrão, só notas sem `scope`; itens de trabalho entram só com `--with-task` (separa `ask` de `task list`). `--with-body` vira `--full-content` (alinha D137); `--container` vira `--scope` (D143/D144); `ask --rank` exige escopo ou `--universe`; `rank`/`tags` migram para `kd knowledge` (que fica `map`/`digest`/`rank`/`tags`), deixando `ask` = recall/get/expand. Revisa D39/D107/D121.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
