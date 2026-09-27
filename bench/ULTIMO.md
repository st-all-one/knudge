| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 241 ns | 247 ns | 283 ns | 339 ns | 20 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.78 µs | 1.82 µs | 1.83 µs | 1.83 µs | 13 ns |
| micro/fixo | schema::body::body_hash | 25 | 524 ns | 531 ns | 559 ns | 589 ns | 13 ns |
| micro/fixo | schema::id::note_id | 25 | 201 ns | 207 ns | 211 ns | 219 ns | 4 ns |
| micro/fixo | schema::hash::short_hash | 25 | 459 ns | 462 ns | 468 ns | 477 ns | 4 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 17 ns | 17 ns | 18 ns | 18 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.55 µs | 1.57 µs | 1.75 µs | 1.77 µs | 54 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 843 ns | 861 ns | 866 ns | 876 ns | 7 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.45 µs | 3.50 µs | 3.61 µs | 4.41 µs | 185 ns |
| micro/fixo | Note::render | 25 | 2.07 µs | 2.09 µs | 2.74 µs | 3.16 µs | 256 ns |
| micro/fixo | jsonl::decode | 25 | 798 ns | 811 ns | 822 ns | 827 ns | 7 ns |
| micro/fixo | jsonl::encode | 25 | 533 ns | 542 ns | 555 ns | 568 ns | 7 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 235 ns | 243 ns | 261 ns | 268 ns | 9 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 2.55 µs | 2.58 µs | 2.60 µs | 2.61 µs | 17 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.94 µs | 1.95 µs | 1.97 µs | 1.98 µs | 10 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 115.76 µs | 116.52 µs | 125.85 µs | 130.30 µs | 3.31 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 15 ns | 16 ns | 16 ns | 17 ns | 0 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 7 ns | 7 ns | 8 ns | 8 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 41 ns | 45 ns | 46 ns | 1 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.72 µs | 3.79 µs | 3.87 µs | 3.91 µs | 45 ns |
| micro/fixo | config::Config::parse | 25 | 3.11 µs | 3.15 µs | 3.17 µs | 3.39 µs | 51 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 231 ns | 234 ns | 240 ns | 251 ns | 4 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 137 ns | 140 ns | 140 ns | 1 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.44 µs | 29.70 µs | 30.65 µs | 32.11 µs | 540 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 805 ns | 806 ns | 813 ns | 815 ns | 3 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.270 ms | 1.292 ms | 1.314 ms | 1.336 ms | 16.03 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 561.67 µs | 574.80 µs | 598.19 µs | 603.22 µs | 12.51 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 239.35 µs | 244.79 µs | 254.71 µs | 254.78 µs | 4.56 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 51.285 ms | 51.757 ms | 53.887 ms | 57.528 ms | 1.587 ms |
| micro/N=200 | write::propose_merges (esparso) | 15 | 266.79 µs | 273.29 µs | 298.78 µs | 350.95 µs | 21.33 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 347.25 µs | 352.07 µs | 367.02 µs | 369.74 µs | 7.54 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 706.93 µs | 722.23 µs | 737.18 µs | 751.98 µs | 12.18 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 20.88 µs | 21.02 µs | 21.86 µs | 22.28 µs | 392 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 145.48 µs | 151.77 µs | 163.29 µs | 166.71 µs | 5.46 µs |
| micro/N=200 | lifecycle::communities | 15 | 730.26 µs | 752.26 µs | 768.82 µs | 786.35 µs | 14.33 µs |
| micro/N=200 | Graph::from_notes | 15 | 362.83 µs | 371.56 µs | 390.34 µs | 406.97 µs | 11.75 µs |
| micro/N=200 | graph::pagerank | 15 | 84.79 µs | 85.83 µs | 88.49 µs | 95.75 µs | 2.65 µs |
| micro/N=200 | retrieval::recall (ppr) | 15 | 507.19 µs | 517.38 µs | 527.02 µs | 527.58 µs | 7.39 µs |
| micro/N=200 | Graph::integrity | 15 | 18.30 µs | 18.79 µs | 19.56 µs | 21.58 µs | 785 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 107.84 µs | 113.98 µs | 117.40 µs | 123.83 µs | 3.28 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 108.33 µs | 113.28 µs | 115.03 µs | 118.87 µs | 2.31 µs |
| micro/N=200 | retrieval::compute_views | 15 | 131.09 µs | 137.31 µs | 143.73 µs | 144.15 µs | 3.76 µs |
| micro/N=200 | task::impact (1 id) | 15 | 16.55 µs | 18.02 µs | 18.72 µs | 19.07 µs | 657 ns |
| micro/N=200 | task::impacts (todos) | 15 | 16.76 µs | 17.67 µs | 23.40 µs | 28.15 µs | 3.04 µs |
| micro/N=200 | handoff::rank (manifest) | 15 | 28.43 µs | 29.54 µs | 34.85 µs | 53.99 µs | 6.40 µs |
| micro/N=200 | Index::serialize + parse | 15 | 3.968 ms | 4.138 ms | 4.434 ms | 4.874 ms | 215.46 µs |
| micro/N=200 | Index::serialize | 15 | 1.583 ms | 1.599 ms | 1.834 ms | 2.207 ms | 164.15 µs |
| micro/N=200 | Index::parse | 15 | 2.357 ms | 2.373 ms | 2.382 ms | 2.385 ms | 9.00 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.88 µs | 1.91 µs | 1.97 µs | 1.99 µs | 37 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.209 ms | 8.638 ms | 9.266 ms | 10.238 ms | 536.80 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.590 ms | 3.748 ms | 4.192 ms | 4.223 ms | 229.49 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.419 ms | 1.518 ms | 1.692 ms | 1.753 ms | 108.27 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 46.189 ms | 46.820 ms | 48.435 ms | 56.318 ms | 2.471 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.519 ms | 1.535 ms | 1.630 ms | 1.643 ms | 39.26 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.174 ms | 2.240 ms | 2.936 ms | 3.432 ms | 350.91 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 4.205 ms | 4.340 ms | 6.625 ms | 7.039 ms | 883.02 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 110.07 µs | 114.82 µs | 123.55 µs | 125.58 µs | 4.02 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 882.38 µs | 893.48 µs | 900.96 µs | 901.93 µs | 5.80 µs |
| micro/N=1000 | lifecycle::communities | 15 | 4.377 ms | 4.463 ms | 4.833 ms | 5.303 ms | 246.36 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.630 ms | 2.691 ms | 3.148 ms | 3.214 ms | 185.29 µs |
| micro/N=1000 | graph::pagerank | 15 | 499.16 µs | 512.85 µs | 524.65 µs | 536.31 µs | 9.94 µs |
| micro/N=1000 | retrieval::recall (ppr) | 15 | 3.121 ms | 3.269 ms | 3.493 ms | 3.503 ms | 137.97 µs |
| micro/N=1000 | Graph::integrity | 15 | 138.01 µs | 142.90 µs | 150.51 µs | 155.82 µs | 4.17 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 604.34 µs | 620.19 µs | 634.72 µs | 652.32 µs | 13.93 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 611.04 µs | 623.96 µs | 631.30 µs | 653.44 µs | 10.50 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 786.55 µs | 803.80 µs | 833.21 µs | 850.11 µs | 16.52 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 152.04 µs | 155.47 µs | 161.54 µs | 180.75 µs | 6.75 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 149.60 µs | 149.95 µs | 157.00 µs | 166.08 µs | 4.57 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 176.42 µs | 180.89 µs | 188.43 µs | 194.93 µs | 4.82 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.733 ms | 22.676 ms | 24.842 ms | 25.492 ms | 1.101 ms |
| micro/N=1000 | Index::serialize | 15 | 8.514 ms | 8.655 ms | 9.114 ms | 9.837 ms | 346.63 µs |
| micro/N=1000 | Index::parse | 15 | 12.994 ms | 13.458 ms | 13.857 ms | 13.953 ms | 308.89 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.83 µs | 1.88 µs | 1.98 µs | 1.99 µs | 55 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 2.407 ms | 3.500 ms | 3.942 ms | 3.942 ms | 644.18 µs |
| e2e/N=200 | self version --json | 8 | 2.202 ms | 3.623 ms | 4.035 ms | 4.035 ms | 793.46 µs |
| e2e/N=200 | --help | 8 | 2.283 ms | 3.050 ms | 3.900 ms | 3.900 ms | 562.43 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 3.942 ms | 3.957 ms | 4.296 ms | 4.296 ms | 126.28 µs |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.439 ms | 5.535 ms | 5.761 ms | 5.761 ms | 106.72 µs |
| e2e/N=200 | prime | 8 | 2.172 ms | 3.350 ms | 4.035 ms | 4.035 ms | 669.04 µs |
| e2e/N=200 | prime --long | 8 | 2.015 ms | 3.624 ms | 4.417 ms | 4.417 ms | 792.74 µs |
| e2e/N=200 | ask (query comum) | 8 | 36.489 ms | 41.680 ms | 53.513 ms | 53.513 ms | 6.796 ms |
| e2e/N=200 | ask (query rara) | 8 | 30.384 ms | 39.798 ms | 53.777 ms | 53.777 ms | 8.517 ms |
| e2e/N=200 | ask --json | 8 | 33.453 ms | 38.981 ms | 56.017 ms | 56.017 ms | 7.829 ms |
| e2e/N=200 | ask --limit 50 | 8 | 33.200 ms | 47.190 ms | 63.318 ms | 63.318 ms | 10.393 ms |
| e2e/N=200 | ask --brief | 8 | 31.950 ms | 43.002 ms | 57.638 ms | 57.638 ms | 9.640 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 32.303 ms | 36.456 ms | 40.774 ms | 40.774 ms | 3.105 ms |
| e2e/N=200 | ask --around <nota> | 8 | 33.044 ms | 40.568 ms | 47.611 ms | 47.611 ms | 5.753 ms |
| e2e/N=200 | ask --id <nota> | 8 | 26.671 ms | 30.767 ms | 42.339 ms | 42.339 ms | 4.846 ms |
| e2e/N=200 | rewind | 8 | 39.850 ms | 49.996 ms | 55.280 ms | 55.280 ms | 6.119 ms |
| e2e/N=200 | rewind --json | 8 | 38.721 ms | 50.388 ms | 65.115 ms | 65.115 ms | 8.938 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 40.496 ms | 49.497 ms | 59.464 ms | 59.464 ms | 5.973 ms |
| e2e/N=200 | task list --universe | 8 | 37.520 ms | 44.941 ms | 53.804 ms | 53.804 ms | 6.289 ms |
| e2e/N=200 | task list --ready | 8 | 29.109 ms | 35.990 ms | 49.378 ms | 49.378 ms | 6.007 ms |
| e2e/N=200 | task list --sort impact | 8 | 29.603 ms | 38.146 ms | 55.689 ms | 55.689 ms | 7.956 ms |
| e2e/N=200 | task list --full-content | 8 | 31.917 ms | 42.211 ms | 52.836 ms | 52.836 ms | 7.732 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 28.118 ms | 33.282 ms | 38.672 ms | 38.672 ms | 3.635 ms |
| e2e/N=200 | task graph | 8 | 28.898 ms | 37.553 ms | 48.712 ms | 48.712 ms | 6.640 ms |
| e2e/N=200 | knowledge map --universe | 8 | 50.841 ms | 57.926 ms | 73.783 ms | 73.783 ms | 8.704 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 39.432 ms | 50.745 ms | 60.731 ms | 60.731 ms | 7.137 ms |
| e2e/N=200 | knowledge tags | 8 | 35.133 ms | 43.649 ms | 53.046 ms | 53.046 ms | 6.484 ms |
| e2e/N=200 | drain --status | 8 | 16.346 ms | 19.184 ms | 31.349 ms | 31.349 ms | 5.378 ms |
| e2e/N=200 | doctor | 8 | 159.223 ms | 176.452 ms | 182.285 ms | 182.285 ms | 9.188 ms |
| e2e/N=200 | doctor --explain | 8 | 158.742 ms | 172.738 ms | 179.820 ms | 179.820 ms | 7.436 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 26.380 ms | 30.736 ms | 46.715 ms | 46.715 ms | 8.515 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 85.742 ms | 93.362 ms | 105.113 ms | 105.113 ms | 8.100 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 19.335 ms | 22.077 ms | 23.925 ms | 23.925 ms | 1.694 ms |
| e2e/N=200 | config list | 8 | 26.871 ms | 32.466 ms | 35.789 ms | 35.789 ms | 2.843 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 23.144 ms | 27.528 ms | 31.096 ms | 31.096 ms | 2.485 ms |
| e2e/N=200 | write (nova) | 8 | 32.230 ms | 44.969 ms | 64.892 ms | 64.892 ms | 11.408 ms |
| e2e/N=200 | write (idempotente) | 8 | 32.367 ms | 37.231 ms | 46.687 ms | 46.687 ms | 4.313 ms |
| e2e/N=200 | config set (projeto) | 8 | 28.966 ms | 33.500 ms | 42.022 ms | 42.022 ms | 4.431 ms |
| e2e/N=200 | forget (soft) | 8 | 32.538 ms | 41.655 ms | 60.885 ms | 60.885 ms | 9.782 ms |
| e2e/N=200 | forget --restore | 8 | 30.160 ms | 32.786 ms | 47.616 ms | 47.616 ms | 7.432 ms |
| e2e/N=200 | sync | 8 | 28.175 ms | 37.802 ms | 44.990 ms | 44.990 ms | 5.900 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 1.684 ms | 3.323 ms | 4.334 ms | 4.334 ms | 986.95 µs |
| e2e/N=1000 | self version --json | 8 | 1.782 ms | 3.899 ms | 4.393 ms | 4.393 ms | 1.149 ms |
| e2e/N=1000 | --help | 8 | 1.796 ms | 3.225 ms | 4.020 ms | 4.020 ms | 774.54 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 8.296 ms | 8.774 ms | 9.555 ms | 9.555 ms | 360.29 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 18.684 ms | 19.500 ms | 19.803 ms | 19.803 ms | 398.45 µs |
| e2e/N=1000 | prime | 8 | 1.773 ms | 2.338 ms | 3.360 ms | 3.360 ms | 564.46 µs |
| e2e/N=1000 | prime --long | 8 | 1.719 ms | 2.394 ms | 3.998 ms | 3.998 ms | 956.23 µs |
| e2e/N=1000 | ask (query comum) | 8 | 71.480 ms | 84.709 ms | 89.064 ms | 89.064 ms | 5.711 ms |
| e2e/N=1000 | ask (query rara) | 8 | 74.152 ms | 86.563 ms | 92.869 ms | 92.869 ms | 6.496 ms |
| e2e/N=1000 | ask --json | 8 | 69.695 ms | 79.056 ms | 94.374 ms | 94.374 ms | 7.758 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 74.442 ms | 80.011 ms | 84.138 ms | 84.138 ms | 3.108 ms |
| e2e/N=1000 | ask --brief | 8 | 74.697 ms | 80.893 ms | 85.573 ms | 85.573 ms | 4.421 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 70.974 ms | 79.697 ms | 92.995 ms | 92.995 ms | 6.452 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 71.583 ms | 86.938 ms | 97.331 ms | 97.331 ms | 9.915 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 53.380 ms | 56.010 ms | 78.311 ms | 78.311 ms | 10.888 ms |
| e2e/N=1000 | rewind | 8 | 80.344 ms | 94.289 ms | 103.769 ms | 103.769 ms | 8.323 ms |
| e2e/N=1000 | rewind --json | 8 | 85.119 ms | 98.149 ms | 117.530 ms | 117.530 ms | 10.770 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 80.317 ms | 95.955 ms | 99.167 ms | 99.167 ms | 6.164 ms |
| e2e/N=1000 | task list --universe | 8 | 59.592 ms | 63.816 ms | 79.669 ms | 79.669 ms | 6.611 ms |
| e2e/N=1000 | task list --ready | 8 | 62.805 ms | 76.940 ms | 80.760 ms | 80.760 ms | 6.776 ms |
| e2e/N=1000 | task list --sort impact | 8 | 57.950 ms | 68.461 ms | 82.758 ms | 82.758 ms | 8.986 ms |
| e2e/N=1000 | task list --full-content | 8 | 96.568 ms | 103.455 ms | 109.082 ms | 109.082 ms | 4.497 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 73.701 ms | 87.267 ms | 89.108 ms | 89.108 ms | 4.988 ms |
| e2e/N=1000 | task graph | 8 | 63.805 ms | 73.381 ms | 75.864 ms | 75.864 ms | 4.879 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 171.066 ms | 185.316 ms | 189.315 ms | 189.315 ms | 5.959 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 103.126 ms | 109.128 ms | 120.926 ms | 120.926 ms | 5.543 ms |
| e2e/N=1000 | knowledge tags | 8 | 86.483 ms | 96.388 ms | 112.103 ms | 112.103 ms | 7.704 ms |
| e2e/N=1000 | drain --status | 8 | 35.788 ms | 50.226 ms | 55.272 ms | 55.272 ms | 8.296 ms |
| e2e/N=1000 | doctor | 8 | 217.021 ms | 230.846 ms | 238.998 ms | 238.998 ms | 7.956 ms |
| e2e/N=1000 | doctor --explain | 8 | 212.655 ms | 226.415 ms | 232.816 ms | 232.816 ms | 7.060 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 40.713 ms | 48.448 ms | 58.211 ms | 58.211 ms | 5.102 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 113.311 ms | 121.009 ms | 134.240 ms | 134.240 ms | 6.212 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 46.188 ms | 61.822 ms | 66.131 ms | 66.131 ms | 7.687 ms |
| e2e/N=1000 | config list | 8 | 57.663 ms | 74.028 ms | 80.337 ms | 80.337 ms | 9.490 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 47.301 ms | 56.243 ms | 74.134 ms | 74.134 ms | 10.612 ms |
| e2e/N=1000 | write (nova) | 8 | 90.515 ms | 101.947 ms | 118.440 ms | 118.440 ms | 9.676 ms |
| e2e/N=1000 | write (idempotente) | 8 | 88.115 ms | 100.341 ms | 114.151 ms | 114.151 ms | 8.128 ms |
| e2e/N=1000 | config set (projeto) | 8 | 45.400 ms | 63.331 ms | 76.137 ms | 76.137 ms | 10.945 ms |
| e2e/N=1000 | forget (soft) | 8 | 82.731 ms | 86.077 ms | 101.788 ms | 101.788 ms | 7.833 ms |
| e2e/N=1000 | forget --restore | 8 | 83.348 ms | 99.066 ms | 109.570 ms | 109.570 ms | 9.654 ms |
| e2e/N=1000 | sync | 8 | 48.824 ms | 62.915 ms | 84.399 ms | 84.399 ms | 13.340 ms |
