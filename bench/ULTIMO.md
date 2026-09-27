| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 182 ns | 188 ns | 205 ns | 207 ns | 7 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.27 µs | 1.30 µs | 1.33 µs | 1.35 µs | 18 ns |
| micro/fixo | schema::body::body_hash | 25 | 450 ns | 453 ns | 458 ns | 459 ns | 2 ns |
| micro/fixo | schema::id::note_id | 25 | 190 ns | 192 ns | 198 ns | 199 ns | 3 ns |
| micro/fixo | schema::hash::short_hash | 25 | 456 ns | 458 ns | 464 ns | 466 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 17 ns | 18 ns | 18 ns | 18 ns | 0 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.53 µs | 1.55 µs | 1.58 µs | 1.67 µs | 26 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 822 ns | 845 ns | 860 ns | 864 ns | 9 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.41 µs | 3.45 µs | 3.49 µs | 3.79 µs | 70 ns |
| micro/fixo | Note::render | 25 | 2.08 µs | 2.10 µs | 2.14 µs | 2.15 µs | 15 ns |
| micro/fixo | jsonl::decode | 25 | 809 ns | 821 ns | 835 ns | 943 ns | 25 ns |
| micro/fixo | jsonl::encode | 25 | 531 ns | 539 ns | 549 ns | 550 ns | 5 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 247 ns | 270 ns | 281 ns | 283 ns | 9 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 2.58 µs | 2.60 µs | 2.64 µs | 2.67 µs | 18 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.92 µs | 1.95 µs | 1.97 µs | 1.99 µs | 16 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 115.40 µs | 116.61 µs | 119.13 µs | 122.29 µs | 1.52 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 13 ns | 14 ns | 15 ns | 16 ns | 1 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 6 ns | 6 ns | 6 ns | 6 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 40 ns | 40 ns | 46 ns | 47 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.72 µs | 3.81 µs | 3.92 µs | 3.96 µs | 60 ns |
| micro/fixo | config::Config::parse | 25 | 3.17 µs | 3.19 µs | 3.54 µs | 3.86 µs | 149 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 232 ns | 233 ns | 238 ns | 238 ns | 2 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 137 ns | 138 ns | 150 ns | 151 ns | 4 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.73 µs | 29.81 µs | 30.56 µs | 31.23 µs | 317 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 801 ns | 803 ns | 803 ns | 804 ns | 1 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.316 ms | 1.408 ms | 1.478 ms | 1.635 ms | 80.84 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 552.38 µs | 569.84 µs | 580.31 µs | 584.43 µs | 10.10 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 222.31 µs | 228.59 µs | 239.63 µs | 243.19 µs | 5.77 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 50.884 ms | 51.191 ms | 51.381 ms | 51.381 ms | 145.77 µs |
| micro/N=200 | write::propose_merges (esparso) | 15 | 262.88 µs | 269.24 µs | 280.76 µs | 281.81 µs | 6.00 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 344.53 µs | 351.37 µs | 366.39 µs | 367.44 µs | 8.06 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 854.86 µs | 871.48 µs | 900.12 µs | 992.87 µs | 32.97 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 18.86 µs | 19.21 µs | 20.39 µs | 20.60 µs | 535 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 148.69 µs | 150.44 µs | 156.79 µs | 158.61 µs | 2.89 µs |
| micro/N=200 | Graph::from_notes | 15 | 365.13 µs | 370.72 µs | 386.43 µs | 387.06 µs | 8.12 µs |
| micro/N=200 | Graph::integrity | 15 | 18.02 µs | 18.58 µs | 18.72 µs | 18.79 µs | 213 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 107.63 µs | 113.28 µs | 120.13 µs | 120.69 µs | 3.97 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 105.67 µs | 112.24 µs | 118.80 µs | 120.62 µs | 4.03 µs |
| micro/N=200 | retrieval::compute_views | 15 | 133.19 µs | 137.38 µs | 145.62 µs | 146.11 µs | 4.27 µs |
| micro/N=200 | task::impact (1 id) | 15 | 17.53 µs | 17.95 µs | 18.58 µs | 18.79 µs | 397 ns |
| micro/N=200 | task::impacts (todos) | 15 | 16.34 µs | 16.83 µs | 18.09 µs | 18.51 µs | 657 ns |
| micro/N=200 | handoff::rank (manifest) | 15 | 29.61 µs | 30.52 µs | 31.50 µs | 31.99 µs | 774 ns |
| micro/N=200 | Index::serialize + parse | 15 | 4.129 ms | 4.162 ms | 4.414 ms | 4.443 ms | 112.83 µs |
| micro/N=200 | Index::serialize | 15 | 1.609 ms | 1.635 ms | 1.644 ms | 1.648 ms | 9.95 µs |
| micro/N=200 | Index::parse | 15 | 2.349 ms | 2.377 ms | 2.417 ms | 2.428 ms | 19.29 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.81 µs | 1.85 µs | 1.92 µs | 1.93 µs | 38 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.522 ms | 8.673 ms | 9.621 ms | 9.680 ms | 369.20 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.737 ms | 3.860 ms | 4.231 ms | 4.268 ms | 187.74 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.416 ms | 1.451 ms | 1.783 ms | 1.786 ms | 146.60 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.446 s | 1.524 s | 1.617 s | 1.711 s | 65.499 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.568 ms | 1.640 ms | 1.743 ms | 1.954 ms | 98.08 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.154 ms | 2.410 ms | 2.924 ms | 2.954 ms | 256.40 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.293 ms | 6.006 ms | 7.048 ms | 7.491 ms | 594.25 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 96.80 µs | 98.20 µs | 110.77 µs | 124.11 µs | 7.67 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 886.08 µs | 945.72 µs | 1.009 ms | 1.039 ms | 50.21 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.827 ms | 3.257 ms | 3.813 ms | 4.105 ms | 333.26 µs |
| micro/N=1000 | Graph::integrity | 15 | 132.07 µs | 137.03 µs | 150.09 µs | 152.46 µs | 5.80 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 610.49 µs | 627.95 µs | 986.44 µs | 1.088 ms | 144.03 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 597.84 µs | 615.09 µs | 645.13 µs | 645.68 µs | 14.32 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 776.36 µs | 807.93 µs | 879.86 µs | 967.38 µs | 47.06 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 144.08 µs | 148.27 µs | 160.99 µs | 163.36 µs | 5.79 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 144.71 µs | 145.20 µs | 159.31 µs | 160.15 µs | 5.45 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 179.14 µs | 182.36 µs | 196.12 µs | 201.07 µs | 6.63 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.997 ms | 22.840 ms | 23.662 ms | 23.904 ms | 534.31 µs |
| micro/N=1000 | Index::serialize | 15 | 8.538 ms | 8.913 ms | 10.234 ms | 10.726 ms | 618.55 µs |
| micro/N=1000 | Index::parse | 15 | 13.310 ms | 13.980 ms | 14.849 ms | 15.133 ms | 560.73 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.79 µs | 1.86 µs | 2.56 µs | 2.65 µs | 336 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 1.946 ms | 3.992 ms | 4.074 ms | 4.074 ms | 875.15 µs |
| e2e/N=200 | self version --json | 8 | 1.994 ms | 2.112 ms | 4.447 ms | 4.447 ms | 1.052 ms |
| e2e/N=200 | --help | 8 | 1.839 ms | 2.519 ms | 4.491 ms | 4.491 ms | 879.74 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 3.983 ms | 4.191 ms | 11.693 ms | 11.693 ms | 3.683 ms |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.476 ms | 5.520 ms | 5.966 ms | 5.966 ms | 164.66 µs |
| e2e/N=200 | prime | 8 | 1.855 ms | 2.344 ms | 3.883 ms | 3.883 ms | 681.39 µs |
| e2e/N=200 | prime --long | 8 | 1.798 ms | 2.514 ms | 3.873 ms | 3.873 ms | 664.43 µs |
| e2e/N=200 | ask (query comum) | 8 | 35.123 ms | 39.964 ms | 60.329 ms | 60.329 ms | 9.245 ms |
| e2e/N=200 | ask (query rara) | 8 | 29.928 ms | 33.503 ms | 34.838 ms | 34.838 ms | 1.812 ms |
| e2e/N=200 | ask --json | 8 | 33.284 ms | 37.243 ms | 48.852 ms | 48.852 ms | 5.033 ms |
| e2e/N=200 | ask --limit 50 | 8 | 31.389 ms | 41.014 ms | 58.020 ms | 58.020 ms | 9.652 ms |
| e2e/N=200 | ask --brief | 8 | 32.917 ms | 35.005 ms | 37.165 ms | 37.165 ms | 1.493 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 33.263 ms | 37.013 ms | 50.123 ms | 50.123 ms | 6.370 ms |
| e2e/N=200 | ask --around <nota> | 8 | 36.206 ms | 45.219 ms | 57.049 ms | 57.049 ms | 6.338 ms |
| e2e/N=200 | ask --id <nota> | 8 | 28.692 ms | 32.484 ms | 42.480 ms | 42.480 ms | 5.043 ms |
| e2e/N=200 | rewind | 8 | 36.563 ms | 58.419 ms | 64.555 ms | 64.555 ms | 10.751 ms |
| e2e/N=200 | rewind --json | 8 | 41.332 ms | 54.644 ms | 60.327 ms | 60.327 ms | 7.423 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 38.613 ms | 53.734 ms | 61.943 ms | 61.943 ms | 8.087 ms |
| e2e/N=200 | task list --universe | 8 | 32.371 ms | 36.274 ms | 43.001 ms | 43.001 ms | 3.569 ms |
| e2e/N=200 | task list --ready | 8 | 39.268 ms | 45.718 ms | 56.822 ms | 56.822 ms | 6.394 ms |
| e2e/N=200 | task list --sort impact | 8 | 36.161 ms | 52.473 ms | 59.633 ms | 59.633 ms | 8.566 ms |
| e2e/N=200 | task list --full-content | 8 | 37.895 ms | 46.871 ms | 54.300 ms | 54.300 ms | 5.587 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 40.004 ms | 45.815 ms | 57.717 ms | 57.717 ms | 5.882 ms |
| e2e/N=200 | task graph | 8 | 29.676 ms | 35.179 ms | 43.523 ms | 43.523 ms | 4.562 ms |
| e2e/N=200 | knowledge map --universe | 8 | 51.752 ms | 62.615 ms | 68.218 ms | 68.218 ms | 5.692 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 37.766 ms | 44.412 ms | 56.136 ms | 56.136 ms | 6.565 ms |
| e2e/N=200 | knowledge tags | 8 | 29.508 ms | 33.302 ms | 36.097 ms | 36.097 ms | 2.192 ms |
| e2e/N=200 | drain --status | 8 | 14.214 ms | 18.242 ms | 22.867 ms | 22.867 ms | 2.784 ms |
| e2e/N=200 | doctor | 8 | 160.806 ms | 171.148 ms | 184.785 ms | 184.785 ms | 8.593 ms |
| e2e/N=200 | doctor --explain | 8 | 160.078 ms | 170.316 ms | 178.672 ms | 178.672 ms | 6.130 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 25.071 ms | 35.927 ms | 45.226 ms | 45.226 ms | 6.778 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 93.391 ms | 102.096 ms | 105.600 ms | 105.600 ms | 4.348 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 30.098 ms | 36.616 ms | 46.858 ms | 46.858 ms | 6.214 ms |
| e2e/N=200 | config list | 8 | 21.767 ms | 31.355 ms | 35.488 ms | 35.488 ms | 5.227 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 20.145 ms | 27.128 ms | 37.084 ms | 37.084 ms | 5.927 ms |
| e2e/N=200 | write (nova) | 8 | 30.107 ms | 43.417 ms | 46.356 ms | 46.356 ms | 6.118 ms |
| e2e/N=200 | write (idempotente) | 8 | 29.864 ms | 34.676 ms | 44.408 ms | 44.408 ms | 4.642 ms |
| e2e/N=200 | config set (projeto) | 8 | 27.921 ms | 30.209 ms | 34.639 ms | 34.639 ms | 2.080 ms |
| e2e/N=200 | forget (soft) | 8 | 29.415 ms | 44.970 ms | 49.283 ms | 49.283 ms | 7.316 ms |
| e2e/N=200 | forget --restore | 8 | 35.999 ms | 46.900 ms | 56.544 ms | 56.544 ms | 7.964 ms |
| e2e/N=200 | sync | 8 | 27.140 ms | 31.592 ms | 37.508 ms | 37.508 ms | 3.709 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 2.187 ms | 3.840 ms | 4.370 ms | 4.370 ms | 903.10 µs |
| e2e/N=1000 | self version --json | 8 | 2.210 ms | 2.599 ms | 3.946 ms | 3.946 ms | 657.22 µs |
| e2e/N=1000 | --help | 8 | 2.080 ms | 3.239 ms | 3.908 ms | 3.908 ms | 623.38 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 6.194 ms | 7.374 ms | 8.446 ms | 8.446 ms | 760.18 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 17.143 ms | 17.788 ms | 19.981 ms | 19.981 ms | 887.85 µs |
| e2e/N=1000 | prime | 8 | 1.970 ms | 2.022 ms | 3.374 ms | 3.374 ms | 482.25 µs |
| e2e/N=1000 | prime --long | 8 | 1.921 ms | 2.704 ms | 3.839 ms | 3.839 ms | 638.41 µs |
| e2e/N=1000 | ask (query comum) | 8 | 72.724 ms | 79.910 ms | 94.237 ms | 94.237 ms | 7.305 ms |
| e2e/N=1000 | ask (query rara) | 8 | 73.613 ms | 80.959 ms | 88.960 ms | 88.960 ms | 4.830 ms |
| e2e/N=1000 | ask --json | 8 | 69.780 ms | 78.002 ms | 89.809 ms | 89.809 ms | 6.445 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 70.348 ms | 75.826 ms | 83.471 ms | 83.471 ms | 4.213 ms |
| e2e/N=1000 | ask --brief | 8 | 70.955 ms | 80.475 ms | 91.099 ms | 91.099 ms | 7.069 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 74.093 ms | 79.783 ms | 98.439 ms | 98.439 ms | 8.132 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 68.363 ms | 74.461 ms | 77.990 ms | 77.990 ms | 3.661 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 45.403 ms | 51.754 ms | 69.735 ms | 69.735 ms | 9.420 ms |
| e2e/N=1000 | rewind | 8 | 77.070 ms | 87.103 ms | 102.296 ms | 102.296 ms | 7.827 ms |
| e2e/N=1000 | rewind --json | 8 | 79.379 ms | 85.534 ms | 94.659 ms | 94.659 ms | 4.231 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 77.920 ms | 87.858 ms | 89.952 ms | 89.952 ms | 4.243 ms |
| e2e/N=1000 | task list --universe | 8 | 58.962 ms | 71.021 ms | 83.496 ms | 83.496 ms | 7.347 ms |
| e2e/N=1000 | task list --ready | 8 | 57.650 ms | 69.719 ms | 87.467 ms | 87.467 ms | 9.213 ms |
| e2e/N=1000 | task list --sort impact | 8 | 57.014 ms | 64.022 ms | 73.272 ms | 73.272 ms | 5.081 ms |
| e2e/N=1000 | task list --full-content | 8 | 94.890 ms | 102.452 ms | 113.213 ms | 113.213 ms | 6.264 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 71.877 ms | 77.964 ms | 103.142 ms | 103.142 ms | 10.762 ms |
| e2e/N=1000 | task graph | 8 | 64.851 ms | 78.499 ms | 91.942 ms | 91.942 ms | 8.360 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 174.139 ms | 178.946 ms | 193.760 ms | 193.760 ms | 6.023 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 104.479 ms | 121.050 ms | 123.300 ms | 123.300 ms | 7.115 ms |
| e2e/N=1000 | knowledge tags | 8 | 84.162 ms | 89.117 ms | 102.292 ms | 102.292 ms | 6.990 ms |
| e2e/N=1000 | drain --status | 8 | 36.224 ms | 37.528 ms | 51.136 ms | 51.136 ms | 5.939 ms |
| e2e/N=1000 | doctor | 8 | 4.041 s | 4.157 s | 4.238 s | 4.238 s | 61.241 ms |
| e2e/N=1000 | doctor --explain | 8 | 3.995 s | 4.201 s | 4.274 s | 4.274 s | 88.114 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 41.719 ms | 54.176 ms | 59.212 ms | 59.212 ms | 5.943 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 1.990 s | 2.078 s | 2.137 s | 2.137 s | 59.949 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 112.655 ms | 123.104 ms | 136.768 ms | 136.768 ms | 7.615 ms |
| e2e/N=1000 | config list | 8 | 44.071 ms | 65.956 ms | 76.674 ms | 76.674 ms | 11.633 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 51.404 ms | 54.958 ms | 72.946 ms | 72.946 ms | 7.971 ms |
| e2e/N=1000 | write (nova) | 8 | 89.755 ms | 95.330 ms | 111.510 ms | 111.510 ms | 7.809 ms |
| e2e/N=1000 | write (idempotente) | 8 | 86.232 ms | 96.882 ms | 100.150 ms | 100.150 ms | 5.356 ms |
| e2e/N=1000 | config set (projeto) | 8 | 50.461 ms | 62.348 ms | 66.168 ms | 66.168 ms | 5.722 ms |
| e2e/N=1000 | forget (soft) | 8 | 79.687 ms | 96.990 ms | 113.849 ms | 113.849 ms | 10.590 ms |
| e2e/N=1000 | forget --restore | 8 | 81.094 ms | 85.489 ms | 97.691 ms | 97.691 ms | 6.293 ms |
| e2e/N=1000 | sync | 8 | 63.828 ms | 80.231 ms | 87.883 ms | 87.883 ms | 9.483 ms |
