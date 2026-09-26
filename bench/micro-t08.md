| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 213 ns | 216 ns | 230 ns | 238 ns | 6 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.49 µs | 1.53 µs | 2.54 µs | 2.61 µs | 309 ns |
| micro/fixo | schema::body::body_hash | 25 | 464 ns | 472 ns | 760 ns | 1.36 µs | 184 ns |
| micro/fixo | schema::id::note_id | 25 | 179 ns | 184 ns | 189 ns | 190 ns | 3 ns |
| micro/fixo | schema::hash::short_hash | 25 | 461 ns | 462 ns | 464 ns | 471 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 18 ns | 18 ns | 19 ns | 19 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.65 µs | 1.66 µs | 1.74 µs | 1.78 µs | 29 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 790 ns | 817 ns | 835 ns | 843 ns | 10 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.53 µs | 3.57 µs | 3.65 µs | 4.56 µs | 199 ns |
| micro/fixo | Note::render | 25 | 2.00 µs | 2.03 µs | 2.55 µs | 2.69 µs | 191 ns |
| micro/fixo | jsonl::decode | 25 | 827 ns | 848 ns | 856 ns | 895 ns | 14 ns |
| micro/fixo | jsonl::encode | 25 | 533 ns | 540 ns | 546 ns | 551 ns | 5 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 255 ns | 261 ns | 269 ns | 270 ns | 3 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 2.20 µs | 2.22 µs | 2.25 µs | 2.31 µs | 21 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 115.39 µs | 115.84 µs | 120.86 µs | 123.50 µs | 1.81 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 5 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 41 ns | 41 ns | 43 ns | 50 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.91 µs | 4.01 µs | 4.04 µs | 4.04 µs | 39 ns |
| micro/fixo | config::Config::parse | 25 | 3.15 µs | 3.17 µs | 3.18 µs | 3.18 µs | 8 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 221 ns | 223 ns | 230 ns | 283 ns | 12 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 137 ns | 143 ns | 158 ns | 4 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.34 µs | 29.81 µs | 30.68 µs | 30.79 µs | 419 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 803 ns | 804 ns | 806 ns | 817 ns | 3 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.241 ms | 8.384 ms | 8.507 ms | 8.574 ms | 94.01 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.549 ms | 3.655 ms | 3.764 ms | 4.091 ms | 129.46 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.257 ms | 1.265 ms | 1.316 ms | 1.339 ms | 25.44 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.374 s | 1.410 s | 1.522 s | 1.596 s | 58.372 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.608 ms | 1.625 ms | 1.635 ms | 1.648 ms | 9.84 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.121 ms | 2.141 ms | 2.173 ms | 2.183 ms | 16.97 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.196 ms | 5.245 ms | 5.317 ms | 6.405 ms | 302.06 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 91.77 µs | 92.96 µs | 97.36 µs | 107.00 µs | 3.86 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 6.423 ms | 6.443 ms | 6.514 ms | 6.544 ms | 34.83 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.640 ms | 2.681 ms | 2.790 ms | 3.062 ms | 106.48 µs |
| micro/N=1000 | Graph::integrity | 15 | 138.08 µs | 138.71 µs | 148.13 µs | 148.27 µs | 3.40 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 633.25 µs | 647.78 µs | 657.14 µs | 660.91 µs | 8.67 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 628.15 µs | 645.83 µs | 654.21 µs | 658.40 µs | 9.64 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 22.097 ms | 22.248 ms | 22.363 ms | 23.273 ms | 280.88 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.81 µs | 1.82 µs | 1.88 µs | 1.89 µs | 31 ns |
