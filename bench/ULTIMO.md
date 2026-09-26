| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 1.97 µs | 1.99 µs | 2.20 µs | 2.28 µs | 70 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 14.18 µs | 15.06 µs | 18.64 µs | 19.31 µs | 1.39 µs |
| micro/fixo | schema::body::body_hash | 25 | 2.94 µs | 2.97 µs | 3.08 µs | 3.68 µs | 145 ns |
| micro/fixo | schema::id::note_id | 25 | 1.17 µs | 1.19 µs | 1.22 µs | 1.22 µs | 15 ns |
| micro/fixo | schema::hash::short_hash | 25 | 459 ns | 462 ns | 625 ns | 668 ns | 52 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 35 ns | 36 ns | 36 ns | 44 ns | 2 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 2.14 µs | 2.17 µs | 2.19 µs | 2.19 µs | 10 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 838 ns | 854 ns | 1.49 µs | 1.90 µs | 242 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.97 µs | 4.02 µs | 4.65 µs | 5.57 µs | 333 ns |
| micro/fixo | Note::render | 25 | 2.10 µs | 2.15 µs | 2.19 µs | 2.19 µs | 30 ns |
| micro/fixo | jsonl::decode | 25 | 841 ns | 853 ns | 892 ns | 903 ns | 17 ns |
| micro/fixo | jsonl::encode | 25 | 615 ns | 621 ns | 674 ns | 677 ns | 19 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 229 ns | 233 ns | 248 ns | 265 ns | 8 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 2.21 µs | 2.23 µs | 2.41 µs | 2.43 µs | 69 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 113.43 µs | 114.62 µs | 117.16 µs | 120.08 µs | 1.56 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 4 ns | 4 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 39 ns | 40 ns | 51 ns | 72 ns | 7 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.88 µs | 3.94 µs | 4.76 µs | 7.98 µs | 818 ns |
| micro/fixo | config::Config::parse | 25 | 3.06 µs | 3.09 µs | 3.11 µs | 3.11 µs | 14 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.49 µs | 29.58 µs | 31.18 µs | 32.04 µs | 595 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 802 ns | 805 ns | 823 ns | 823 ns | 8 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.266 ms | 1.280 ms | 1.303 ms | 1.307 ms | 11.91 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 212.60 µs | 218.60 µs | 228.24 µs | 232.29 µs | 5.49 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 336.85 µs | 348.72 µs | 359.06 µs | 359.47 µs | 6.26 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 803.80 µs | 882.03 µs | 1.098 ms | 1.164 ms | 111.49 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 17.39 µs | 18.09 µs | 18.44 µs | 19.77 µs | 543 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 328.19 µs | 341.11 µs | 379.87 µs | 405.08 µs | 20.68 µs |
| micro/N=200 | Graph::from_notes | 15 | 358.57 µs | 371.07 µs | 387.97 µs | 392.79 µs | 11.35 µs |
| micro/N=200 | Graph::integrity | 15 | 17.67 µs | 18.09 µs | 18.44 µs | 18.51 µs | 257 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 106.93 µs | 112.44 µs | 119.29 µs | 122.01 µs | 4.27 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 105.46 µs | 109.58 µs | 118.17 µs | 122.64 µs | 4.53 µs |
| micro/N=200 | Index::serialize + parse | 15 | 3.971 ms | 4.014 ms | 4.124 ms | 4.235 ms | 71.40 µs |
| micro/N=200 | write::Draft::to_note | 15 | 3.53 µs | 3.63 µs | 3.65 µs | 3.65 µs | 38 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.268 ms | 8.537 ms | 9.185 ms | 9.544 ms | 361.93 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.353 ms | 1.369 ms | 1.405 ms | 1.421 ms | 18.34 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 1.956 ms | 1.978 ms | 2.256 ms | 2.256 ms | 128.09 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 4.838 ms | 4.942 ms | 5.399 ms | 5.673 ms | 242.67 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 93.87 µs | 94.43 µs | 104.34 µs | 108.74 µs | 4.33 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 6.851 ms | 7.034 ms | 8.336 ms | 8.832 ms | 579.82 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.624 ms | 2.644 ms | 2.668 ms | 2.716 ms | 23.39 µs |
| micro/N=1000 | Graph::integrity | 15 | 132.42 µs | 137.24 µs | 142.41 µs | 155.19 µs | 5.73 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 602.38 µs | 614.75 µs | 624.45 µs | 629.69 µs | 8.04 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 597.28 µs | 613.07 µs | 627.88 µs | 628.71 µs | 11.10 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.345 ms | 21.538 ms | 22.067 ms | 22.557 ms | 337.91 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 3.47 µs | 3.55 µs | 3.59 µs | 3.59 µs | 42 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 17.679 ms | 25.790 ms | 30.760 ms | 30.760 ms | 5.050 ms |
| e2e/N=200 | self version --json | 8 | 16.173 ms | 19.075 ms | 25.683 ms | 25.683 ms | 3.552 ms |
| e2e/N=200 | --help | 8 | 1.660 ms | 3.336 ms | 3.807 ms | 3.807 ms | 793.87 µs |
| e2e/N=200 | prime | 8 | 14.860 ms | 18.386 ms | 28.091 ms | 28.091 ms | 4.423 ms |
| e2e/N=200 | prime --long | 8 | 18.229 ms | 24.174 ms | 26.550 ms | 26.550 ms | 2.446 ms |
| e2e/N=200 | ask (query comum) | 8 | 39.738 ms | 45.476 ms | 59.274 ms | 59.274 ms | 7.003 ms |
| e2e/N=200 | ask (query rara) | 8 | 35.448 ms | 50.462 ms | 68.839 ms | 68.839 ms | 11.194 ms |
| e2e/N=200 | ask --json | 8 | 30.373 ms | 37.124 ms | 50.349 ms | 50.349 ms | 7.269 ms |
| e2e/N=200 | ask --limit 50 | 8 | 30.218 ms | 37.619 ms | 40.204 ms | 40.204 ms | 3.109 ms |
| e2e/N=200 | ask --brief | 8 | 38.310 ms | 45.374 ms | 52.501 ms | 52.501 ms | 4.492 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 34.308 ms | 44.331 ms | 55.402 ms | 55.402 ms | 7.702 ms |
| e2e/N=200 | ask --around <nota> | 8 | 34.037 ms | 36.861 ms | 48.535 ms | 48.535 ms | 5.664 ms |
| e2e/N=200 | ask --id <nota> | 8 | 26.892 ms | 31.303 ms | 35.020 ms | 35.020 ms | 2.700 ms |
| e2e/N=200 | rewind | 8 | 47.497 ms | 60.155 ms | 73.714 ms | 73.714 ms | 8.913 ms |
| e2e/N=200 | rewind --json | 8 | 46.454 ms | 61.945 ms | 77.908 ms | 77.908 ms | 10.295 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 44.111 ms | 61.402 ms | 73.833 ms | 73.833 ms | 11.425 ms |
| e2e/N=200 | task list | 8 | 15.252 ms | 17.363 ms | 19.131 ms | 19.131 ms | 1.386 ms |
| e2e/N=200 | task list --ready | 8 | 37.834 ms | 47.646 ms | 69.157 ms | 69.157 ms | 11.538 ms |
| e2e/N=200 | task list --sort impact | 8 | 16.084 ms | 19.201 ms | 20.444 ms | 20.444 ms | 1.434 ms |
| e2e/N=200 | task list --full-content | 8 | 13.513 ms | 17.777 ms | 18.783 ms | 18.783 ms | 2.217 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 42.672 ms | 47.058 ms | 50.433 ms | 50.433 ms | 3.186 ms |
| e2e/N=200 | task graph | 8 | 48.922 ms | 59.094 ms | 64.860 ms | 64.860 ms | 6.520 ms |
| e2e/N=200 | knowledge map --universe | 8 | 74.090 ms | 84.004 ms | 93.351 ms | 93.351 ms | 5.887 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 55.828 ms | 65.047 ms | 69.218 ms | 69.218 ms | 5.727 ms |
| e2e/N=200 | knowledge tags | 8 | 50.010 ms | 64.070 ms | 68.437 ms | 68.437 ms | 6.690 ms |
| e2e/N=200 | drain --status | 8 | 23.528 ms | 29.896 ms | 31.996 ms | 31.996 ms | 3.326 ms |
| e2e/N=200 | doctor | 8 | 222.625 ms | 278.478 ms | 314.713 ms | 314.713 ms | 34.801 ms |
| e2e/N=200 | doctor --explain | 8 | 186.088 ms | 202.506 ms | 207.461 ms | 207.461 ms | 8.835 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 30.307 ms | 38.718 ms | 42.550 ms | 42.550 ms | 4.299 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 99.407 ms | 114.374 ms | 122.438 ms | 122.438 ms | 8.456 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 31.557 ms | 51.172 ms | 53.804 ms | 53.804 ms | 7.995 ms |
| e2e/N=200 | config list | 8 | 30.545 ms | 36.588 ms | 41.904 ms | 41.904 ms | 3.277 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 1.644 ms | 3.125 ms | 3.515 ms | 3.515 ms | 618.04 µs |
| e2e/N=200 | write (nova) | 8 | 34.803 ms | 42.995 ms | 50.684 ms | 50.684 ms | 5.148 ms |
| e2e/N=200 | write (idempotente) | 8 | 35.602 ms | 48.821 ms | 59.700 ms | 59.700 ms | 8.789 ms |
| e2e/N=200 | config set (projeto) | 8 | 2.470 ms | 3.684 ms | 4.032 ms | 4.032 ms | 508.16 µs |
| e2e/N=200 | forget (soft) | 8 | 17.644 ms | 22.088 ms | 31.165 ms | 31.165 ms | 4.333 ms |
| e2e/N=200 | forget --restore | 8 | 16.693 ms | 30.447 ms | 36.190 ms | 36.190 ms | 6.945 ms |
| e2e/N=200 | sync | 8 | 34.604 ms | 43.994 ms | 53.694 ms | 53.694 ms | 7.088 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 36.605 ms | 42.876 ms | 60.384 ms | 60.384 ms | 9.124 ms |
| e2e/N=1000 | self version --json | 8 | 38.142 ms | 55.433 ms | 67.119 ms | 67.119 ms | 11.588 ms |
| e2e/N=1000 | --help | 8 | 1.873 ms | 3.817 ms | 3.982 ms | 3.982 ms | 841.69 µs |
| e2e/N=1000 | prime | 8 | 36.038 ms | 43.636 ms | 55.659 ms | 55.659 ms | 7.838 ms |
| e2e/N=1000 | prime --long | 8 | 35.050 ms | 47.261 ms | 61.291 ms | 61.291 ms | 8.261 ms |
| e2e/N=1000 | ask (query comum) | 8 | 88.384 ms | 103.502 ms | 119.197 ms | 119.197 ms | 10.776 ms |
| e2e/N=1000 | ask (query rara) | 8 | 85.684 ms | 97.094 ms | 107.653 ms | 107.653 ms | 7.925 ms |
| e2e/N=1000 | ask --json | 8 | 89.692 ms | 102.947 ms | 111.926 ms | 111.926 ms | 8.554 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 89.064 ms | 102.126 ms | 119.048 ms | 119.048 ms | 10.451 ms |
| e2e/N=1000 | ask --brief | 8 | 89.934 ms | 103.850 ms | 109.690 ms | 109.690 ms | 7.615 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 91.599 ms | 95.991 ms | 108.536 ms | 108.536 ms | 6.959 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 75.505 ms | 90.293 ms | 94.683 ms | 94.683 ms | 7.335 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 44.559 ms | 63.697 ms | 72.272 ms | 72.272 ms | 11.512 ms |
| e2e/N=1000 | rewind | 8 | 286.986 ms | 294.993 ms | 313.337 ms | 313.337 ms | 10.115 ms |
| e2e/N=1000 | rewind --json | 8 | 290.785 ms | 299.569 ms | 319.909 ms | 319.909 ms | 10.557 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 95.837 ms | 100.688 ms | 118.740 ms | 118.740 ms | 7.444 ms |
| e2e/N=1000 | task list | 8 | 14.700 ms | 19.997 ms | 27.034 ms | 27.034 ms | 4.329 ms |
| e2e/N=1000 | task list --ready | 8 | 93.436 ms | 107.409 ms | 113.446 ms | 113.446 ms | 7.309 ms |
| e2e/N=1000 | task list --sort impact | 8 | 15.837 ms | 19.231 ms | 21.621 ms | 21.621 ms | 1.974 ms |
| e2e/N=1000 | task list --full-content | 8 | 14.938 ms | 17.145 ms | 20.635 ms | 20.635 ms | 2.241 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 68.555 ms | 77.280 ms | 91.332 ms | 91.332 ms | 7.443 ms |
| e2e/N=1000 | task graph | 8 | 95.068 ms | 106.839 ms | 132.229 ms | 132.229 ms | 11.597 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 192.480 ms | 198.540 ms | 202.678 ms | 202.678 ms | 3.832 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 108.462 ms | 117.318 ms | 125.775 ms | 125.775 ms | 5.327 ms |
| e2e/N=1000 | knowledge tags | 8 | 87.015 ms | 98.843 ms | 112.713 ms | 112.713 ms | 9.002 ms |
| e2e/N=1000 | drain --status | 8 | 32.333 ms | 49.289 ms | 52.562 ms | 52.562 ms | 7.231 ms |
| e2e/N=1000 | doctor | 8 | 4.811 s | 4.867 s | 4.946 s | 4.946 s | 53.926 ms |
| e2e/N=1000 | doctor --explain | 8 | 4.646 s | 4.806 s | 4.969 s | 4.969 s | 117.254 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 60.798 ms | 68.967 ms | 81.129 ms | 81.129 ms | 6.931 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 2.403 s | 2.444 s | 2.581 s | 2.581 s | 63.504 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 130.322 ms | 143.335 ms | 150.652 ms | 150.652 ms | 8.953 ms |
| e2e/N=1000 | config list | 8 | 52.969 ms | 71.764 ms | 85.298 ms | 85.298 ms | 11.124 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 1.684 ms | 3.766 ms | 3.871 ms | 3.871 ms | 926.81 µs |
| e2e/N=1000 | write (nova) | 8 | 88.582 ms | 99.564 ms | 126.964 ms | 126.964 ms | 13.864 ms |
| e2e/N=1000 | write (idempotente) | 8 | 82.909 ms | 95.954 ms | 109.329 ms | 109.329 ms | 8.901 ms |
| e2e/N=1000 | config set (projeto) | 8 | 1.659 ms | 3.225 ms | 4.036 ms | 4.036 ms | 946.32 µs |
| e2e/N=1000 | forget (soft) | 8 | 49.158 ms | 55.155 ms | 70.710 ms | 70.710 ms | 7.909 ms |
| e2e/N=1000 | forget --restore | 8 | 52.799 ms | 53.732 ms | 56.110 ms | 56.110 ms | 1.253 ms |
| e2e/N=1000 | sync | 8 | 50.639 ms | 77.936 ms | 90.518 ms | 90.518 ms | 13.699 ms |
