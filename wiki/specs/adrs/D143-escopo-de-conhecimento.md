# D143 — Escopo de conhecimento

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D128/D129.

## Decisão

**Escopo de conhecimento: ponto de partida no `knowledge map` e no `rewind`.** Ambos ganham filtros de corpus (`--tag`/`--anchor`/`--type`/`--class`) e vizinhança (`--around <ID> --depth N`), aplicados **antes** de clusterizar/ranquear; o `map` ganha `--universe` para o projeto inteiro; **operação que varre o corpus exige escopo explícito** — sem filtro, sem `--around` e sem `--universe` → erro (D130). O `--scope` (container) continua. Revisa D128/D129.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
