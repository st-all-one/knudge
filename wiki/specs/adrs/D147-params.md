# D147 — --params '{

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Generaliza D141; Estende D140.

## Decisão

**`--params '{...}'` universal e stdin/heredoc universal.** `write`, `ask` e `task new` aceitam `--params '{json}'` (objeto com os campos do comando; `--params -` lê de stdin) — o conjunto completo de uma vez, útil para scripts/MCP. O conteúdo por stdin/heredoc (posicional `-`; sem posicional e stdin não-TTY) vira **universal** do `kd` (estende D140): `write` (corpo), `ask` (consulta), etc. `--params` e o posicional `-` são vias exclusivas. Generaliza D141.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
