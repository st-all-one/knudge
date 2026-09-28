# Escrita e deduplicação

O protocolo de escrita do knudge: **idempotente por conteúdo**, com dedup em **duas fases** e uma
decisão calibrada entre criar, fundir e rejeitar.

- Código: `crates/knudge-core/src/write/`
- Decisões: D01, D05, D16, D17, D26, D47, D52, D80, D103, D110, D147, D191, D204, D207
- Fórmulas: [`matematica.md`](matematica.md) §9 (MinHash/LSH).
- Bordas: [`DIVERGENCES.md`](DIVERGENCES.md) (dedup, idempotência)

## Idempotência (D01)

O `id` é endereçado por conteúdo (`type + U+001F + normalize(statement)`), então reescrever a
mesma afirmação **não duplica**: o `write` reconhece a nota existente. Se a chave muda, cria novo
`id` + `superseded_by` (linhagem explícita).

## Duas fases (D26/D80)

```
Draft
  → validação de forma (chave/tipo desconhecidos rejeitados; opcionais omitidos)
  → fase 1: recall lexical de candidatos (dedup on-write)
  → decisão calibrada:
        score < 0.75        → cria nota nova
        0.75 ≤ score < 0.92 → merge na existente
        score ≥ 0.92        → rejeita (duplicata)
  → merge/update + evento (op. commit)
  → purge/flush derivados
```

- **Dedup on-write** para notas; **on-read** para `events.jsonl`/containers (D26).
- **Lexical** no write; o **semântico é eventual** (reconciliação) — nunca bloqueia (D80).
- Limiares vêm de `thresholds_from_config` (`dedup.*`).

## Dedup escalável (D204)

`propose_merges` escolhe a estratégia por corpus:

| Condição | Estratégia |
|---|---|
| corpus pequeno/esparso | **peneira exata** de postings (`dedup/sieve.rs`, E15-T04) — byte-idêntica |
| > `MIN_LSH_CORPUS` (256) **e** vocabulário denso | **MinHash + LSH** (`dedup/lsh.rs`, D204) |

- **MinHash** (64 permutações, FNV-1a + splitmix64) + *banding* LSH (16 bandas × 4 linhas).
- LSH é **aproximado**: pares com Jaccard ≥ 0,92 têm probabilidade ~1 de compartilhar banda; os
  candidatos ainda passam pelo **Dice exato** e pelo limiar de merge.
- Consequência: as propostas são **idênticas ou um superconjunto** (recall ≥) — nunca perdem um
  par real. A peneira exata é preservada em corpus pequeno (`sieve_matches_reference` verde).
- Ganho medido: `propose_merges` denso N=1000 **−97 %**; `compact`/`doctor` **−94 %**.

## Merge de campos (D26/D207)

`write/merge.rs` funde campos semânticos numa nota existente sem sobrescrever o corpo: união de
`tags`/`anchors`/`outcomes`, `revision++`, e **claims/proveniência** (D207) sem duplicar. A
`confidence` nunca é armazenada (D142).

## Update e supersede (D01/D21/D48)

- `write::update` aplica um `Patch` versionado; cada update incrementa `revision` (D96).
- `history` lista as versões.
- Supersede é **caminhável**: `replaces` + ponteiro reverso `superseded_by` (D98); a
  bidirecionalidade é validada pela integridade (D46).
- `Patch` aceita `--params '<json>'` (D147): `type/statement/body/tags/anchors/classification/
  status/scope/claims/provenance` + `--clear-anchors`.

## Ciclo de vida soft (D49/D52)

- `link` cria aresta explícita (via única: `kd write --link <FROM:ARESTA:TO>`, D126).
- `forget`/`restore` alternam `status` (soft — nada é apagado; D52).
- Transições permitidas validadas por `status::validate_transition`.

## Outcomes (D103)

`write::outcome` registra `outcomes[]` para **qualquer** nota (status/duration/agent/notes/
recorded_at), emitindo evento `op=outcome`. A **confirmação é derivada** (D87/D189/D190) — o
outcome alimenta a confiança e a retenção.

## Data contract (D191)

O `write` confere **slots mínimos de corpo** por espécie (soft): aviso em `warnings[]`,
`invalid_input` só sob `behavior.strict`, e `--dry-run` expõe `missing_slots`.

## Lote (D110/D141)

`write --batch -` aplica um lote de rascunhos **JSONL** pelo mesmo dedup; item inválido vira
`warnings[]` (R33); `--dry-run` só avalia; teto `write.batch_max` (default 100). Espelha
`task new --batch` (D141).

## Saída

`WriteAction` (`Created`/`Merged`/`Rejected`/`Updated`) + `WriteOutcome`. O `--json` expõe a
ação, o id e os warnings.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Orquestração `write` | `write/mod.rs` |
| Rascunho tipado | `write/draft.rs` |
| Dedup (duas fases) | `write/dedup/{mod,sieve,lsh,merges}.rs` |
| Merge | `write/merge.rs` |
| Update/patch/history | `write/update/{mod,patch}.rs` |
| Ciclo de vida soft | `write/lifecycle.rs` |
| Transições de status | `write/status.rs` |
| Outcomes | `write/outcome.rs` |
| Lote | `write/batch.rs` |

## Testes

`write/tests/` (batch, dedup, idempotent, lifecycle, lsh, outcome, reconcile, strict, update) +
proptest. `DIVERGENCES.md` #107 (LSH).
