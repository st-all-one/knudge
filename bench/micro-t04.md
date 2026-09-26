| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 1.95 µs | 1.97 µs | 3.28 µs | 4.43 µs | 555 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 13.78 µs | 14.04 µs | 14.43 µs | 14.74 µs | 193 ns |
| micro/fixo | schema::body::body_hash | 25 | 2.89 µs | 2.90 µs | 2.94 µs | 2.97 µs | 20 ns |
| micro/fixo | schema::id::note_id | 25 | 1.17 µs | 1.18 µs | 1.22 µs | 1.23 µs | 17 ns |
| micro/fixo | schema::hash::short_hash | 25 | 464 ns | 467 ns | 470 ns | 472 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 37 ns | 38 ns | 39 ns | 39 ns | 1 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 2.06 µs | 2.08 µs | 2.10 µs | 2.15 µs | 17 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 805 ns | 822 ns | 832 ns | 834 ns | 7 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.97 µs | 3.99 µs | 4.04 µs | 4.10 µs | 27 ns |
| micro/fixo | Note::render | 25 | 1.97 µs | 2.01 µs | 2.09 µs | 2.58 µs | 116 ns |
| micro/fixo | jsonl::decode | 25 | 820 ns | 841 ns | 1.27 µs | 1.28 µs | 146 ns |
| micro/fixo | jsonl::encode | 25 | 563 ns | 572 ns | 582 ns | 582 ns | 5 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 234 ns | 240 ns | 324 ns | 392 ns | 35 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 2.28 µs | 2.29 µs | 2.33 µs | 2.40 µs | 25 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 107.66 µs | 108.34 µs | 111.94 µs | 114.76 µs | 1.56 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 4 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 46 ns | 46 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.91 µs | 3.97 µs | 4.23 µs | 5.07 µs | 227 ns |
| micro/fixo | config::Config::parse | 25 | 3.04 µs | 3.06 µs | 3.09 µs | 3.15 µs | 24 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.42 µs | 29.64 µs | 29.91 µs | 30.10 µs | 151 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 803 ns | 806 ns | 829 ns | 831 ns | 9 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.239 ms | 8.768 ms | 10.129 ms | 10.243 ms | 726.19 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.404 ms | 1.655 ms | 2.021 ms | 2.503 ms | 277.82 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.422 s | 1.455 s | 1.504 s | 1.529 s | 26.825 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.593 ms | 1.610 ms | 1.629 ms | 1.639 ms | 12.52 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.002 ms | 2.064 ms | 2.545 ms | 2.601 ms | 215.15 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 4.958 ms | 5.046 ms | 5.520 ms | 5.656 ms | 217.92 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 91.98 µs | 93.38 µs | 104.41 µs | 107.49 µs | 4.73 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 6.602 ms | 6.663 ms | 6.885 ms | 7.068 ms | 123.74 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.669 ms | 2.844 ms | 3.279 ms | 3.990 ms | 326.29 µs |
| micro/N=1000 | Graph::integrity | 15 | 137.24 µs | 138.43 µs | 143.94 µs | 145.62 µs | 2.53 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 601.76 µs | 618.10 µs | 630.18 µs | 633.53 µs | 9.13 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 600.64 µs | 622.50 µs | 636.19 µs | 641.99 µs | 10.93 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.212 ms | 21.574 ms | 22.186 ms | 22.351 ms | 322.54 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 3.56 µs | 3.64 µs | 3.66 µs | 3.72 µs | 43 ns |
