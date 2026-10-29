| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 189 ns | 199 ns | 223 ns | 273 ns | 17 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.36 µs | 1.44 µs | 2.10 µs | 2.63 µs | 288 ns |
| micro/fixo | schema::body::body_hash | 25 | 400 ns | 411 ns | 524 ns | 575 ns | 39 ns |
| micro/fixo | schema::id::note_id | 25 | 158 ns | 167 ns | 178 ns | 195 ns | 7 ns |
| micro/fixo | schema::hash::short_hash | 25 | 427 ns | 431 ns | 506 ns | 520 ns | 23 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 14 ns | 14 ns | 15 ns | 17 ns | 1 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.21 µs | 1.25 µs | 1.34 µs | 1.36 µs | 42 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 582 ns | 607 ns | 654 ns | 920 ns | 65 ns |
| micro/fixo | Note::parse (render completo) | 25 | 2.95 µs | 3.11 µs | 3.20 µs | 3.27 µs | 85 ns |
| micro/fixo | Note::render | 25 | 1.77 µs | 1.84 µs | 1.96 µs | 1.99 µs | 56 ns |
| micro/fixo | jsonl::decode | 25 | 703 ns | 714 ns | 729 ns | 777 ns | 15 ns |
| micro/fixo | jsonl::encode | 25 | 394 ns | 400 ns | 410 ns | 422 ns | 6 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 224 ns | 233 ns | 249 ns | 255 ns | 7 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 1.90 µs | 1.92 µs | 1.96 µs | 1.97 µs | 18 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.54 µs | 1.56 µs | 1.60 µs | 1.61 µs | 21 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 91.36 µs | 91.85 µs | 92.69 µs | 93.49 µs | 487 ns |
| micro/fixo | lifecycle::confidence_score | 25 | 13 ns | 14 ns | 14 ns | 14 ns | 0 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 7 ns | 7 ns | 7 ns | 7 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 30 ns | 32 ns | 36 ns | 43 ns | 3 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 2.56 µs | 2.66 µs | 2.85 µs | 2.89 µs | 91 ns |
| micro/fixo | config::Config::parse | 25 | 2.40 µs | 2.43 µs | 2.45 µs | 2.49 µs | 19 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 129 ns | 131 ns | 133 ns | 133 ns | 1 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 112 ns | 113 ns | 115 ns | 115 ns | 1 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 25.61 µs | 26.12 µs | 27.78 µs | 28.94 µs | 748 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 712 ns | 719 ns | 732 ns | 734 ns | 5 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 6.410 ms | 6.719 ms | 7.159 ms | 7.476 ms | 291.49 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 2.598 ms | 2.658 ms | 2.698 ms | 2.702 ms | 37.97 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 938.18 µs | 963.74 µs | 997.47 µs | 1.011 ms | 18.42 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 26.571 ms | 27.281 ms | 27.668 ms | 28.680 ms | 479.07 µs |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.233 ms | 1.264 ms | 1.280 ms | 1.285 ms | 16.56 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 1.408 ms | 1.456 ms | 1.596 ms | 1.632 ms | 65.50 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 2.659 ms | 2.714 ms | 2.750 ms | 2.760 ms | 27.17 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 94.42 µs | 100.71 µs | 101.06 µs | 111.40 µs | 3.39 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 691.08 µs | 718.39 µs | 737.87 µs | 792.49 µs | 23.34 µs |
| micro/N=1000 | lifecycle::communities | 15 | 3.559 ms | 3.663 ms | 3.727 ms | 3.730 ms | 46.52 µs |
| micro/N=1000 | Graph::from_notes | 15 | 1.617 ms | 1.680 ms | 1.802 ms | 1.902 ms | 79.05 µs |
| micro/N=1000 | graph::pagerank | 15 | 404.10 µs | 425.54 µs | 475.06 µs | 475.62 µs | 22.51 µs |
| micro/N=1000 | retrieval::recall (ppr) | 15 | 1.977 ms | 2.028 ms | 2.067 ms | 2.071 ms | 29.17 µs |
| micro/N=1000 | Graph::integrity | 15 | 106.51 µs | 110.63 µs | 123.90 µs | 138.15 µs | 8.34 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 476.67 µs | 501.88 µs | 534.36 µs | 536.38 µs | 17.33 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 477.71 µs | 508.79 µs | 534.43 µs | 538.83 µs | 19.81 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 612.02 µs | 641.77 µs | 667.05 µs | 686.89 µs | 20.33 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 100.15 µs | 103.58 µs | 112.72 µs | 118.03 µs | 4.98 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 99.25 µs | 113.00 µs | 115.38 µs | 116.92 µs | 5.70 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 143.52 µs | 155.89 µs | 163.71 µs | 179.49 µs | 7.70 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 16.583 ms | 16.873 ms | 17.346 ms | 17.374 ms | 236.62 µs |
| micro/N=1000 | Index::serialize | 15 | 6.464 ms | 6.546 ms | 6.582 ms | 6.725 ms | 57.70 µs |
| micro/N=1000 | Index::parse | 15 | 9.531 ms | 9.644 ms | 9.773 ms | 9.896 ms | 94.40 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.61 µs | 1.67 µs | 1.75 µs | 1.78 µs | 51 ns |
| micro/ai | corpus::parse_jsonl (shard N=1000) | 3 | 4.463 ms | 4.550 ms | 4.579 ms | 4.579 ms | 60.39 µs |
| micro/ai-legacy | upsert linear (referência N=1000) | 3 | 20.980 ms | 20.994 ms | 21.058 ms | 21.058 ms | 41.95 µs |
| micro/ai | corpus::parse_jsonl (shard N=10000) | 3 | 54.570 ms | 55.215 ms | 56.264 ms | 56.264 ms | 855.04 µs |
| micro/ai-legacy | upsert linear (referência N=10000) | 3 | 2.342 s | 2.479 s | 2.588 s | 2.588 s | 122.791 ms |
| micro/ai | corpus::parse_jsonl (shard N=100000) | 3 | 622.745 ms | 626.008 ms | 635.451 ms | 635.451 ms | 6.598 ms |
