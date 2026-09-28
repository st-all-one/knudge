# D144 — Escopo obrigatório em learn/compact/prune e task list

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Escopo obrigatório em `learn`/`compact`/`prune` e `task list`.** Aplica D143: `maintenance learn`/`compact`/`prune` exigem `--tag`/`--anchor`/`--type`/`--class`/`--scope` ou `--universe`; `kd task list` exige ao menos um filtro (`--scope`/`--status`/`--kind`/`--parent`/`--ready`/`--blocked`/`--tag`/`--anchor`) ou `--universe` (panorama geral). `--sort impact`/`--explain` não contam como escopo. Sem escopo → exit 2 (D130).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
