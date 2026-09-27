| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 536 ns | 543 ns | 564 ns | 577 ns | 10 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.27 µs | 1.30 µs | 3.24 µs | 3.77 µs | 635 ns |
| micro/fixo | schema::body::body_hash | 25 | 410 ns | 417 ns | 423 ns | 426 ns | 3 ns |
| micro/fixo | schema::id::note_id | 25 | 162 ns | 165 ns | 170 ns | 172 ns | 3 ns |
| micro/fixo | schema::hash::short_hash | 25 | 457 ns | 458 ns | 460 ns | 461 ns | 1 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 17 ns | 17 ns | 18 ns | 18 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.54 µs | 1.57 µs | 2.33 µs | 2.91 µs | 304 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 775 ns | 793 ns | 801 ns | 804 ns | 8 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.71 µs | 3.76 µs | 3.97 µs | 4.69 µs | 191 ns |
| micro/fixo | Note::render | 25 | 2.11 µs | 2.13 µs | 2.16 µs | 2.17 µs | 16 ns |
| micro/fixo | jsonl::decode | 25 | 833 ns | 843 ns | 864 ns | 1.43 µs | 117 ns |
| micro/fixo | jsonl::encode | 25 | 504 ns | 516 ns | 529 ns | 542 ns | 9 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 293 ns | 297 ns | 313 ns | 318 ns | 8 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 2.75 µs | 2.78 µs | 2.85 µs | 2.85 µs | 27 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 2.02 µs | 2.03 µs | 2.51 µs | 2.70 µs | 162 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 115.25 µs | 116.05 µs | 118.11 µs | 121.17 µs | 1.19 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 15 ns | 16 ns | 16 ns | 16 ns | 0 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 7 ns | 7 ns | 8 ns | 9 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 46 ns | 46 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.73 µs | 3.82 µs | 4.01 µs | 4.05 µs | 75 ns |
| micro/fixo | config::Config::parse | 25 | 3.12 µs | 3.15 µs | 3.62 µs | 4.21 µs | 231 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 227 ns | 229 ns | 261 ns | 296 ns | 15 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 135 ns | 138 ns | 142 ns | 150 ns | 3 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.93 µs | 30.04 µs | 32.02 µs | 32.61 µs | 795 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 804 ns | 805 ns | 812 ns | 819 ns | 3 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.502 ms | 1.569 ms | 1.900 ms | 2.296 ms | 210.20 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 561.53 µs | 579.55 µs | 591.56 µs | 602.45 µs | 10.89 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 214.69 µs | 219.09 µs | 232.64 µs | 234.32 µs | 6.84 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 51.485 ms | 51.944 ms | 53.953 ms | 54.487 ms | 1.049 ms |
| micro/N=200 | write::propose_merges (esparso) | 15 | 277.20 µs | 278.88 µs | 289.42 µs | 290.54 µs | 4.91 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 343.20 µs | 350.46 µs | 362.69 µs | 374.35 µs | 8.32 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 704.42 µs | 719.09 µs | 727.75 µs | 730.05 µs | 8.36 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 20.05 µs | 20.46 µs | 22.07 µs | 32.90 µs | 3.21 µs |
| micro/N=200 | lifecycle::structural_clusters | 15 | 146.04 µs | 147.85 µs | 154.84 µs | 160.29 µs | 3.78 µs |
| micro/N=200 | lifecycle::communities | 15 | 776.50 µs | 801.22 µs | 822.32 µs | 837.89 µs | 16.29 µs |
| micro/N=200 | Graph::from_notes | 15 | 390.13 µs | 402.36 µs | 415.91 µs | 431.34 µs | 10.75 µs |
| micro/N=200 | graph::pagerank | 15 | 82.20 µs | 82.97 µs | 86.46 µs | 91.98 µs | 2.49 µs |
| micro/N=200 | retrieval::recall (ppr) | 15 | 501.25 µs | 510.54 µs | 520.74 µs | 528.42 µs | 8.19 µs |
| micro/N=200 | Graph::integrity | 15 | 18.44 µs | 19.07 µs | 19.14 µs | 29.47 µs | 2.74 µs |
| micro/N=200 | Graph::supersession_cycles | 15 | 109.79 µs | 111.12 µs | 117.06 µs | 117.54 µs | 2.41 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 105.60 µs | 109.02 µs | 115.73 µs | 117.26 µs | 3.36 µs |
| micro/N=200 | retrieval::compute_views | 15 | 128.79 µs | 135.14 µs | 140.24 µs | 140.24 µs | 2.70 µs |
| micro/N=200 | task::impact (1 id) | 15 | 17.46 µs | 18.09 µs | 18.65 µs | 19.42 µs | 499 ns |
| micro/N=200 | task::impacts (todos) | 15 | 16.76 µs | 17.25 µs | 18.65 µs | 28.01 µs | 2.78 µs |
| micro/N=200 | handoff::rank (manifest) | 15 | 29.05 µs | 30.52 µs | 31.99 µs | 32.20 µs | 989 ns |
| micro/N=200 | Index::serialize + parse | 15 | 4.152 ms | 4.421 ms | 4.585 ms | 4.832 ms | 165.07 µs |
| micro/N=200 | Index::serialize | 15 | 1.562 ms | 1.596 ms | 1.805 ms | 1.825 ms | 81.82 µs |
| micro/N=200 | Index::parse | 15 | 2.361 ms | 2.387 ms | 2.441 ms | 2.454 ms | 24.68 µs |
| micro/N=200 | write::Draft::to_note | 15 | 2.00 µs | 2.04 µs | 2.07 µs | 2.09 µs | 31 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 9.236 ms | 10.159 ms | 10.965 ms | 11.942 ms | 802.40 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.597 ms | 3.946 ms | 4.482 ms | 5.642 ms | 520.23 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.288 ms | 1.345 ms | 1.834 ms | 3.644 ms | 597.92 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 35.705 ms | 36.842 ms | 40.347 ms | 40.599 ms | 1.513 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.590 ms | 1.610 ms | 1.773 ms | 1.794 ms | 71.38 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.069 ms | 2.330 ms | 2.630 ms | 2.648 ms | 205.77 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 4.215 ms | 4.707 ms | 4.976 ms | 5.116 ms | 269.05 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 104.90 µs | 110.14 µs | 113.42 µs | 117.40 µs | 2.58 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 864.71 µs | 881.05 µs | 891.95 µs | 894.95 µs | 7.41 µs |
| micro/N=1000 | lifecycle::communities | 15 | 4.655 ms | 4.942 ms | 5.279 ms | 5.402 ms | 248.27 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.824 ms | 3.238 ms | 3.536 ms | 3.675 ms | 254.73 µs |
| micro/N=1000 | graph::pagerank | 15 | 485.89 µs | 498.18 µs | 510.47 µs | 513.54 µs | 9.22 µs |
| micro/N=1000 | retrieval::recall (ppr) | 15 | 3.181 ms | 3.422 ms | 3.619 ms | 3.659 ms | 137.60 µs |
| micro/N=1000 | Graph::integrity | 15 | 134.38 µs | 139.19 µs | 147.02 µs | 150.51 µs | 4.40 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 594.00 µs | 609.72 µs | 617.12 µs | 619.36 µs | 8.21 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 598.40 µs | 612.30 µs | 620.05 µs | 622.01 µs | 6.87 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 772.24 µs | 796.54 µs | 812.96 µs | 814.21 µs | 12.46 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 147.51 µs | 152.67 µs | 160.78 µs | 162.94 µs | 4.40 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 146.95 µs | 147.44 µs | 155.19 µs | 158.26 µs | 3.36 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 170.13 µs | 177.54 µs | 188.85 µs | 194.86 µs | 6.99 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.708 ms | 22.464 ms | 22.827 ms | 26.778 ms | 1.199 ms |
| micro/N=1000 | Index::serialize | 15 | 8.292 ms | 8.643 ms | 9.643 ms | 10.962 ms | 668.15 µs |
| micro/N=1000 | Index::parse | 15 | 12.708 ms | 13.242 ms | 13.856 ms | 14.016 ms | 476.90 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 2.01 µs | 2.02 µs | 2.10 µs | 2.10 µs | 34 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 3.272 ms | 3.620 ms | 3.961 ms | 3.961 ms | 193.35 µs |
| e2e/N=200 | self version --json | 8 | 2.030 ms | 4.014 ms | 4.109 ms | 4.109 ms | 942.45 µs |
| e2e/N=200 | --help | 8 | 2.253 ms | 3.441 ms | 4.220 ms | 4.220 ms | 554.91 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 4.042 ms | 4.093 ms | 4.141 ms | 4.141 ms | 38.48 µs |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.814 ms | 5.854 ms | 5.905 ms | 5.905 ms | 33.71 µs |
| e2e/N=200 | prime | 8 | 1.881 ms | 3.277 ms | 3.765 ms | 3.765 ms | 630.11 µs |
| e2e/N=200 | prime --long | 8 | 2.228 ms | 3.084 ms | 3.723 ms | 3.723 ms | 535.47 µs |
| e2e/N=200 | ask (query comum) | 8 | 42.405 ms | 59.574 ms | 66.988 ms | 66.988 ms | 9.597 ms |
| e2e/N=200 | ask (query rara) | 8 | 33.480 ms | 58.065 ms | 67.560 ms | 67.560 ms | 11.935 ms |
| e2e/N=200 | ask --json | 8 | 32.485 ms | 40.094 ms | 46.142 ms | 46.142 ms | 4.961 ms |
| e2e/N=200 | ask --limit 50 | 8 | 44.690 ms | 64.026 ms | 67.282 ms | 67.282 ms | 7.273 ms |
| e2e/N=200 | ask --brief | 8 | 39.341 ms | 51.668 ms | 68.714 ms | 68.714 ms | 11.194 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 42.607 ms | 56.415 ms | 62.462 ms | 62.462 ms | 7.134 ms |
| e2e/N=200 | ask --around <nota> | 8 | 35.189 ms | 47.373 ms | 54.038 ms | 54.038 ms | 6.446 ms |
| e2e/N=200 | ask --id <nota> | 8 | 27.899 ms | 32.916 ms | 36.127 ms | 36.127 ms | 2.583 ms |
| e2e/N=200 | rewind | 8 | 47.738 ms | 56.777 ms | 70.297 ms | 70.297 ms | 7.713 ms |
| e2e/N=200 | rewind --json | 8 | 50.367 ms | 57.834 ms | 68.263 ms | 68.263 ms | 5.873 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 52.506 ms | 64.960 ms | 73.121 ms | 73.121 ms | 7.568 ms |
| e2e/N=200 | task list --universe | 8 | 38.037 ms | 49.745 ms | 60.591 ms | 60.591 ms | 8.957 ms |
| e2e/N=200 | task list --ready | 8 | 34.721 ms | 42.796 ms | 58.070 ms | 58.070 ms | 8.588 ms |
| e2e/N=200 | task list --sort impact | 8 | 35.332 ms | 41.416 ms | 53.995 ms | 53.995 ms | 6.171 ms |
| e2e/N=200 | task list --full-content | 8 | 38.633 ms | 51.705 ms | 63.918 ms | 63.918 ms | 9.082 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 36.104 ms | 53.657 ms | 57.900 ms | 57.900 ms | 7.857 ms |
| e2e/N=200 | task graph | 8 | 39.871 ms | 57.659 ms | 63.098 ms | 63.098 ms | 9.703 ms |
| e2e/N=200 | map --universe | 8 | 55.101 ms | 69.943 ms | 74.375 ms | 74.375 ms | 8.603 ms |
| e2e/N=200 | ask --rank --universe | 8 | 41.314 ms | 54.701 ms | 66.859 ms | 66.859 ms | 9.216 ms |
| e2e/N=200 | ask --tags | 8 | 34.345 ms | 48.844 ms | 51.722 ms | 51.722 ms | 6.931 ms |
| e2e/N=200 | drain --status | 8 | 13.586 ms | 15.522 ms | 20.996 ms | 20.996 ms | 2.340 ms |
| e2e/N=200 | doctor | 8 | 156.342 ms | 178.118 ms | 182.040 ms | 182.040 ms | 10.573 ms |
| e2e/N=200 | doctor --explain | 8 | 159.178 ms | 163.256 ms | 182.715 ms | 182.715 ms | 7.905 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 25.538 ms | 30.282 ms | 34.561 ms | 34.561 ms | 3.390 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 86.433 ms | 104.443 ms | 106.813 ms | 106.813 ms | 8.162 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 26.297 ms | 40.489 ms | 44.214 ms | 44.214 ms | 6.760 ms |
| e2e/N=200 | config list | 8 | 26.572 ms | 35.029 ms | 41.442 ms | 41.442 ms | 5.016 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 30.300 ms | 35.985 ms | 39.897 ms | 39.897 ms | 3.395 ms |
| e2e/N=200 | write (nova) | 8 | 39.341 ms | 55.685 ms | 63.296 ms | 63.296 ms | 7.753 ms |
| e2e/N=200 | write (idempotente) | 8 | 33.730 ms | 42.127 ms | 62.688 ms | 62.688 ms | 10.857 ms |
| e2e/N=200 | config set (projeto) | 8 | 24.751 ms | 31.276 ms | 40.085 ms | 40.085 ms | 4.883 ms |
| e2e/N=200 | forget (soft) | 8 | 38.099 ms | 56.516 ms | 65.879 ms | 65.879 ms | 10.863 ms |
| e2e/N=200 | forget --restore | 8 | 42.671 ms | 45.883 ms | 66.592 ms | 66.592 ms | 7.563 ms |
| e2e/N=200 | sync | 8 | 27.810 ms | 35.127 ms | 39.456 ms | 39.456 ms | 3.727 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 2.604 ms | 3.487 ms | 3.963 ms | 3.963 ms | 520.43 µs |
| e2e/N=1000 | self version --json | 8 | 1.962 ms | 3.784 ms | 4.099 ms | 4.099 ms | 774.67 µs |
| e2e/N=1000 | --help | 8 | 2.277 ms | 3.851 ms | 4.215 ms | 4.215 ms | 619.49 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 8.839 ms | 9.583 ms | 21.579 ms | 21.579 ms | 4.296 ms |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 19.802 ms | 21.761 ms | 22.388 ms | 22.388 ms | 934.81 µs |
| e2e/N=1000 | prime | 8 | 2.849 ms | 3.339 ms | 4.410 ms | 4.410 ms | 518.08 µs |
| e2e/N=1000 | prime --long | 8 | 1.829 ms | 3.925 ms | 4.073 ms | 4.073 ms | 757.33 µs |
| e2e/N=1000 | ask (query comum) | 8 | 79.353 ms | 90.499 ms | 99.905 ms | 99.905 ms | 7.256 ms |
| e2e/N=1000 | ask (query rara) | 8 | 72.513 ms | 82.127 ms | 91.345 ms | 91.345 ms | 6.214 ms |
| e2e/N=1000 | ask --json | 8 | 73.352 ms | 90.477 ms | 106.875 ms | 106.875 ms | 10.734 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 80.004 ms | 96.184 ms | 106.249 ms | 106.249 ms | 8.288 ms |
| e2e/N=1000 | ask --brief | 8 | 82.192 ms | 92.475 ms | 100.122 ms | 100.122 ms | 7.208 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 84.161 ms | 93.341 ms | 101.338 ms | 101.338 ms | 6.560 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 76.318 ms | 91.308 ms | 93.018 ms | 93.018 ms | 7.952 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 48.611 ms | 70.260 ms | 81.163 ms | 81.163 ms | 11.290 ms |
| e2e/N=1000 | rewind | 8 | 86.845 ms | 102.729 ms | 108.425 ms | 108.425 ms | 7.956 ms |
| e2e/N=1000 | rewind --json | 8 | 92.778 ms | 100.820 ms | 106.253 ms | 106.253 ms | 4.695 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 88.900 ms | 97.424 ms | 109.066 ms | 109.066 ms | 6.105 ms |
| e2e/N=1000 | task list --universe | 8 | 61.446 ms | 75.432 ms | 85.706 ms | 85.706 ms | 10.486 ms |
| e2e/N=1000 | task list --ready | 8 | 66.254 ms | 74.876 ms | 100.010 ms | 100.010 ms | 11.162 ms |
| e2e/N=1000 | task list --sort impact | 8 | 59.808 ms | 81.165 ms | 96.380 ms | 96.380 ms | 12.137 ms |
| e2e/N=1000 | task list --full-content | 8 | 98.517 ms | 109.619 ms | 112.877 ms | 112.877 ms | 5.247 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 74.027 ms | 80.261 ms | 94.039 ms | 94.039 ms | 6.912 ms |
| e2e/N=1000 | task graph | 8 | 67.373 ms | 72.923 ms | 84.145 ms | 84.145 ms | 5.601 ms |
| e2e/N=1000 | map --universe | 8 | 177.613 ms | 185.412 ms | 199.965 ms | 199.965 ms | 7.880 ms |
| e2e/N=1000 | ask --rank --universe | 8 | 117.098 ms | 126.891 ms | 139.873 ms | 139.873 ms | 7.456 ms |
| e2e/N=1000 | ask --tags | 8 | 89.082 ms | 100.438 ms | 110.583 ms | 110.583 ms | 7.695 ms |
| e2e/N=1000 | drain --status | 8 | 34.973 ms | 38.020 ms | 52.648 ms | 52.648 ms | 7.302 ms |
| e2e/N=1000 | doctor | 8 | 190.473 ms | 199.688 ms | 208.689 ms | 208.689 ms | 6.326 ms |
| e2e/N=1000 | doctor --explain | 8 | 188.909 ms | 193.446 ms | 200.375 ms | 200.375 ms | 3.307 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 52.804 ms | 59.330 ms | 63.826 ms | 63.826 ms | 4.259 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 100.411 ms | 104.577 ms | 107.151 ms | 107.151 ms | 2.358 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 47.381 ms | 51.269 ms | 58.009 ms | 58.009 ms | 3.128 ms |
| e2e/N=1000 | config list | 8 | 56.114 ms | 72.247 ms | 77.521 ms | 77.521 ms | 9.272 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 50.732 ms | 71.861 ms | 84.025 ms | 84.025 ms | 11.589 ms |
| e2e/N=1000 | write (nova) | 8 | 89.641 ms | 96.262 ms | 106.968 ms | 106.968 ms | 5.058 ms |
| e2e/N=1000 | write (idempotente) | 8 | 86.787 ms | 103.850 ms | 119.663 ms | 119.663 ms | 10.694 ms |
| e2e/N=1000 | config set (projeto) | 8 | 51.332 ms | 68.145 ms | 78.266 ms | 78.266 ms | 11.687 ms |
| e2e/N=1000 | forget (soft) | 8 | 88.444 ms | 99.988 ms | 106.748 ms | 106.748 ms | 6.940 ms |
| e2e/N=1000 | forget --restore | 8 | 82.685 ms | 98.569 ms | 119.488 ms | 119.488 ms | 12.120 ms |
| e2e/N=1000 | sync | 8 | 54.484 ms | 79.326 ms | 98.826 ms | 98.826 ms | 15.799 ms |
