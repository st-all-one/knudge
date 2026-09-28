# D140 — Convenção universal do posicional

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Convenção universal do posicional: ele é conteúdo, nunca metadado.** Em `kd write`/`kd task new` o posicional é o **corpo** (Markdown) e a afirmação vira `--summary`; em `kd ask` o posicional é a **consulta**. Identificadores viram `--id` (`show`/`update`/`close`/`forget`); `--body` é removido. Comandos que não criam nem consultam rejeitam posicional (exit 2). O corpo aceita `-` (stdin) e, sem posicional com stdin não-TTY, lê do stdin — cobrindo pipe e heredoc (`<<'EOF'`); sem TTY e sem entrada → erro.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
