# D150 — O mapa de conhecimento é material e versionado

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**O mapa de conhecimento é material e versionado (A+B+D).** `notas/<tipo>/<id>.md` (path derivável do prefixo do id) + `MAP.md` centralizador (árvore de grupos + clusters, com `--semantic`) + MOC/hub notes (nota real por grupo/cluster, com `references`); tudo **versionado**. Dá ponto de entrada humano e reduz a poluição visual (526 notas planas no `TMP`), sem chave nova. Revisa o layout do store (`note_path`) e o `onboard`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
