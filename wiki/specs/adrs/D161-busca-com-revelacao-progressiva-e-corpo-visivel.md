# D161 — Busca com revelação progressiva e corpo visível

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D39/D151.

## Decisão

**Busca com revelação progressiva e corpo visível.** `kd ask` (recall) passa a exibir por padrão: **1º hit com corpo completo**, hits **2–5 com corpo truncado** a `recall.preview_chars` (default 280, config), hits **6+ no padrão** `id\|statement\|score\|why`. `--brief` desliga (2 colunas); `--full-content` mantém todos completos. O `--json` ganha por hit `body_match` (bool), `body_snippet` (trecho que casou, função pura no core com a tokenização ASCII de D36) e `channels.body` (parcela do body no lexical, `[0,1]`, informativo). Revisa D39/D151.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
