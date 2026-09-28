# D130 — Verbos falham alto, nunca em silêncio

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Verbos falham alto, nunca em silêncio.** `kd ask` sem modo devolve o uso (exit 2) em vez de sair vazio; `kd write`/`kd task new` sem `statement` é `invalid_input` (2) — antes criavam nota/tarefa com `statement: ""`; nota ausente é `not_found` (3), ativando a degradação que o `get` já previa (era `io`=5). Nenhuma entrada vazia grava lixo nem retorna vazio sem explicação.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
