| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 183 ns | 186 ns | 203 ns | 404 ns | 44 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.27 µs | 1.29 µs | 1.31 µs | 1.31 µs | 14 ns |
| micro/fixo | schema::body::body_hash | 25 | 414 ns | 426 ns | 438 ns | 440 ns | 7 ns |
| micro/fixo | schema::id::note_id | 25 | 159 ns | 164 ns | 168 ns | 169 ns | 2 ns |
| micro/fixo | schema::hash::short_hash | 25 | 456 ns | 459 ns | 461 ns | 464 ns | 2 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 18 ns | 18 ns | 23 ns | 24 ns | 2 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.51 µs | 1.53 µs | 1.55 µs | 1.65 µs | 27 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 786 ns | 812 ns | 823 ns | 825 ns | 9 ns |
| micro/fixo | Note::parse (render completo) | 25 | 3.41 µs | 3.43 µs | 3.61 µs | 3.73 µs | 71 ns |
| micro/fixo | Note::render | 25 | 2.06 µs | 2.07 µs | 2.11 µs | 2.13 µs | 16 ns |
| micro/fixo | jsonl::decode | 25 | 796 ns | 803 ns | 820 ns | 838 ns | 9 ns |
| micro/fixo | jsonl::encode | 25 | 575 ns | 582 ns | 588 ns | 588 ns | 3 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 260 ns | 286 ns | 292 ns | 296 ns | 10 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.97 µs | 1.99 µs | 3.09 µs | 3.20 µs | 333 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 116.56 µs | 116.89 µs | 117.83 µs | 119.52 µs | 604 ns |
| micro/fixo | lifecycle::confidence_score | 25 | 4 ns | 5 ns | 5 ns | 5 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 41 ns | 42 ns | 47 ns | 47 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 3.87 µs | 3.98 µs | 4.07 µs | 4.15 µs | 60 ns |
| micro/fixo | config::Config::parse | 25 | 3.04 µs | 3.05 µs | 3.20 µs | 3.86 µs | 162 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 243 ns | 244 ns | 246 ns | 246 ns | 1 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 136 ns | 138 ns | 140 ns | 179 ns | 8 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 29.58 µs | 29.92 µs | 31.45 µs | 32.03 µs | 635 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 803 ns | 804 ns | 815 ns | 818 ns | 4 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.273 ms | 1.292 ms | 1.306 ms | 1.313 ms | 9.89 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 561.25 µs | 577.45 µs | 588.97 µs | 657.56 µs | 21.65 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 218.32 µs | 219.65 µs | 225.73 µs | 229.85 µs | 3.54 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 51.843 ms | 52.475 ms | 53.960 ms | 55.752 ms | 980.63 µs |
| micro/N=200 | write::propose_merges (esparso) | 15 | 259.39 µs | 261.42 µs | 278.25 µs | 309.75 µs | 13.25 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 386.64 µs | 392.44 µs | 408.92 µs | 412.41 µs | 8.07 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 905.15 µs | 916.04 µs | 919.32 µs | 920.44 µs | 5.08 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 17.25 µs | 18.09 µs | 18.93 µs | 19.07 µs | 585 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 147.78 µs | 151.14 µs | 160.29 µs | 163.29 µs | 4.10 µs |
| micro/N=200 | Graph::from_notes | 15 | 362.20 µs | 369.60 µs | 389.44 µs | 393.00 µs | 9.67 µs |
| micro/N=200 | Graph::integrity | 15 | 18.09 µs | 18.51 µs | 18.86 µs | 19.14 µs | 271 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 107.62 µs | 112.93 µs | 114.89 µs | 120.90 µs | 3.00 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 107.56 µs | 113.49 µs | 114.47 µs | 119.92 µs | 3.15 µs |
| micro/N=200 | retrieval::compute_views | 15 | 137.87 µs | 139.06 µs | 145.06 µs | 147.57 µs | 2.75 µs |
| micro/N=200 | task::impact (1 id) | 15 | 17.46 µs | 17.74 µs | 18.58 µs | 19.07 µs | 446 ns |
| micro/N=200 | task::impacts (todos) | 15 | 16.20 µs | 16.55 µs | 18.51 µs | 27.31 µs | 2.79 µs |
| micro/N=200 | handoff::rank (manifest) | 15 | 28.70 µs | 30.03 µs | 31.43 µs | 31.43 µs | 783 ns |
| micro/N=200 | Index::serialize + parse | 15 | 3.954 ms | 4.111 ms | 4.297 ms | 4.599 ms | 167.81 µs |
| micro/N=200 | Index::serialize | 15 | 1.565 ms | 1.582 ms | 1.597 ms | 1.599 ms | 9.79 µs |
| micro/N=200 | Index::parse | 15 | 2.283 ms | 2.310 ms | 2.369 ms | 2.433 ms | 39.67 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.77 µs | 1.80 µs | 1.85 µs | 1.88 µs | 30 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 8.347 ms | 8.487 ms | 9.854 ms | 10.132 ms | 560.95 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.748 ms | 3.799 ms | 4.200 ms | 4.280 ms | 184.42 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.381 ms | 1.512 ms | 1.662 ms | 1.693 ms | 108.67 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 1.458 s | 1.477 s | 1.528 s | 1.533 s | 27.804 ms |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.547 ms | 1.570 ms | 1.706 ms | 1.745 ms | 64.53 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.054 ms | 2.093 ms | 2.217 ms | 2.363 ms | 81.86 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.041 ms | 5.173 ms | 5.389 ms | 6.069 ms | 253.13 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 93.45 µs | 94.22 µs | 95.89 µs | 106.65 µs | 3.26 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 885.38 µs | 910.73 µs | 1.030 ms | 1.056 ms | 53.71 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.608 ms | 2.657 ms | 2.795 ms | 2.812 ms | 64.30 µs |
| micro/N=1000 | Graph::integrity | 15 | 139.75 µs | 143.38 µs | 154.98 µs | 156.38 µs | 4.80 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 621.52 µs | 631.86 µs | 646.52 µs | 649.32 µs | 8.29 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 611.18 µs | 633.11 µs | 643.38 µs | 652.39 µs | 11.00 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 789.35 µs | 817.08 µs | 830.83 µs | 831.04 µs | 13.19 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 148.83 µs | 152.81 µs | 164.69 µs | 164.83 µs | 4.99 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 142.69 µs | 146.18 µs | 157.63 µs | 163.99 µs | 6.06 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 172.93 µs | 176.56 µs | 187.46 µs | 190.46 µs | 4.91 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.335 ms | 21.542 ms | 21.932 ms | 23.042 ms | 424.81 µs |
| micro/N=1000 | Index::serialize | 15 | 8.257 ms | 8.385 ms | 8.734 ms | 10.018 ms | 433.00 µs |
| micro/N=1000 | Index::parse | 15 | 12.329 ms | 12.738 ms | 13.159 ms | 13.182 ms | 295.35 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.76 µs | 1.80 µs | 1.84 µs | 1.87 µs | 32 ns |
| e2e/N=200 | self version (piso de startup) | 8 | 1.805 ms | 3.966 ms | 4.079 ms | 4.079 ms | 915.62 µs |
| e2e/N=200 | self version --json | 8 | 1.779 ms | 3.831 ms | 4.056 ms | 4.056 ms | 982.61 µs |
| e2e/N=200 | --help | 8 | 2.068 ms | 3.940 ms | 4.041 ms | 4.041 ms | 755.37 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 3.938 ms | 4.210 ms | 11.430 ms | 11.430 ms | 3.630 ms |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 5.544 ms | 5.596 ms | 5.660 ms | 5.660 ms | 36.55 µs |
| e2e/N=200 | prime | 8 | 2.387 ms | 3.674 ms | 4.140 ms | 4.140 ms | 512.21 µs |
| e2e/N=200 | prime --long | 8 | 1.915 ms | 3.120 ms | 3.362 ms | 3.362 ms | 532.54 µs |
| e2e/N=200 | ask (query comum) | 8 | 33.413 ms | 43.992 ms | 52.959 ms | 52.959 ms | 6.658 ms |
| e2e/N=200 | ask (query rara) | 8 | 32.656 ms | 42.891 ms | 51.692 ms | 51.692 ms | 6.016 ms |
| e2e/N=200 | ask --json | 8 | 35.416 ms | 53.750 ms | 60.141 ms | 60.141 ms | 8.767 ms |
| e2e/N=200 | ask --limit 50 | 8 | 35.215 ms | 48.435 ms | 61.841 ms | 61.841 ms | 9.756 ms |
| e2e/N=200 | ask --brief | 8 | 35.118 ms | 49.261 ms | 53.716 ms | 53.716 ms | 7.971 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 36.344 ms | 51.750 ms | 63.972 ms | 63.972 ms | 8.458 ms |
| e2e/N=200 | ask --around <nota> | 8 | 31.311 ms | 43.046 ms | 57.947 ms | 57.947 ms | 8.526 ms |
| e2e/N=200 | ask --id <nota> | 8 | 28.730 ms | 34.029 ms | 36.384 ms | 36.384 ms | 2.864 ms |
| e2e/N=200 | rewind | 8 | 38.801 ms | 45.618 ms | 66.784 ms | 66.784 ms | 9.913 ms |
| e2e/N=200 | rewind --json | 8 | 40.196 ms | 49.332 ms | 60.723 ms | 60.723 ms | 6.644 ms |
| e2e/N=200 | rewind --files src/core/** | 8 | 38.282 ms | 46.027 ms | 61.484 ms | 61.484 ms | 7.174 ms |
| e2e/N=200 | task list --universe | 8 | 28.681 ms | 37.409 ms | 58.837 ms | 58.837 ms | 10.881 ms |
| e2e/N=200 | task list --ready | 8 | 30.543 ms | 43.817 ms | 55.262 ms | 55.262 ms | 8.573 ms |
| e2e/N=200 | task list --sort impact | 8 | 32.841 ms | 38.589 ms | 46.612 ms | 46.612 ms | 5.387 ms |
| e2e/N=200 | task list --full-content | 8 | 35.289 ms | 47.167 ms | 57.278 ms | 57.278 ms | 7.264 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 28.629 ms | 33.779 ms | 43.499 ms | 43.499 ms | 5.330 ms |
| e2e/N=200 | task graph | 8 | 34.015 ms | 45.207 ms | 48.734 ms | 48.734 ms | 5.697 ms |
| e2e/N=200 | knowledge map --universe | 8 | 49.834 ms | 63.307 ms | 82.743 ms | 82.743 ms | 10.967 ms |
| e2e/N=200 | knowledge rank --universe | 8 | 36.063 ms | 42.013 ms | 58.624 ms | 58.624 ms | 7.771 ms |
| e2e/N=200 | knowledge tags | 8 | 33.797 ms | 38.343 ms | 50.733 ms | 50.733 ms | 6.277 ms |
| e2e/N=200 | drain --status | 8 | 14.973 ms | 19.508 ms | 25.139 ms | 25.139 ms | 2.914 ms |
| e2e/N=200 | doctor | 8 | 164.989 ms | 167.596 ms | 180.284 ms | 180.284 ms | 6.508 ms |
| e2e/N=200 | doctor --explain | 8 | 164.203 ms | 174.779 ms | 188.907 ms | 188.907 ms | 8.762 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 28.592 ms | 31.826 ms | 47.964 ms | 47.964 ms | 6.299 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 89.005 ms | 102.007 ms | 107.057 ms | 107.057 ms | 7.378 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 31.055 ms | 33.652 ms | 51.268 ms | 51.268 ms | 7.735 ms |
| e2e/N=200 | config list | 8 | 29.280 ms | 33.008 ms | 45.527 ms | 45.527 ms | 6.357 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 26.210 ms | 28.794 ms | 35.107 ms | 35.107 ms | 3.170 ms |
| e2e/N=200 | write (nova) | 8 | 35.065 ms | 47.514 ms | 54.273 ms | 54.273 ms | 6.759 ms |
| e2e/N=200 | write (idempotente) | 8 | 29.699 ms | 37.875 ms | 42.283 ms | 42.283 ms | 4.330 ms |
| e2e/N=200 | config set (projeto) | 8 | 28.734 ms | 32.685 ms | 42.062 ms | 42.062 ms | 4.634 ms |
| e2e/N=200 | forget (soft) | 8 | 35.514 ms | 46.132 ms | 62.703 ms | 62.703 ms | 9.120 ms |
| e2e/N=200 | forget --restore | 8 | 31.101 ms | 48.549 ms | 60.504 ms | 60.504 ms | 10.912 ms |
| e2e/N=200 | sync | 8 | 38.025 ms | 46.053 ms | 53.089 ms | 53.089 ms | 6.266 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 2.379 ms | 3.938 ms | 4.007 ms | 4.007 ms | 681.06 µs |
| e2e/N=1000 | self version --json | 8 | 3.402 ms | 4.001 ms | 5.028 ms | 5.028 ms | 454.18 µs |
| e2e/N=1000 | --help | 8 | 3.922 ms | 4.010 ms | 4.091 ms | 4.091 ms | 51.59 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 8.081 ms | 8.699 ms | 9.921 ms | 9.921 ms | 621.50 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 18.632 ms | 19.342 ms | 19.904 ms | 19.904 ms | 379.30 µs |
| e2e/N=1000 | prime | 8 | 1.712 ms | 2.278 ms | 3.194 ms | 3.194 ms | 583.19 µs |
| e2e/N=1000 | prime --long | 8 | 1.742 ms | 2.872 ms | 3.911 ms | 3.911 ms | 808.63 µs |
| e2e/N=1000 | ask (query comum) | 8 | 73.025 ms | 80.404 ms | 83.639 ms | 83.639 ms | 4.336 ms |
| e2e/N=1000 | ask (query rara) | 8 | 72.599 ms | 84.844 ms | 97.150 ms | 97.150 ms | 8.733 ms |
| e2e/N=1000 | ask --json | 8 | 73.796 ms | 82.755 ms | 91.176 ms | 91.176 ms | 6.424 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 74.856 ms | 82.187 ms | 92.010 ms | 92.010 ms | 6.330 ms |
| e2e/N=1000 | ask --brief | 8 | 74.437 ms | 86.259 ms | 101.929 ms | 101.929 ms | 9.125 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 75.252 ms | 86.481 ms | 107.741 ms | 107.741 ms | 11.486 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 78.435 ms | 91.034 ms | 93.430 ms | 93.430 ms | 4.735 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 51.124 ms | 75.891 ms | 81.123 ms | 81.123 ms | 12.990 ms |
| e2e/N=1000 | rewind | 8 | 84.320 ms | 92.263 ms | 97.650 ms | 97.650 ms | 5.533 ms |
| e2e/N=1000 | rewind --json | 8 | 83.748 ms | 99.042 ms | 106.394 ms | 106.394 ms | 8.501 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 79.659 ms | 90.854 ms | 102.362 ms | 102.362 ms | 7.282 ms |
| e2e/N=1000 | task list --universe | 8 | 60.090 ms | 72.697 ms | 91.534 ms | 91.534 ms | 12.228 ms |
| e2e/N=1000 | task list --ready | 8 | 58.914 ms | 69.450 ms | 91.951 ms | 91.951 ms | 11.729 ms |
| e2e/N=1000 | task list --sort impact | 8 | 57.345 ms | 66.585 ms | 91.536 ms | 91.536 ms | 11.806 ms |
| e2e/N=1000 | task list --full-content | 8 | 101.762 ms | 106.136 ms | 117.252 ms | 117.252 ms | 5.400 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 69.960 ms | 87.516 ms | 96.877 ms | 96.877 ms | 9.971 ms |
| e2e/N=1000 | task graph | 8 | 67.992 ms | 81.759 ms | 94.722 ms | 94.722 ms | 8.777 ms |
| e2e/N=1000 | knowledge map --universe | 8 | 170.518 ms | 178.748 ms | 204.323 ms | 204.323 ms | 11.338 ms |
| e2e/N=1000 | knowledge rank --universe | 8 | 101.331 ms | 108.296 ms | 128.858 ms | 128.858 ms | 8.808 ms |
| e2e/N=1000 | knowledge tags | 8 | 82.359 ms | 90.305 ms | 101.170 ms | 101.170 ms | 6.268 ms |
| e2e/N=1000 | drain --status | 8 | 34.699 ms | 39.511 ms | 56.816 ms | 56.816 ms | 8.012 ms |
| e2e/N=1000 | doctor | 8 | 4.002 s | 4.105 s | 4.201 s | 4.201 s | 69.468 ms |
| e2e/N=1000 | doctor --explain | 8 | 3.990 s | 4.183 s | 4.322 s | 4.322 s | 107.219 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 48.421 ms | 57.586 ms | 70.527 ms | 70.527 ms | 7.511 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 2.024 s | 2.080 s | 2.117 s | 2.117 s | 38.034 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 113.795 ms | 125.347 ms | 130.719 ms | 130.719 ms | 5.786 ms |
| e2e/N=1000 | config list | 8 | 52.045 ms | 62.341 ms | 71.618 ms | 71.618 ms | 8.034 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 52.049 ms | 74.572 ms | 80.323 ms | 80.323 ms | 11.090 ms |
| e2e/N=1000 | write (nova) | 8 | 87.796 ms | 94.860 ms | 111.834 ms | 111.834 ms | 9.632 ms |
| e2e/N=1000 | write (idempotente) | 8 | 82.423 ms | 98.172 ms | 109.438 ms | 109.438 ms | 9.651 ms |
| e2e/N=1000 | config set (projeto) | 8 | 53.531 ms | 70.441 ms | 79.655 ms | 79.655 ms | 10.320 ms |
| e2e/N=1000 | forget (soft) | 8 | 84.557 ms | 88.243 ms | 97.368 ms | 97.368 ms | 4.344 ms |
| e2e/N=1000 | forget --restore | 8 | 81.356 ms | 87.575 ms | 99.528 ms | 99.528 ms | 6.230 ms |
| e2e/N=1000 | sync | 8 | 57.500 ms | 79.584 ms | 91.996 ms | 91.996 ms | 10.959 ms |
