| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 532 ns | 697 ns | 805 ns | 823 ns | 87 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.67 µs | 1.71 µs | 1.78 µs | 1.78 µs | 28 ns |
| micro/fixo | schema::body::body_hash | 25 | 504 ns | 511 ns | 519 ns | 520 ns | 5 ns |
| micro/fixo | schema::id::note_id | 25 | 208 ns | 212 ns | 223 ns | 223 ns | 5 ns |
| micro/fixo | schema::hash::short_hash | 25 | 458 ns | 460 ns | 463 ns | 468 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 18 ns | 18 ns | 20 ns | 20 ns | 1 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.53 µs | 1.55 µs | 1.58 µs | 1.61 µs | 18 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 805 ns | 829 ns | 848 ns | 892 ns | 17 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.48 µs | 3.53 µs | 3.58 µs | 3.60 µs | 26 ns |
| micro/fixo | Note::render | 25 | 2.05 µs | 2.07 µs | 2.09 µs | 2.25 µs | 38 ns |
| micro/fixo | jsonl::decode | 25 | 818 ns | 825 ns | 833 ns | 838 ns | 5 ns |
| micro/fixo | jsonl::encode | 25 | 532 ns | 544 ns | 553 ns | 556 ns | 6 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 240 ns | 249 ns | 270 ns | 279 ns | 11 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 2.53 µs | 2.56 µs | 2.65 µs | 2.67 µs | 35 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.93 µs | 1.95 µs | 1.98 µs | 3.10 µs | 230 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 115.66 µs | 116.63 µs | 125.02 µs | 136.83 µs | 4.54 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 5 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 45 ns | 46 ns | 1 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.74 µs | 3.83 µs | 3.87 µs | 3.88 µs | 37 ns |
| micro/fixo | config::Config::parse | 25 | 3.05 µs | 3.07 µs | 3.09 µs | 3.99 µs | 185 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 241 ns | 243 ns | 305 ns | 349 ns | 24 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 138 ns | 145 ns | 154 ns | 4 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.46 µs | 29.68 µs | 29.89 µs | 30.40 µs | 178 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 803 ns | 814 ns | 829 ns | 829 ns | 8 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.284 ms | 1.301 ms | 1.385 ms | 1.480 ms | 50.56 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 562.99 µs | 575.56 µs | 596.87 µs | 605.11 µs | 12.42 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 218.81 µs | 223.28 µs | 235.50 µs | 242.42 µs | 6.76 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 51.051 ms | 51.392 ms | 53.023 ms | 54.193 ms | 868.25 µs |
| micro/N=200 | write::propose_merges (esparso) | 15 | 273.22 µs | 276.01 µs | 292.43 µs | 303.74 µs | 8.97 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 360.38 µs | 375.89 µs | 430.22 µs | 485.96 µs | 34.64 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 837.68 µs | 856.75 µs | 989.10 µs | 1.004 ms | 57.90 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 16.69 µs | 17.53 µs | 18.23 µs | 18.58 µs | 577 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 145.48 µs | 150.79 µs | 157.70 µs | 162.52 µs | 3.95 µs |
| micro/N=200 | Graph::from_notes | 15 | 360.31 µs | 370.93 µs | 382.59 µs | 385.52 µs | 7.90 µs |
| micro/N=200 | Graph::integrity | 15 | 18.30 µs | 18.65 µs | 19.00 µs | 19.07 µs | 217 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 111.61 µs | 121.80 µs | 149.74 µs | 151.77 µs | 14.82 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 112.93 µs | 116.29 µs | 146.46 µs | 152.81 µs | 13.33 µs |
| micro/N=200 | retrieval::compute_views | 15 | 135.84 µs | 141.64 µs | 173.70 µs | 183.06 µs | 13.84 µs |
| micro/N=200 | task::impact (1 id) | 15 | 17.11 µs | 17.53 µs | 18.72 µs | 26.75 µs | 2.40 µs |
| micro/N=200 | task::impacts (todos) | 15 | 16.62 µs | 17.04 µs | 18.23 µs | 18.79 µs | 595 ns |
| micro/N=200 | handoff::rank (manifest) | 15 | 30.10 µs | 31.01 µs | 33.38 µs | 40.86 µs | 2.64 µs |
| micro/N=200 | Index::serialize + parse | 15 | 3.876 ms | 4.052 ms | 5.160 ms | 5.263 ms | 438.95 µs |
| micro/N=200 | Index::serialize | 15 | 1.532 ms | 1.572 ms | 2.174 ms | 2.908 ms | 373.13 µs |
| micro/N=200 | Index::parse | 15 | 2.304 ms | 2.341 ms | 2.387 ms | 2.398 ms | 25.83 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.89 µs | 1.97 µs | 3.74 µs | 4.30 µs | 729 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.415 ms | 9.452 ms | 10.130 ms | 10.485 ms | 635.31 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.748 ms | 4.237 ms | 4.464 ms | 4.477 ms | 242.73 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.364 ms | 1.572 ms | 1.781 ms | 1.902 ms | 162.44 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.510 s | 1.560 s | 1.644 s | 1.720 s | 60.808 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.569 ms | 1.595 ms | 1.666 ms | 1.686 ms | 33.82 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.165 ms | 2.189 ms | 2.455 ms | 2.523 ms | 123.10 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.089 ms | 5.214 ms | 5.967 ms | 6.063 ms | 386.95 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 92.96 µs | 93.73 µs | 108.88 µs | 110.56 µs | 5.66 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 889.92 µs | 910.59 µs | 935.04 µs | 983.44 µs | 21.64 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.724 ms | 2.907 ms | 3.238 ms | 3.476 ms | 234.36 µs |
| micro/N=1000 | Graph::integrity | 15 | 133.19 µs | 134.86 µs | 161.75 µs | 169.02 µs | 10.75 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 631.02 µs | 658.81 µs | 669.92 µs | 673.55 µs | 13.46 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 630.67 µs | 653.72 µs | 666.22 µs | 667.13 µs | 13.01 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 803.18 µs | 833.21 µs | 849.06 µs | 853.67 µs | 13.58 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 148.69 µs | 149.46 µs | 162.87 µs | 166.22 µs | 5.38 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 147.30 µs | 148.06 µs | 160.29 µs | 168.67 µs | 5.98 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 173.00 µs | 177.26 µs | 187.10 µs | 201.70 µs | 6.88 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.006 ms | 21.564 ms | 22.401 ms | 22.713 ms | 507.10 µs |
| micro/N=1000 | Index::serialize | 15 | 7.890 ms | 7.990 ms | 8.450 ms | 8.536 ms | 229.87 µs |
| micro/N=1000 | Index::parse | 15 | 12.477 ms | 13.138 ms | 13.505 ms | 14.114 ms | 488.80 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.87 µs | 1.91 µs | 1.94 µs | 1.97 µs | 27 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 2.890 ms | 4.024 ms | 4.356 ms | 4.356 ms | 473.51 µs |
| e2e/N=200 | self version --json | 8 | 1.675 ms | 2.277 ms | 2.863 ms | 2.863 ms | 472.27 µs |
| e2e/N=200 | --help | 8 | 1.724 ms | 4.051 ms | 4.203 ms | 4.203 ms | 1.177 ms |
| e2e/N=200 | core: Corpus::load_notes | 8 | 3.973 ms | 4.161 ms | 4.460 ms | 4.460 ms | 178.12 µs |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.611 ms | 5.718 ms | 6.518 ms | 6.518 ms | 304.36 µs |
| e2e/N=200 | prime | 8 | 1.649 ms | 3.455 ms | 3.703 ms | 3.703 ms | 808.63 µs |
| e2e/N=200 | prime --long | 8 | 1.667 ms | 2.085 ms | 3.060 ms | 3.060 ms | 445.42 µs |
| e2e/N=200 | ask (query comum) | 8 | 31.575 ms | 35.644 ms | 62.431 ms | 62.431 ms | 13.535 ms |
| e2e/N=200 | ask (query rara) | 8 | 37.246 ms | 54.911 ms | 63.399 ms | 63.399 ms | 7.905 ms |
| e2e/N=200 | ask --json | 8 | 34.533 ms | 41.083 ms | 46.475 ms | 46.475 ms | 4.929 ms |
| e2e/N=200 | ask --limit 50 | 8 | 32.813 ms | 36.106 ms | 46.744 ms | 46.744 ms | 5.201 ms |
| e2e/N=200 | ask --brief | 8 | 29.462 ms | 39.744 ms | 54.493 ms | 54.493 ms | 8.133 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 34.275 ms | 50.408 ms | 64.598 ms | 64.598 ms | 10.927 ms |
| e2e/N=200 | ask --around <nota> | 8 | 27.317 ms | 30.168 ms | 41.583 ms | 41.583 ms | 4.791 ms |
| e2e/N=200 | ask --id <nota> | 8 | 26.455 ms | 31.377 ms | 41.271 ms | 41.271 ms | 4.785 ms |
| e2e/N=200 | rewind | 8 | 40.639 ms | 49.906 ms | 57.567 ms | 57.567 ms | 6.418 ms |
| e2e/N=200 | rewind --json | 8 | 37.766 ms | 50.319 ms | 52.940 ms | 52.940 ms | 5.018 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 43.912 ms | 55.584 ms | 67.754 ms | 67.754 ms | 8.699 ms |
| e2e/N=200 | task list --universe | 8 | 31.589 ms | 47.568 ms | 57.638 ms | 57.638 ms | 11.004 ms |
| e2e/N=200 | task list --ready | 8 | 32.205 ms | 48.868 ms | 61.485 ms | 61.485 ms | 11.995 ms |
| e2e/N=200 | task list --sort impact | 8 | 32.730 ms | 39.623 ms | 43.843 ms | 43.843 ms | 3.669 ms |
| e2e/N=200 | task list --full-content | 8 | 34.885 ms | 38.946 ms | 56.082 ms | 56.082 ms | 6.933 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 30.994 ms | 44.901 ms | 58.025 ms | 58.025 ms | 8.373 ms |
| e2e/N=200 | task graph | 8 | 30.349 ms | 34.125 ms | 43.672 ms | 43.672 ms | 4.708 ms |
| e2e/N=200 | knowledge map --universe | 8 | 53.367 ms | 65.618 ms | 74.451 ms | 74.451 ms | 7.816 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 41.452 ms | 50.984 ms | 61.743 ms | 61.743 ms | 6.919 ms |
| e2e/N=200 | knowledge tags | 8 | 32.742 ms | 52.784 ms | 56.804 ms | 56.804 ms | 9.232 ms |
| e2e/N=200 | drain --status | 8 | 21.796 ms | 26.730 ms | 31.891 ms | 31.891 ms | 3.439 ms |
| e2e/N=200 | doctor | 8 | 160.918 ms | 173.662 ms | 180.503 ms | 180.503 ms | 8.016 ms |
| e2e/N=200 | doctor --explain | 8 | 156.853 ms | 165.971 ms | 182.293 ms | 182.293 ms | 8.289 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 26.186 ms | 29.609 ms | 41.900 ms | 41.900 ms | 5.971 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 87.049 ms | 95.417 ms | 107.712 ms | 107.712 ms | 7.283 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 30.681 ms | 40.049 ms | 49.562 ms | 49.562 ms | 7.206 ms |
| e2e/N=200 | config list | 8 | 28.719 ms | 34.810 ms | 43.775 ms | 43.775 ms | 5.169 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 27.037 ms | 31.538 ms | 38.029 ms | 38.029 ms | 3.559 ms |
| e2e/N=200 | write (nova) | 8 | 34.377 ms | 40.466 ms | 55.078 ms | 55.078 ms | 7.341 ms |
| e2e/N=200 | write (idempotente) | 8 | 36.787 ms | 39.543 ms | 55.071 ms | 55.071 ms | 7.491 ms |
| e2e/N=200 | config set (projeto) | 8 | 23.944 ms | 29.935 ms | 34.669 ms | 34.669 ms | 3.347 ms |
| e2e/N=200 | forget (soft) | 8 | 29.356 ms | 37.773 ms | 54.864 ms | 54.864 ms | 8.779 ms |
| e2e/N=200 | forget --restore | 8 | 30.751 ms | 37.686 ms | 52.573 ms | 52.573 ms | 8.864 ms |
| e2e/N=200 | sync | 8 | 28.903 ms | 30.202 ms | 39.883 ms | 39.883 ms | 3.573 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 1.995 ms | 3.535 ms | 4.386 ms | 4.386 ms | 917.26 µs |
| e2e/N=1000 | self version --json | 8 | 3.735 ms | 4.063 ms | 4.105 ms | 4.105 ms | 117.72 µs |
| e2e/N=1000 | --help | 8 | 2.373 ms | 3.966 ms | 4.068 ms | 4.068 ms | 568.38 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 8.520 ms | 8.985 ms | 9.397 ms | 9.397 ms | 284.58 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 18.649 ms | 19.585 ms | 19.888 ms | 19.888 ms | 390.39 µs |
| e2e/N=1000 | prime | 8 | 1.743 ms | 2.033 ms | 2.464 ms | 2.464 ms | 205.20 µs |
| e2e/N=1000 | prime --long | 8 | 1.996 ms | 2.228 ms | 2.639 ms | 2.639 ms | 224.66 µs |
| e2e/N=1000 | ask (query comum) | 8 | 70.784 ms | 86.628 ms | 93.877 ms | 93.877 ms | 7.984 ms |
| e2e/N=1000 | ask (query rara) | 8 | 73.186 ms | 86.361 ms | 93.654 ms | 93.654 ms | 7.661 ms |
| e2e/N=1000 | ask --json | 8 | 70.046 ms | 85.739 ms | 92.159 ms | 92.159 ms | 6.422 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 73.906 ms | 87.760 ms | 113.106 ms | 113.106 ms | 11.368 ms |
| e2e/N=1000 | ask --brief | 8 | 69.117 ms | 82.643 ms | 86.618 ms | 86.618 ms | 5.771 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 71.995 ms | 84.149 ms | 98.252 ms | 98.252 ms | 9.099 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 73.240 ms | 86.139 ms | 105.921 ms | 105.921 ms | 10.291 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 45.831 ms | 58.566 ms | 73.695 ms | 73.695 ms | 8.775 ms |
| e2e/N=1000 | rewind | 8 | 81.364 ms | 93.558 ms | 100.460 ms | 100.460 ms | 7.550 ms |
| e2e/N=1000 | rewind --json | 8 | 81.434 ms | 92.875 ms | 107.124 ms | 107.124 ms | 9.493 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 77.725 ms | 85.651 ms | 108.639 ms | 108.639 ms | 10.239 ms |
| e2e/N=1000 | task list --universe | 8 | 57.110 ms | 68.767 ms | 89.898 ms | 89.898 ms | 11.703 ms |
| e2e/N=1000 | task list --ready | 8 | 62.214 ms | 73.363 ms | 80.544 ms | 80.544 ms | 6.208 ms |
| e2e/N=1000 | task list --sort impact | 8 | 60.281 ms | 65.477 ms | 94.069 ms | 94.069 ms | 14.875 ms |
| e2e/N=1000 | task list --full-content | 8 | 97.387 ms | 102.764 ms | 113.303 ms | 113.303 ms | 5.960 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 72.991 ms | 79.710 ms | 88.909 ms | 88.909 ms | 5.986 ms |
| e2e/N=1000 | task graph | 8 | 60.422 ms | 66.500 ms | 79.115 ms | 79.115 ms | 5.860 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 175.587 ms | 191.040 ms | 204.943 ms | 204.943 ms | 8.825 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 105.251 ms | 112.018 ms | 122.513 ms | 122.513 ms | 7.222 ms |
| e2e/N=1000 | knowledge tags | 8 | 85.096 ms | 101.315 ms | 117.782 ms | 117.782 ms | 10.430 ms |
| e2e/N=1000 | drain --status | 8 | 35.119 ms | 38.878 ms | 51.990 ms | 51.990 ms | 6.536 ms |
| e2e/N=1000 | doctor | 8 | 3.908 s | 4.004 s | 4.229 s | 4.229 s | 130.049 ms |
| e2e/N=1000 | doctor --explain | 8 | 3.894 s | 3.963 s | 4.026 s | 4.026 s | 44.623 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 46.846 ms | 54.056 ms | 68.654 ms | 68.654 ms | 6.320 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 1.962 s | 2.035 s | 2.143 s | 2.143 s | 53.457 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 120.209 ms | 126.404 ms | 132.384 ms | 132.384 ms | 4.274 ms |
| e2e/N=1000 | config list | 8 | 49.900 ms | 57.038 ms | 69.670 ms | 69.670 ms | 7.606 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 43.726 ms | 53.210 ms | 73.317 ms | 73.317 ms | 9.070 ms |
| e2e/N=1000 | write (nova) | 8 | 85.723 ms | 106.036 ms | 125.357 ms | 125.357 ms | 14.692 ms |
| e2e/N=1000 | write (idempotente) | 8 | 82.995 ms | 89.212 ms | 99.989 ms | 99.989 ms | 5.922 ms |
| e2e/N=1000 | config set (projeto) | 8 | 48.855 ms | 54.115 ms | 74.783 ms | 74.783 ms | 8.462 ms |
| e2e/N=1000 | forget (soft) | 8 | 84.471 ms | 86.205 ms | 98.716 ms | 98.716 ms | 4.668 ms |
| e2e/N=1000 | forget --restore | 8 | 79.298 ms | 88.948 ms | 112.451 ms | 112.451 ms | 10.006 ms |
| e2e/N=1000 | sync | 8 | 54.008 ms | 69.230 ms | 91.279 ms | 91.279 ms | 11.570 ms |
