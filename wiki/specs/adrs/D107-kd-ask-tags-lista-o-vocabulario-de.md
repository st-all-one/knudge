# D107 — `kd ask --tags` lista o vocabulário de tags

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

`kd ask --tags` lista o vocabulário de tags (`tag\|count`, `count` desc, `tag` asc), ignorando `forgotten`/`superseded`. `kd ask --rank` ranqueia por confiança derivada sem query (`id\|statement\|confidence\|why`), no universo conhecimento (`scope` ausente), ordem `(confidence desc, id asc)`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
