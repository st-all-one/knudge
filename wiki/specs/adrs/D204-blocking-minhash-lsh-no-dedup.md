# D204 — Blocking MinHash/LSH no dedup

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Blocking MinHash/LSH no dedup** (fecha E19-T07). Acima de `MIN_LSH_CORPUS` (256 notas) **e** com vocabulário denso (algum termo em ≥ `DENSE_MIN_DF` docs), `propose_merges` substitui a peneira de postings (E15-T04/O3) por **assinaturas MinHash** (64 permutações, FNV-1a + splitmix64) + *banding* LSH (16 bandas × 4 linhas) — `write/dedup/lsh.rs`, determinístico e sem dep. Em corpus pequeno/esparso a peneira exata (byte-idêntica, `sieve_matches_reference`) segue valendo. LSH é **aproximado**: pares com Jaccard ≥ 0,92 têm probabilidade ~1 de compartilhar banda; os candidatos ainda passam pelo Dice exato e pelo limiar de merge, então as propostas são idênticas ou um superconjunto (recall ≥). `DIVERGENCES.md` #107. A/B: `propose_merges` denso N=1000 **1,42 s → 46 ms (−97 %)**; `compact`/`doctor` N=1000 **−94 %**; esparso inalterado.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #107.
- Fecha E19-T07.
- Ganho medido: −97 %.
- Ganho medido: −94 %.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
