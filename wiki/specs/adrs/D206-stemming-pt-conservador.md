# D206 — Stemming PT conservador

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D36/D172/D173.

## Decisão

**Stemming PT conservador** (fecha E16-T11). O canal **lexical** (índice + consulta) corta sufixos flexionais/derivacionais do PT-BR sobre o token dobrado (D172), com radical mínimo de 4 bytes e **normalização de plural antes do corte derivacional** (`decoder`/`decoders` convergem) — `retrieval/stem.rs`. O `normalize` do schema (`id`/`body_hash`, D06/D95) **não muda**; o snippet segue com termos crus (`content_terms`) e o índice persiste o radical (`INDEX_FORMAT` → `retrieval-v4`). Adotado por medição (R43): **+25 % de nDCG@5** na bancada de qualidade (`bench/t11_stemming.md`), família `morfologia` (consulta plural × nota singular) de 0 % para 100 %, sem regressão de família. Revisa D36/D172/D173. `DIVERGENCES.md` #109.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #109.
- Fecha E16-T11.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
