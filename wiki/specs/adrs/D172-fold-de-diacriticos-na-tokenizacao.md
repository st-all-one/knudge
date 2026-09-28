# D172 — Fold de diacríticos na tokenização

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Fold de diacríticos na tokenização.** `retrieval/token.rs::tokenize` decompõe não-ASCII em NFD e descarta marcas combinantes, de modo que `café`≡`cafe` (fast-path ASCII emprestado, `Cow`); `content_terms`/`query_terms` herdam e `snippet.rs` busca no texto dobrado, devolvendo o trecho original. O `normalize` do schema **não muda** (D06/D95): `id`/`body_hash` seguem NFC, então `notas/` fica intacto. Casamento é por **termo inteiro** (não prefixo: `lat` não casa `latencia`). O índice derivado ganha o cabeçalho `INDEX_FORMAT` (`retrieval-v2`), que força o rebuild do `.idx/` pré-fold (frescura é por `mtime`). Ganho na bancada: `sem-acento` nDCG@1 8,3 % → 100 %. `DIVERGENCES.md` #96. Fecha E16-T03.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #96.
- Fecha E16-T03.
- Ganho medido: 8,3 % → 100 %.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
