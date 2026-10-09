# 13 — Performance e funcionamento otimizado

O knudge foi desenhado para **derivar uma vez e reaproveitar**. Este documento é o playbook para
manter a integração rápida e previsível. Régua de medição: `make bench` → `bench/ULTIMO.md`
(observação, E13-T09).

## Princípio: a nota é a verdade, o derivado é cache

`Index`, `Graph`, `.idx/`, cache vetorial e drift são **descartáveis e reconstruíveis**
(D15/D20/D84). Reconstruí-los a cada operação é o erro mais comum. Regra: **uma passada por
invalidação**, não por consulta.

## 1. Leitura única (`Corpus`)

```rust
use knudge_core::corpus::Corpus;

// uma passada: notas + índice + grafo do MESMO vetor
let corpus = Corpus::load(&store)?;

// reusa o índice persistido em `.idx/` se estiver fresco (mtime >= notas)
let (corpus, warnings) = Corpus::load_fresh(&store, kd.fs_dyn(), &kd.knowledge_dir())?;
```

- `Corpus::load` faz a leitura paralela internamente a partir de ~256 notas (até 16 threads);
  abaixo disso, sequencial (o spawn dominaria).
- `from_notes(Vec<Note>)` evita reler quando você já tem as notas.
- **Nunca** chame `Index::from_store` + `Graph::build` separadamente por request: são duas
  passadas.

## 2. Índice persistido

```rust
use knudge_core::retrieval::index::INDEX_WARN_BYTES;
```

- `Corpus::load_fresh` reusa o índice persistido quando ele está **fresco** (`mtime` ≥ todas as
  notas) e o regrava quando reconstrói. A checagem de frescura é interna — você só chama
  `load_fresh`.
- Acima de `INDEX_WARN_BYTES` (8 MiB) o índice emite aviso — considere particionar o corpus.
- Num serviço de vida longa, guarde `Index`/`Graph` e invalide por mtime/quantidade de notas.

## 3. Busca: filtre antes, restrinja canais

```rust
query.filter.types = vec![NoteType::Decision];   // reduz candidatos antes do BM25
query.limit = 5;                                  // default enxuto (D121)
query.working_paths = touched_paths.clone();      // canal de âncoras é determinístico
```

- `limit` menor = menos hits e menos pós-processamento.
- O canal vetorial só entra se `query.vector` for `Some`; não pague embed à toa.
- PPR (`recall.ppr_weight`) custa iteração de potência: deixe `0.0` até medir ganho.
- Filtro de alta frequência (`recall.max_term_ratio`) só vale para corpora ≥ 64 notas.

## 4. Escrita e dedup

- `write` avalia no máximo `dedup::MAX_CANDIDATES` (10) candidatos.
- Acima de `lsh::MIN_LSH_CORPUS` (256 notas) e vocabulário denso, `propose_merges` troca a
  peneira exata por **MinHash/LSH** (D204) — ganho medido de ~97 % em corpus denso.
- Para lotes, use `batch_jsonl` (uma passada) em vez de N `write`.
- Faça read-modify-write sob **lock** só quando houver concorrência real.

## 5. Embeddings

```rust
use knudge_core::embeddings::{drain, DrainInput, is_backlogged};

if is_backlogged(pending, max_pending) {
    // aviso de backpressure; o lote continua limitado a `embeddings.batch` (D215)
}
let out = drain(&DrainInput { store, embedder, config, now_ms })?;
```

- **Cache** por `(body_hash, model)` evita inferência; com `version_cache` o cache é versionado
  (`merge=union`).
- `DEFAULT_FLUSH_MS` (2 s) agrupa gravações do serviço.
- Rode o drain **fora** do caminho de `ask`/`write` (worker/lote).
- `pending` acima de `max_pending` **avisa**; itere `drain` em lotes de `embeddings.batch` (ou use
  `kd drain --digest`, que itera sozinho) — nunca mande a fila inteira numa requisição (D215).

## 6. Handoff com orçamento (D40/D82)

```rust
let request = RewindRequest { budget: 1500, ..RewindRequest::default() };
```

- `rewind` trunca pelo orçamento e descarta sobra < 100 tokens.
- Modo `Manifest` é ~30 tokens; use `Auto` só quando precisar do working set.
- `context_id` permite retomar bytes idênticos sem recomputar (D88).

## 7. Varreduras off-path e limitadas

- `walk_paths`/`walk_paths_ignoring` limitam profundidade (`MAX_WALK_DEPTH=32`) e entradas
  (`MAX_WALK_ENTRIES=20.000`).
- Passe `Project::ignored_dirs()` para não varrer o próprio diretório de conhecimento.
- Persista drift (`DriftStore`) no mesmo walk, e reutilize a lista para todas as notas
  (`compute_anchor_validity_cached`).
- `doctor`/`audit`/manutenção: rode em lote, não por request.

## 8. Concorrência

- `Knudge` não é `Sync`: guarde **dados imutáveis** (`Index`, `Graph`, `Corpus`) compartilhados e
  reabra `Store`/`WriteContext` por thread.
- Escritas concorrentes no mesmo alvo: `store::acquire` (lock advisory) antes do RMW.
- Leitores nunca veem índice parcial: `Staging` faz double-buffer + rename atômico.

## 9. Medição

```sh
make bench          # micro + e2e -> bench/ULTIMO.md
make bench-quick    # smoke
cargo bench         # alvos específicos, quando disponíveis
```

Antes de otimizar, **meça**; a bancada de qualidade (`bench/qualidade.md`) é a régua de busca
(Recall@k/MRR/nDCG@k).

## Checklist de produção

- [ ] `Corpus::load`/`load_fresh` uma vez por invalidação, não por query.
- [ ] `Index`/`Graph` reaproveitados; invalidados por mtime/contagem.
- [ ] Filtros aplicados antes do BM25; `limit` enxuto.
- [ ] Embeddings em worker/lote; cache ligado; `ppr_weight` medido.
- [ ] `rewind` com orçamento; `context_id` para retomar.
- [ ] Varreduras com `ignored_dirs` e limites; drift persistido uma vez.
- [ ] Escrita sob lock quando concorrente; `batch_jsonl` para lotes.
- [ ] `doctor`/`audit`/`prune` off-path.
- [ ] Logs em stderr com `Redactor`; stdout só dados.
- [ ] `warnings` propagados (ou `strict` consciente).
- [ ] Critério de benchmark registrado antes/depois.
