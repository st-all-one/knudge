| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 182 ns | 190 ns | 200 ns | 218 ns | 8 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.27 µs | 1.31 µs | 1.37 µs | 1.37 µs | 27 ns |
| micro/fixo | schema::body::body_hash | 25 | 432 ns | 445 ns | 456 ns | 458 ns | 5 ns |
| micro/fixo | schema::id::note_id | 25 | 176 ns | 179 ns | 185 ns | 186 ns | 3 ns |
| micro/fixo | schema::hash::short_hash | 25 | 458 ns | 460 ns | 462 ns | 463 ns | 1 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 18 ns | 18 ns | 19 ns | 19 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.66 µs | 1.69 µs | 1.69 µs | 1.69 µs | 8 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 816 ns | 834 ns | 846 ns | 856 ns | 10 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.51 µs | 3.57 µs | 3.61 µs | 3.93 µs | 75 ns |
| micro/fixo | Note::render | 25 | 2.07 µs | 2.08 µs | 2.11 µs | 2.11 µs | 11 ns |
| micro/fixo | jsonl::decode | 25 | 822 ns | 827 ns | 837 ns | 841 ns | 5 ns |
| micro/fixo | jsonl::encode | 25 | 523 ns | 539 ns | 547 ns | 549 ns | 6 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 217 ns | 224 ns | 234 ns | 238 ns | 6 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.89 µs | 1.91 µs | 1.92 µs | 1.93 µs | 7 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 117.52 µs | 118.08 µs | 119.11 µs | 123.09 µs | 1.08 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 5 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 46 ns | 46 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.84 µs | 3.92 µs | 4.21 µs | 4.48 µs | 141 ns |
| micro/fixo | config::Config::parse | 25 | 3.13 µs | 3.17 µs | 3.93 µs | 4.11 µs | 297 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 233 ns | 237 ns | 241 ns | 243 ns | 2 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 145 ns | 148 ns | 151 ns | 152 ns | 2 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.23 µs | 29.46 µs | 32.25 µs | 33.21 µs | 1.09 µs |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 802 ns | 804 ns | 824 ns | 828 ns | 8 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.372 ms | 8.615 ms | 9.613 ms | 9.774 ms | 487.61 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.676 ms | 3.890 ms | 4.211 ms | 4.250 ms | 182.87 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.328 ms | 1.386 ms | 1.481 ms | 1.508 ms | 53.09 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.425 s | 1.692 s | 1.779 s | 1.809 s | 122.769 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.489 ms | 1.512 ms | 1.804 ms | 2.671 ms | 302.99 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.107 ms | 2.154 ms | 2.680 ms | 2.728 ms | 212.24 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.251 ms | 5.818 ms | 6.088 ms | 7.734 ms | 581.99 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 92.47 µs | 93.94 µs | 115.10 µs | 115.31 µs | 7.71 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 900.47 µs | 940.07 µs | 1.123 ms | 2.027 ms | 283.59 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.653 ms | 2.768 ms | 4.167 ms | 4.219 ms | 507.26 µs |
| micro/N=1000 | Graph::integrity | 15 | 133.82 µs | 138.56 µs | 151.63 µs | 153.30 µs | 6.93 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 601.75 µs | 625.22 µs | 642.12 µs | 644.85 µs | 13.28 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 596.52 µs | 626.48 µs | 635.14 µs | 664.47 µs | 16.73 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 777.75 µs | 805.13 µs | 911.50 µs | 920.09 µs | 44.04 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 146.39 µs | 149.60 µs | 162.24 µs | 168.95 µs | 6.39 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 142.82 µs | 147.65 µs | 163.08 µs | 192.62 µs | 12.71 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.452 ms | 22.584 ms | 23.366 ms | 24.714 ms | 795.43 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.79 µs | 1.84 µs | 3.37 µs | 4.76 µs | 829 ns |
