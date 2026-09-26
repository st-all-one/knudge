| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 216 ns | 219 ns | 235 ns | 239 ns | 6 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.50 µs | 1.52 µs | 1.55 µs | 1.58 µs | 19 ns |
| micro/fixo | schema::body::body_hash | 25 | 482 ns | 488 ns | 493 ns | 500 ns | 3 ns |
| micro/fixo | schema::id::note_id | 25 | 176 ns | 178 ns | 186 ns | 186 ns | 3 ns |
| micro/fixo | schema::hash::short_hash | 25 | 457 ns | 460 ns | 464 ns | 466 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 17 ns | 18 ns | 18 ns | 19 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.64 µs | 1.66 µs | 1.67 µs | 1.69 µs | 11 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 763 ns | 781 ns | 789 ns | 799 ns | 9 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.58 µs | 3.60 µs | 3.62 µs | 3.63 µs | 11 ns |
| micro/fixo | Note::render | 25 | 1.98 µs | 1.99 µs | 2.01 µs | 2.02 µs | 10 ns |
| micro/fixo | jsonl::decode | 25 | 798 ns | 804 ns | 809 ns | 812 ns | 3 ns |
| micro/fixo | jsonl::encode | 25 | 556 ns | 563 ns | 578 ns | 588 ns | 7 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 211 ns | 217 ns | 227 ns | 228 ns | 5 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 2.11 µs | 2.13 µs | 2.16 µs | 2.17 µs | 13 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 116.48 µs | 116.94 µs | 123.50 µs | 124.11 µs | 2.40 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 5 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 45 ns | 46 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.89 µs | 3.98 µs | 4.15 µs | 4.21 µs | 78 ns |
| micro/fixo | config::Config::parse | 25 | 3.11 µs | 3.12 µs | 3.13 µs | 3.13 µs | 6 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 224 ns | 225 ns | 229 ns | 230 ns | 2 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 138 ns | 140 ns | 141 ns | 1 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.30 µs | 29.51 µs | 29.60 µs | 30.30 µs | 195 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 801 ns | 803 ns | 812 ns | 813 ns | 3 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.149 ms | 8.207 ms | 8.625 ms | 8.655 ms | 163.45 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.586 ms | 3.664 ms | 3.794 ms | 3.829 ms | 73.18 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.252 ms | 1.268 ms | 1.279 ms | 1.292 ms | 9.80 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.391 s | 1.439 s | 1.470 s | 1.487 s | 28.514 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.619 ms | 1.647 ms | 1.774 ms | 1.885 ms | 70.87 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.036 ms | 2.075 ms | 2.309 ms | 2.387 ms | 111.04 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.152 ms | 5.485 ms | 5.942 ms | 5.989 ms | 273.70 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 93.03 µs | 94.22 µs | 97.15 µs | 109.86 µs | 4.09 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 877.21 µs | 889.99 µs | 947.82 µs | 988.33 µs | 29.92 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.607 ms | 2.696 ms | 3.007 ms | 3.039 ms | 145.60 µs |
| micro/N=1000 | Graph::integrity | 15 | 134.10 µs | 137.24 µs | 145.48 µs | 148.27 µs | 3.65 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 603.50 µs | 614.33 µs | 625.08 µs | 626.34 µs | 7.07 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 598.40 µs | 618.10 µs | 625.92 µs | 648.62 µs | 11.54 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 778.45 µs | 795.50 µs | 799.13 µs | 801.22 µs | 7.52 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 148.48 µs | 153.37 µs | 165.94 µs | 167.41 µs | 5.57 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 145.06 µs | 150.09 µs | 159.87 µs | 162.24 µs | 4.42 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 20.997 ms | 21.342 ms | 21.657 ms | 22.146 ms | 286.77 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.75 µs | 1.79 µs | 1.81 µs | 1.86 µs | 30 ns |
