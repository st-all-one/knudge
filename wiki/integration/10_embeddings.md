# 10 — Embeddings

Embedding é **opcional e plugável** (D79): o domínio fala com a porta `Embedder`, nunca HTTP. O
índice vetorial e o cache são derivados. Especificação: [`../specs/embeddings.md`](../specs/embeddings.md).

## A porta `Embedder`

```rust
use knudge_core::ports::Embedder;
use knudge_core::embeddings::EmbeddingMeta;

struct MeuProvedor { meta: EmbeddingMeta }

impl Embedder for MeuProvedor {
    fn meta(&self) -> &EmbeddingMeta { &self.meta }
    fn embed(&self, texts: &[String]) -> knudge_core::Result<Vec<Vec<f32>>> {
        // preserve a ordem de entrada; em falha, propague Error (a nota fica `pending`)
        Ok(texts
            .iter()
            .map(|texto| vec![texto.len() as f32; self.meta.dimensions])
            .collect())
    }
}
```

- `meta()` identifica provedor/modelo/dimensão/métrica; **mudar o modelo invalida o índice**.
- `embed` recebe lote e devolve vetores na **mesma ordem**.
- Em falha, retorne `Error`; a nota permanece `pending` e é reprocessada — **nunca** descarte.

### Adapters prontos

```rust
use knudge_core::adapters::HttpEmbedder;          // servidor OpenAI-compatible
use knudge_core::embeddings::LightweightEmbedder;  // determinístico (testes/CI — D89)

let http = HttpEmbedder::new(kd.config(), kd.env())?; // lê [embeddings]
let fake = LightweightEmbedder::new(384)?;
```

Em testes do domínio, use `ports::fakes::FakeEmbedder`.

## Drenar a fila

```rust
use knudge_core::embeddings::{drain, DrainInput};

let out = drain(&DrainInput {
    store: &kd.store(),
    embedder: &embedder,
    config: kd.config(),
    now_ms: kd.now_ms(),
})?;

println!("indexados={} pending={} stale={} cache_hits={}",
         out.indexed, out.pending, out.stale, out.cache_hits);
for aviso in &out.warnings { eprintln!("aviso: {aviso}"); }
```

- `drain` carrega o índice (`EmbeddingIndex::load`), usa o cache, embute o lote e regrava.
- Cache por `(body_hash, modelo)` evita inferência repetida.
- Fila acima de `max_pending` força *catch-up* com aviso (degradação graciosa).

## Índice vetorial

```rust
use knudge_core::embeddings::EmbeddingIndex;

let mut warnings = Vec::new();
let index = EmbeddingIndex::load(kd.fs_dyn(), &kd.knowledge_dir(), embedder.meta(), &mut warnings)?;
// None = ausente/invalidado (modelo/dimensão mudou)
if let Some(index) = index {
    println!("{} vetores", index.len());
    let _ = index.save(kd.fs_dyn(), &kd.knowledge_dir(), &mut warnings)?;
}
```

`EmbeddingIndex::path(root)` aponta para `.idx/embeddings.jsonl` (ou o equivalente no seu layout).

## Canal vetorial no `recall`

```rust
use knudge_core::embeddings::rank_query;

let vetor_query = embedder.embed(&[consulta.clone()])?.remove(0);
let ids = rank_query(&embedding_index, &vetor_query, 50, 0.0); // top_k, min_score
query.vector = Some(ids);                                     // entra na fusão RRF
```

Sem índice, deixe `query.vector = None`: o canal é desligado sem aviso.

## Sugestões semânticas (D158)

```rust
use knudge_core::embeddings::{semantic_suggestions, SuggestionPolicy};

let policy = SuggestionPolicy { duplicate: 0.92, low: 0.4, high: 0.75 };
let sugestoes = semantic_suggestions(&embedding_index, &graph, &anchors_of, &policy, 20);
// propostas de merge/contradição/link — nunca viram aresta sozinhas (D49)
```

## Estado da fila

```rust
use knudge_core::embeddings::{classify, is_backlogged, EmbeddingState};

let estado: EmbeddingState = classify(Some(true));   // indexed/pending/stale
if is_backlogged(pending, max_pending) { /* catch-up */ }
```

## Recomendações

- **Opcional por design.** Se não houver provedor, tudo funciona com o canal lexical + âncoras;
  não acople o serviço a um worker.
- **Uma identidade de modelo.** Fixe `model`+`revision`+`dimensions`; qualquer mudança re-embute
  tudo.
- **Drene off-path.** Rode `drain` num worker/lote, não no `ask`/`write` quente.
- **Cache versionado.** Com `embeddings.version_cache`, o cache vira `.knudge/emb_cache.jsonl`
  versionado (`merge=union`), reduzindo re-embeds.
- **Degradação graciosa.** Timeout/erro do provedor não derruba a escrita: a nota fica `pending`.
- **Não persistir vetores à mão.** Use `EmbeddingIndex::save`/`drain`; `.idx/` é descartável.
