# D155 — Consulta temporal kd ask --as-of <TS>

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D143/D146.

## Decisão

**Consulta temporal `kd ask --as-of <TS>`.** Reconstrói o conjunto **ativo em `T`** a partir do log de eventos (`forget`/`restore` + `link replaces`, D46/D52) e roda o pipeline determinístico (BM25/RRF) sobre o subconjunto — o ranking de `T` é reproduzível (o `ai-memory`, com FTS, não reproduzia). Nota purgada vira `warnings[]` (R33); `T` no futuro é `invalid_input` (2); `T` sem eventos é `[no_results]` (D152). O estado reconstruído é soberano sobre o filtro de status default (D43). `--json` ganha `as_of` e `historical`; o `why` (D39) não muda. Revisa D143/D146.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
