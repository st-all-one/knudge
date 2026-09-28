# D138 — kd task plan fica só com --prompt/--submit

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D53/D105.

## Decisão

**`kd task plan` fica só com `--prompt`/`--submit`.** Saem `--adopt`/`--release`/`--review` (apelidos de transição de status; `--review` fechava sem evidência, furando D55) e `--reorder` (`blocks`). Transições de status ficam em `kd task update --status`; o fechamento, em `kd task close` (com evidência). A ordem de irmãos, quando houver, vem do próprio plano submetido. Revisa D53/D105.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
