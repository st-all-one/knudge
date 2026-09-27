| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 233 ns | 247 ns | 464 ns | 572 ns | 89 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.65 µs | 1.68 µs | 2.00 µs | 2.04 µs | 105 ns |
| micro/fixo | schema::body::body_hash | 25 | 505 ns | 509 ns | 760 ns | 1.05 µs | 116 ns |
| micro/fixo | schema::id::note_id | 25 | 181 ns | 188 ns | 487 ns | 532 ns | 102 ns |
| micro/fixo | schema::hash::short_hash | 25 | 457 ns | 460 ns | 466 ns | 654 ns | 39 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 18 ns | 18 ns | 19 ns | 19 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.56 µs | 1.59 µs | 1.65 µs | 1.67 µs | 26 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 779 ns | 792 ns | 854 ns | 1.75 µs | 192 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.39 µs | 3.41 µs | 3.70 µs | 4.41 µs | 207 ns |
| micro/fixo | Note::render | 25 | 2.02 µs | 2.03 µs | 2.08 µs | 2.09 µs | 18 ns |
| micro/fixo | jsonl::decode | 25 | 790 ns | 798 ns | 813 ns | 815 ns | 6 ns |
| micro/fixo | jsonl::encode | 25 | 520 ns | 530 ns | 559 ns | 582 ns | 13 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 257 ns | 264 ns | 277 ns | 278 ns | 6 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 2.59 µs | 2.61 µs | 3.02 µs | 3.51 µs | 195 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.94 µs | 1.96 µs | 2.08 µs | 3.58 µs | 324 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 116.90 µs | 121.90 µs | 125.57 µs | 125.65 µs | 2.45 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 15 ns | 16 ns | 18 ns | 23 ns | 1 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 7 ns | 7 ns | 9 ns | 18 ns | 2 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 41 ns | 46 ns | 47 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.69 µs | 3.77 µs | 3.80 µs | 4.02 µs | 62 ns |
| micro/fixo | config::Config::parse | 25 | 3.12 µs | 3.19 µs | 3.73 µs | 3.83 µs | 179 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 230 ns | 236 ns | 242 ns | 282 ns | 10 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 140 ns | 144 ns | 144 ns | 2 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.75 µs | 30.16 µs | 31.99 µs | 32.35 µs | 748 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 805 ns | 810 ns | 837 ns | 849 ns | 12 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.479 ms | 1.508 ms | 1.531 ms | 1.537 ms | 17.94 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 544.07 µs | 566.69 µs | 577.73 µs | 584.64 µs | 12.62 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 229.29 µs | 232.22 µs | 252.41 µs | 255.27 µs | 8.63 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 51.748 ms | 53.697 ms | 56.095 ms | 57.197 ms | 1.539 ms |
| micro/N=200 | write::propose_merges (esparso) | 15 | 267.49 µs | 272.87 µs | 290.33 µs | 291.17 µs | 8.80 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 340.41 µs | 349.00 µs | 364.78 µs | 366.11 µs | 9.03 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 698.69 µs | 724.33 µs | 746.47 µs | 1.153 ms | 111.91 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 19.77 µs | 20.39 µs | 21.09 µs | 21.51 µs | 504 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 144.22 µs | 147.16 µs | 157.84 µs | 161.68 µs | 5.05 µs |
| micro/N=200 | lifecycle::communities | 15 | 733.26 µs | 753.03 µs | 1.015 ms | 1.072 ms | 105.32 µs |
| micro/N=200 | Graph::from_notes | 15 | 366.32 µs | 382.59 µs | 408.78 µs | 425.96 µs | 18.69 µs |
| micro/N=200 | graph::pagerank | 15 | 81.99 µs | 82.55 µs | 84.09 µs | 97.71 µs | 3.93 µs |
| micro/N=200 | retrieval::recall (ppr) | 15 | 495.18 µs | 512.15 µs | 556.78 µs | 740.67 µs | 61.06 µs |
| micro/N=200 | Graph::integrity | 15 | 17.11 µs | 18.58 µs | 18.93 µs | 19.14 µs | 444 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 103.85 µs | 106.65 µs | 138.22 µs | 140.87 µs | 14.42 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 102.39 µs | 104.90 µs | 123.48 µs | 136.40 µs | 9.37 µs |
| micro/N=200 | retrieval::compute_views | 15 | 128.37 µs | 129.84 µs | 143.24 µs | 144.57 µs | 5.88 µs |
| micro/N=200 | task::impact (1 id) | 15 | 17.11 µs | 17.60 µs | 18.23 µs | 18.72 µs | 439 ns |
| micro/N=200 | task::impacts (todos) | 15 | 16.13 µs | 17.53 µs | 23.82 µs | 30.52 µs | 3.73 µs |
| micro/N=200 | handoff::rank (manifest) | 15 | 28.70 µs | 30.03 µs | 31.50 µs | 32.41 µs | 1.02 µs |
| micro/N=200 | Index::serialize + parse | 15 | 4.150 ms | 4.368 ms | 5.046 ms | 5.368 ms | 345.93 µs |
| micro/N=200 | Index::serialize | 15 | 1.608 ms | 1.636 ms | 1.725 ms | 1.746 ms | 46.89 µs |
| micro/N=200 | Index::parse | 15 | 2.332 ms | 2.405 ms | 2.638 ms | 3.126 ms | 206.91 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.82 µs | 1.90 µs | 2.06 µs | 2.17 µs | 100 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 9.288 ms | 10.327 ms | 11.324 ms | 11.565 ms | 610.30 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.667 ms | 4.004 ms | 4.532 ms | 4.602 ms | 304.49 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.396 ms | 1.669 ms | 2.052 ms | 2.234 ms | 255.56 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 36.385 ms | 37.547 ms | 39.648 ms | 39.701 ms | 1.093 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.585 ms | 1.642 ms | 2.135 ms | 2.407 ms | 248.28 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.059 ms | 2.215 ms | 2.570 ms | 2.591 ms | 180.21 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 4.169 ms | 4.611 ms | 5.083 ms | 5.162 ms | 355.42 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 102.25 µs | 106.65 µs | 138.78 µs | 146.11 µs | 13.67 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 902.91 µs | 918.48 µs | 1.026 ms | 1.030 ms | 48.34 µs |
| micro/N=1000 | lifecycle::communities | 15 | 4.476 ms | 4.742 ms | 5.117 ms | 5.270 ms | 239.40 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.648 ms | 2.780 ms | 3.238 ms | 3.279 ms | 219.02 µs |
| micro/N=1000 | graph::pagerank | 15 | 486.65 µs | 558.94 µs | 686.19 µs | 838.10 µs | 95.71 µs |
| micro/N=1000 | retrieval::recall (ppr) | 15 | 3.033 ms | 3.138 ms | 3.568 ms | 3.958 ms | 251.73 µs |
| micro/N=1000 | Graph::integrity | 15 | 134.59 µs | 139.40 µs | 150.72 µs | 152.74 µs | 5.76 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 598.54 µs | 617.26 µs | 654.13 µs | 693.60 µs | 23.94 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 589.74 µs | 610.34 µs | 716.92 µs | 817.35 µs | 60.88 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 777.75 µs | 804.85 µs | 900.67 µs | 1.127 ms | 90.74 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 143.52 µs | 149.04 µs | 158.33 µs | 160.78 µs | 4.92 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 144.43 µs | 146.04 µs | 155.96 µs | 159.38 µs | 4.43 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 167.27 µs | 171.39 µs | 182.43 µs | 182.78 µs | 5.21 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.862 ms | 22.493 ms | 24.253 ms | 24.451 ms | 783.59 µs |
| micro/N=1000 | Index::serialize | 15 | 8.519 ms | 9.145 ms | 10.378 ms | 11.822 ms | 860.70 µs |
| micro/N=1000 | Index::parse | 15 | 12.886 ms | 13.510 ms | 14.311 ms | 14.358 ms | 478.92 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.81 µs | 1.84 µs | 1.90 µs | 2.11 µs | 74 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 2.236 ms | 3.400 ms | 3.658 ms | 3.658 ms | 495.87 µs |
| e2e/N=200 | self version --json | 8 | 1.685 ms | 2.472 ms | 3.143 ms | 3.143 ms | 466.33 µs |
| e2e/N=200 | --help | 8 | 1.783 ms | 2.167 ms | 2.988 ms | 2.988 ms | 498.34 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 4.042 ms | 4.657 ms | 6.772 ms | 6.772 ms | 1.112 ms |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.791 ms | 5.832 ms | 6.112 ms | 6.112 ms | 117.97 µs |
| e2e/N=200 | prime | 8 | 1.554 ms | 1.677 ms | 1.854 ms | 1.854 ms | 99.19 µs |
| e2e/N=200 | prime --long | 8 | 1.844 ms | 2.295 ms | 2.454 ms | 2.454 ms | 243.81 µs |
| e2e/N=200 | ask (query comum) | 8 | 28.525 ms | 32.720 ms | 34.265 ms | 34.265 ms | 1.756 ms |
| e2e/N=200 | ask (query rara) | 8 | 30.889 ms | 32.740 ms | 37.727 ms | 37.727 ms | 2.196 ms |
| e2e/N=200 | ask --json | 8 | 33.099 ms | 35.406 ms | 50.480 ms | 50.480 ms | 5.926 ms |
| e2e/N=200 | ask --limit 50 | 8 | 33.949 ms | 44.781 ms | 51.059 ms | 51.059 ms | 5.909 ms |
| e2e/N=200 | ask --brief | 8 | 31.937 ms | 37.300 ms | 44.110 ms | 44.110 ms | 4.119 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 31.271 ms | 34.194 ms | 40.549 ms | 40.549 ms | 3.032 ms |
| e2e/N=200 | ask --around <nota> | 8 | 28.699 ms | 35.448 ms | 39.896 ms | 39.896 ms | 3.647 ms |
| e2e/N=200 | ask --id <nota> | 8 | 22.688 ms | 24.794 ms | 26.512 ms | 26.512 ms | 1.400 ms |
| e2e/N=200 | rewind | 8 | 35.337 ms | 40.991 ms | 48.700 ms | 48.700 ms | 4.363 ms |
| e2e/N=200 | rewind --json | 8 | 34.037 ms | 36.940 ms | 41.634 ms | 41.634 ms | 2.238 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 34.426 ms | 37.805 ms | 48.888 ms | 48.888 ms | 4.427 ms |
| e2e/N=200 | task list --universe | 8 | 29.424 ms | 37.991 ms | 43.788 ms | 43.788 ms | 4.793 ms |
| e2e/N=200 | task list --ready | 8 | 25.752 ms | 36.408 ms | 47.030 ms | 47.030 ms | 7.892 ms |
| e2e/N=200 | task list --sort impact | 8 | 27.281 ms | 30.418 ms | 31.673 ms | 31.673 ms | 1.788 ms |
| e2e/N=200 | task list --full-content | 8 | 31.357 ms | 35.262 ms | 38.604 ms | 38.604 ms | 2.386 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 27.113 ms | 31.908 ms | 37.512 ms | 37.512 ms | 3.025 ms |
| e2e/N=200 | task graph | 8 | 31.953 ms | 38.711 ms | 51.040 ms | 51.040 ms | 6.920 ms |
| e2e/N=200 | knowledge map --universe | 8 | 50.433 ms | 62.393 ms | 71.882 ms | 71.882 ms | 7.499 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 34.053 ms | 47.709 ms | 65.271 ms | 65.271 ms | 11.155 ms |
| e2e/N=200 | knowledge tags | 8 | 31.462 ms | 34.314 ms | 36.015 ms | 36.015 ms | 1.553 ms |
| e2e/N=200 | drain --status | 8 | 14.773 ms | 19.401 ms | 22.795 ms | 22.795 ms | 2.739 ms |
| e2e/N=200 | doctor | 8 | 166.439 ms | 174.649 ms | 190.870 ms | 190.870 ms | 8.177 ms |
| e2e/N=200 | doctor --explain | 8 | 159.100 ms | 181.744 ms | 192.254 ms | 192.254 ms | 11.228 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 23.520 ms | 30.045 ms | 39.683 ms | 39.683 ms | 5.551 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 87.818 ms | 94.125 ms | 108.029 ms | 108.029 ms | 6.930 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 19.714 ms | 23.906 ms | 39.647 ms | 39.647 ms | 7.554 ms |
| e2e/N=200 | config list | 8 | 24.925 ms | 30.413 ms | 36.683 ms | 36.683 ms | 3.959 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 24.755 ms | 29.382 ms | 37.502 ms | 37.502 ms | 4.117 ms |
| e2e/N=200 | write (nova) | 8 | 31.665 ms | 34.498 ms | 36.708 ms | 36.708 ms | 2.086 ms |
| e2e/N=200 | write (idempotente) | 8 | 29.997 ms | 34.307 ms | 39.002 ms | 39.002 ms | 3.006 ms |
| e2e/N=200 | config set (projeto) | 8 | 21.933 ms | 26.542 ms | 29.294 ms | 29.294 ms | 2.183 ms |
| e2e/N=200 | forget (soft) | 8 | 29.868 ms | 34.803 ms | 46.179 ms | 46.179 ms | 5.571 ms |
| e2e/N=200 | forget --restore | 8 | 30.168 ms | 36.223 ms | 40.753 ms | 40.753 ms | 3.350 ms |
| e2e/N=200 | sync | 8 | 25.662 ms | 27.944 ms | 31.596 ms | 31.596 ms | 2.131 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 1.565 ms | 2.353 ms | 3.319 ms | 3.319 ms | 615.51 µs |
| e2e/N=1000 | self version --json | 8 | 1.792 ms | 2.325 ms | 4.030 ms | 4.030 ms | 732.62 µs |
| e2e/N=1000 | --help | 8 | 1.563 ms | 2.380 ms | 3.979 ms | 3.979 ms | 805.24 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 5.959 ms | 7.155 ms | 7.387 ms | 7.387 ms | 449.40 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 17.808 ms | 19.379 ms | 20.536 ms | 20.536 ms | 947.03 µs |
| e2e/N=1000 | prime | 8 | 1.657 ms | 2.246 ms | 3.616 ms | 3.616 ms | 701.57 µs |
| e2e/N=1000 | prime --long | 8 | 1.766 ms | 1.948 ms | 3.629 ms | 3.629 ms | 648.49 µs |
| e2e/N=1000 | ask (query comum) | 8 | 73.967 ms | 82.962 ms | 96.343 ms | 96.343 ms | 7.568 ms |
| e2e/N=1000 | ask (query rara) | 8 | 73.867 ms | 79.402 ms | 82.859 ms | 82.859 ms | 2.840 ms |
| e2e/N=1000 | ask --json | 8 | 72.189 ms | 82.975 ms | 102.677 ms | 102.677 ms | 10.051 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 72.054 ms | 81.447 ms | 100.289 ms | 100.289 ms | 8.839 ms |
| e2e/N=1000 | ask --brief | 8 | 70.646 ms | 77.402 ms | 88.269 ms | 88.269 ms | 5.873 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 72.952 ms | 83.399 ms | 95.025 ms | 95.025 ms | 7.163 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 70.631 ms | 72.046 ms | 77.341 ms | 77.341 ms | 2.275 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 45.995 ms | 53.139 ms | 77.619 ms | 77.619 ms | 10.593 ms |
| e2e/N=1000 | rewind | 8 | 77.504 ms | 88.625 ms | 105.190 ms | 105.190 ms | 10.330 ms |
| e2e/N=1000 | rewind --json | 8 | 80.252 ms | 86.668 ms | 100.870 ms | 100.870 ms | 6.830 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 82.043 ms | 85.310 ms | 95.115 ms | 95.115 ms | 4.015 ms |
| e2e/N=1000 | task list --universe | 8 | 57.112 ms | 63.880 ms | 87.704 ms | 87.704 ms | 9.463 ms |
| e2e/N=1000 | task list --ready | 8 | 56.539 ms | 61.301 ms | 63.407 ms | 63.407 ms | 2.598 ms |
| e2e/N=1000 | task list --sort impact | 8 | 60.838 ms | 70.988 ms | 76.801 ms | 76.801 ms | 6.401 ms |
| e2e/N=1000 | task list --full-content | 8 | 97.118 ms | 103.366 ms | 127.934 ms | 127.934 ms | 10.621 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 68.835 ms | 74.857 ms | 78.432 ms | 78.432 ms | 2.933 ms |
| e2e/N=1000 | task graph | 8 | 62.778 ms | 69.984 ms | 76.939 ms | 76.939 ms | 4.910 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 177.706 ms | 184.510 ms | 194.432 ms | 194.432 ms | 6.911 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 105.448 ms | 116.256 ms | 134.543 ms | 134.543 ms | 9.113 ms |
| e2e/N=1000 | knowledge tags | 8 | 84.085 ms | 89.161 ms | 116.553 ms | 116.553 ms | 12.839 ms |
| e2e/N=1000 | drain --status | 8 | 34.201 ms | 41.596 ms | 57.652 ms | 57.652 ms | 8.837 ms |
| e2e/N=1000 | doctor | 8 | 192.431 ms | 204.482 ms | 234.160 ms | 234.160 ms | 14.668 ms |
| e2e/N=1000 | doctor --explain | 8 | 187.266 ms | 197.746 ms | 204.016 ms | 204.016 ms | 5.777 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 40.195 ms | 48.931 ms | 50.690 ms | 50.690 ms | 3.292 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 92.993 ms | 100.290 ms | 136.173 ms | 136.173 ms | 16.126 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 44.596 ms | 47.846 ms | 60.329 ms | 60.329 ms | 5.054 ms |
| e2e/N=1000 | config list | 8 | 46.578 ms | 52.551 ms | 66.702 ms | 66.702 ms | 7.900 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 46.484 ms | 54.457 ms | 71.556 ms | 71.556 ms | 9.510 ms |
| e2e/N=1000 | write (nova) | 8 | 89.250 ms | 97.710 ms | 126.838 ms | 126.838 ms | 11.558 ms |
| e2e/N=1000 | write (idempotente) | 8 | 82.329 ms | 86.279 ms | 111.168 ms | 111.168 ms | 9.279 ms |
| e2e/N=1000 | config set (projeto) | 8 | 44.982 ms | 58.617 ms | 83.688 ms | 83.688 ms | 12.208 ms |
| e2e/N=1000 | forget (soft) | 8 | 82.413 ms | 87.155 ms | 103.179 ms | 103.179 ms | 6.812 ms |
| e2e/N=1000 | forget --restore | 8 | 85.934 ms | 92.157 ms | 107.959 ms | 107.959 ms | 7.489 ms |
| e2e/N=1000 | sync | 8 | 52.807 ms | 67.642 ms | 83.139 ms | 83.139 ms | 9.928 ms |
