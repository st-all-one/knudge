# 04 — Busca e recall

O retrieval é **derivado** de `notas/`: você reconstrói o `Index`/`Graph` e chama `recall`/`get`.
Especificação: [`../specs/busca.md`](../specs/busca.md) e [`../specs/matematica.md`](../specs/matematica.md).

## Derivar índice e grafo

```rust
use knudge_core::corpus::Corpus;
use knudge_core::graph::Graph;
use knudge_core::retrieval::Index;

// (a) uma passada, sem tocar `.idx/`
let index = Index::from_store(&store)?;
let graph = Graph::build(&store)?;

// (b) corpus completo (notas + índice + grafo) na mesma leitura
let corpus = Corpus::load(&store)?;
let (notes, index, graph) = (corpus.notes, corpus.index, corpus.graph);

// (c) reusa o índice persistido em `.idx/` se estiver fresco (E15-T11)
let (corpus, warnings) = Corpus::load_fresh(&store, fs, &knowledge_dir)?;
```

> **Otimização-chave:** derive uma vez e **reaproveite**. `Index::build` é O(notas) e o `Corpus`
> coordena a leitura paralela. Reconstruir por query mata a performance (doc 13).

## `recall`

```rust
use knudge_core::retrieval::{RecallQuery, recall};
use knudge_core::retrieval::rank::Universe;
use knudge_core::schema::NoteType;

let mut query = RecallQuery::new("como o parser trata whitespace");
query.limit = 5;                       // 0 = sem limite (default = DEFAULT_LIMIT)
query.universe = Universe::Knowledge;  // Knowledge (só conhecimento) | All
query.filter.types = vec![NoteType::Fact, NoteType::Decision];
query.filter.tags = vec!["toon".into()];
query.working_paths = vec!["src/toon/parse.rs".into()]; // canal de âncoras
query.now_ms = Some(kd.now_ms());      // recência/confiança

let out = recall(&index, &graph, &query)?;
for hit in &out.hits {
    println!("{:>6.3} {} — {} ({:?})", hit.score, hit.id, hit.statement, hit.why);
}
for aviso in &out.warnings { eprintln!("aviso: {aviso}"); }
```

`RecallHit` traz `id`, `statement`, `score` (RRF), `confidence` (derivada), `why` (`Why`) e
`channels` (lexical/anchor/semantic/ppr/recent/stars/body).

| Campo de `RecallQuery` | Para que serve |
|---|---|
| `text` | consulta livre |
| `limit` | teto de hits (`0` = todos) |
| `filter` | `types`, `classifications`, `statuses`, `tags`, `anchors` |
| `scope` | pertencimento via `depends_on` transitivo |
| `working_paths`/`working_ids` | canal de âncoras |
| `vector` | ids do canal vetorial (doc 10) |
| `universe` | `Knowledge` (default) ou `All` |
| `as_of` | conjunto ativo num instante (D155) |
| `drift` | drift de âncoras (doc 09) |
| `strict` | promove `warnings` a erro |

## Corpos: `get`

```rust
use knudge_core::retrieval::get;

let ids: Vec<String> = out.hits.iter().map(|h| h.id.clone()).collect();
let got = get(&store, &ids)?;          // preserva a ordem; ausente vira warning
for note in got.notes { println!("{}", note.body); }
```

## Views e bloqueios

```rust
use knudge_core::retrieval::{compute_views, block_reason};

let views = compute_views(&graph);       // { ready, blocked, ... }
if let Some(reason) = block_reason(&graph, &id) {
    // depends_on não satisfeito / ciclo etc.
}
```

## Ranking sem query, tags e snippets

```rust
use knudge_core::retrieval::{rank, tag_counts, format_brief, format_hit, body_snippet};
use knudge_core::retrieval::rank::RankQuery;

let hits = rank(&index, &graph, &query.filter, &RankQuery::default());
let tags = tag_counts(&index);                       // (tag, contagem), ordenado
let linha = format_brief(&hits[0]);                 // saída curta
let snippet = body_snippet(&note.body, "whitespace", 280);
```

## Canal vetorial (opcional)

```rust
use knudge_core::embeddings::rank_query;

let query_vec = embedder.embed(&[consulta])?.remove(0);
let ids = rank_query(&embedding_index, &query_vec, 50, 0.0);
query.vector = Some(ids);      // entra na fusão RRF como canal `semantic`
```

Sem provedor, deixe `query.vector = None` — o canal é desligado **sem aviso** (degradação
graciosa).

## Recomendações

- **Índice por processo/escopo.** Guarde `Index`/`Graph` imutáveis e faça quantas `recall`
  quiser; invalide quando o corpus mudar.
- **`load_fresh` para persistir.** Se você mantém um serviço, chame `Corpus::load_fresh` para
  reusar `.idx/retrieval.jsonl` entre reinícios.
- **Filtro antes do BM25.** `filter` reduz candidatos; use-o em vez de pós-filtrar hits.
- **`working_paths` > query textual** quando o alvo é código: âncora é determinística.
- **`strict` só quando fizer sentido.** Em serviço, prefira propagar `warnings` a falhar.
- **Assine `Why`/`channels`** para explicar ao usuário; não invente motivo (conjunto fechado — D39).
