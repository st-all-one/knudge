| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 3.23 µs | 3.27 µs | 3.37 µs | 3.39 µs | 45 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 23.75 µs | 23.87 µs | 24.51 µs | 25.26 µs | 311 ns |
| micro/fixo | schema::body::body_hash | 25 | 4.89 µs | 4.98 µs | 5.10 µs | 5.23 µs | 60 ns |
| micro/fixo | schema::id::note_id | 25 | 1.96 µs | 1.98 µs | 2.04 µs | 2.24 µs | 54 ns |
| micro/fixo | schema::hash::short_hash | 25 | 763 ns | 769 ns | 771 ns | 791 ns | 5 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 58 ns | 59 ns | 60 ns | 61 ns | 1 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 3.50 µs | 3.53 µs | 3.61 µs | 3.63 µs | 31 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 1.37 µs | 1.38 µs | 1.40 µs | 1.41 µs | 12 ns |
| micro/fixo | Note::parse (render completo) | 25 | 6.55 µs | 6.59 µs | 6.66 µs | 6.66 µs | 26 ns |
| micro/fixo | Note::render | 25 | 3.41 µs | 3.45 µs | 3.49 µs | 3.54 µs | 26 ns |
| micro/fixo | jsonl::decode | 25 | 1.38 µs | 1.39 µs | 1.40 µs | 1.41 µs | 7 ns |
| micro/fixo | jsonl::encode | 25 | 1.01 µs | 1.02 µs | 1.02 µs | 1.04 µs | 5 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 376 ns | 383 ns | 398 ns | 403 ns | 7 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 3.48 µs | 3.51 µs | 3.52 µs | 3.56 µs | 15 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 178.95 µs | 180.26 µs | 189.81 µs | 199.30 µs | 4.25 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 7 ns | 7 ns | 7 ns | 8 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 65 ns | 67 ns | 75 ns | 77 ns | 3 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 6.49 µs | 6.57 µs | 6.64 µs | 6.68 µs | 50 ns |
| micro/fixo | config::Config::parse | 25 | 5.15 µs | 5.18 µs | 5.22 µs | 5.22 µs | 21 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 49.18 µs | 49.30 µs | 49.55 µs | 49.59 µs | 93 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 1.17 µs | 1.33 µs | 1.33 µs | 1.34 µs | 38 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.774 ms | 1.804 ms | 1.933 ms | 1.992 ms | 66.97 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 302.69 µs | 306.19 µs | 330.56 µs | 341.11 µs | 11.87 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 455.50 µs | 463.89 µs | 476.39 µs | 476.67 µs | 8.66 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 1.071 ms | 1.083 ms | 1.103 ms | 1.105 ms | 11.34 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 24.23 µs | 24.79 µs | 26.47 µs | 35.48 µs | 2.81 µs |
| micro/N=200 | lifecycle::structural_clusters | 15 | 445.31 µs | 452.22 µs | 466.54 µs | 485.68 µs | 10.32 µs |
| micro/N=200 | Graph::from_notes | 15 | 482.11 µs | 497.69 µs | 501.25 µs | 502.09 µs | 8.28 µs |
| micro/N=200 | Graph::integrity | 15 | 24.51 µs | 25.21 µs | 25.77 µs | 26.05 µs | 426 ns |
| micro/N=200 | Graph::supersession_cycles | 15 | 148.20 µs | 152.95 µs | 160.35 µs | 164.20 µs | 4.03 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 148.83 µs | 152.95 µs | 161.40 µs | 162.66 µs | 3.73 µs |
| micro/N=200 | Index::serialize + parse | 15 | 5.200 ms | 5.304 ms | 5.493 ms | 5.537 ms | 116.42 µs |
| micro/N=200 | write::Draft::to_note | 15 | 4.46 µs | 4.58 µs | 4.79 µs | 4.93 µs | 115 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 9.412 ms | 9.708 ms | 10.406 ms | 10.808 ms | 416.04 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 1.480 ms | 1.530 ms | 1.743 ms | 1.842 ms | 102.37 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 2.120 ms | 2.162 ms | 2.221 ms | 2.265 ms | 43.16 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 5.053 ms | 5.167 ms | 5.910 ms | 5.936 ms | 294.62 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 103.92 µs | 108.81 µs | 116.29 µs | 118.80 µs | 3.75 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 6.789 ms | 7.051 ms | 7.368 ms | 8.995 ms | 520.65 µs |
| micro/N=1000 | Graph::from_notes | 15 | 2.569 ms | 2.730 ms | 3.060 ms | 3.378 ms | 235.46 µs |
| micro/N=1000 | Graph::integrity | 15 | 136.68 µs | 139.05 µs | 142.55 µs | 149.32 µs | 2.96 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 615.72 µs | 626.34 µs | 644.08 µs | 645.33 µs | 9.57 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 612.86 µs | 627.66 µs | 638.00 µs | 638.84 µs | 8.44 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 21.288 ms | 21.819 ms | 22.145 ms | 22.168 ms | 294.80 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 3.46 µs | 3.52 µs | 3.57 µs | 3.67 µs | 55 ns |
| e2e/N=200 | self version (piso de startup) | 10 | 17.132 ms | 27.044 ms | 30.674 ms | 30.674 ms | 5.173 ms |
| e2e/N=200 | self version --json | 10 | 19.834 ms | 28.638 ms | 30.985 ms | 30.985 ms | 4.309 ms |
| e2e/N=200 | --help | 10 | 2.791 ms | 3.838 ms | 3.989 ms | 3.989 ms | 339.40 µs |
| e2e/N=200 | prime | 10 | 19.402 ms | 28.509 ms | 31.123 ms | 31.123 ms | 4.838 ms |
| e2e/N=200 | prime --long | 10 | 19.061 ms | 21.345 ms | 31.211 ms | 31.211 ms | 4.813 ms |
| e2e/N=200 | ask (query comum) | 10 | 40.300 ms | 43.893 ms | 60.677 ms | 60.677 ms | 7.716 ms |
| e2e/N=200 | ask (query rara) | 10 | 39.735 ms | 47.657 ms | 61.723 ms | 61.723 ms | 6.267 ms |
| e2e/N=200 | ask --json | 10 | 40.555 ms | 48.910 ms | 60.290 ms | 60.290 ms | 8.098 ms |
| e2e/N=200 | ask --limit 50 | 10 | 44.563 ms | 59.524 ms | 80.411 ms | 80.411 ms | 11.678 ms |
| e2e/N=200 | ask --brief | 10 | 41.248 ms | 56.927 ms | 70.027 ms | 70.027 ms | 8.418 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 10 | 45.614 ms | 57.691 ms | 78.512 ms | 78.512 ms | 9.117 ms |
| e2e/N=200 | ask --around <nota> | 10 | 36.578 ms | 40.253 ms | 59.589 ms | 59.589 ms | 7.050 ms |
| e2e/N=200 | ask --id <nota> | 10 | 32.319 ms | 36.313 ms | 41.290 ms | 41.290 ms | 2.945 ms |
| e2e/N=200 | rewind | 10 | 69.602 ms | 80.711 ms | 85.697 ms | 85.697 ms | 5.444 ms |
| e2e/N=200 | rewind --json | 10 | 76.319 ms | 80.145 ms | 87.343 ms | 87.343 ms | 3.035 ms |
| e2e/N=200 | rewind --files src/core/** | 10 | 74.212 ms | 79.577 ms | 89.676 ms | 89.676 ms | 5.468 ms |
| e2e/N=200 | task list | 10 | 15.082 ms | 16.815 ms | 19.115 ms | 19.115 ms | 1.399 ms |
| e2e/N=200 | task list --ready | 10 | 49.246 ms | 64.314 ms | 72.798 ms | 72.798 ms | 7.249 ms |
| e2e/N=200 | task list --sort impact | 10 | 15.078 ms | 17.447 ms | 19.624 ms | 19.624 ms | 1.205 ms |
| e2e/N=200 | task list --full-content | 10 | 16.963 ms | 17.562 ms | 19.502 ms | 19.502 ms | 777.00 µs |
| e2e/N=200 | task show --id <tarefa> | 10 | 43.581 ms | 47.733 ms | 56.170 ms | 56.170 ms | 3.756 ms |
| e2e/N=200 | task graph | 10 | 53.071 ms | 59.452 ms | 68.257 ms | 68.257 ms | 5.266 ms |
| e2e/N=200 | knowledge map --universe | 10 | 76.435 ms | 85.768 ms | 93.100 ms | 93.100 ms | 5.646 ms |
| e2e/N=200 | knowledge rank --universe | 10 | 53.968 ms | 59.829 ms | 73.898 ms | 73.898 ms | 6.363 ms |
| e2e/N=200 | knowledge tags | 10 | 47.228 ms | 54.721 ms | 64.685 ms | 64.685 ms | 4.667 ms |
| e2e/N=200 | knowledge digest --status | 10 | 44.161 ms | 52.417 ms | 54.474 ms | 54.474 ms | 3.057 ms |
| e2e/N=200 | maintenance doctor | 10 | 166.584 ms | 167.193 ms | 181.051 ms | 181.051 ms | 5.877 ms |
| e2e/N=200 | maintenance doctor --audit | 10 | 135.463 ms | 157.785 ms | 172.242 ms | 172.242 ms | 12.340 ms |
| e2e/N=200 | maintenance learn --universe | 10 | 32.427 ms | 51.114 ms | 57.003 ms | 57.003 ms | 9.936 ms |
| e2e/N=200 | maintenance compact --universe | 10 | 97.897 ms | 108.007 ms | 121.869 ms | 121.869 ms | 8.416 ms |
| e2e/N=200 | maintenance prune --universe | 10 | 41.293 ms | 52.755 ms | 60.859 ms | 60.859 ms | 7.184 ms |
| e2e/N=200 | config list | 10 | 28.688 ms | 35.791 ms | 40.938 ms | 40.938 ms | 3.974 ms |
| e2e/N=200 | config get recall.default_limit | 10 | 1.812 ms | 3.109 ms | 3.842 ms | 3.842 ms | 801.23 µs |
| e2e/N=200 | write (nova) | 10 | 35.419 ms | 38.526 ms | 55.317 ms | 55.317 ms | 5.762 ms |
| e2e/N=200 | write (idempotente) | 10 | 34.314 ms | 40.854 ms | 55.455 ms | 55.455 ms | 6.636 ms |
| e2e/N=200 | config set (projeto) | 10 | 2.238 ms | 3.474 ms | 3.812 ms | 3.812 ms | 637.85 µs |
| e2e/N=200 | forget (soft) | 10 | 19.182 ms | 20.670 ms | 25.677 ms | 25.677 ms | 2.162 ms |
| e2e/N=200 | forget --restore | 10 | 21.116 ms | 24.511 ms | 36.604 ms | 36.604 ms | 5.929 ms |
| e2e/N=200 | sync | 10 | 29.985 ms | 39.129 ms | 54.005 ms | 54.005 ms | 7.235 ms |
| e2e/N=1000 | self version (piso de startup) | 10 | 36.197 ms | 47.606 ms | 58.395 ms | 58.395 ms | 8.959 ms |
| e2e/N=1000 | self version --json | 10 | 38.610 ms | 53.769 ms | 66.644 ms | 66.644 ms | 9.183 ms |
| e2e/N=1000 | --help | 10 | 2.641 ms | 3.876 ms | 3.982 ms | 3.982 ms | 516.58 µs |
| e2e/N=1000 | prime | 10 | 37.099 ms | 40.171 ms | 58.208 ms | 58.208 ms | 8.541 ms |
| e2e/N=1000 | prime --long | 10 | 35.584 ms | 41.137 ms | 53.204 ms | 53.204 ms | 6.132 ms |
| e2e/N=1000 | ask (query comum) | 10 | 108.699 ms | 112.836 ms | 144.842 ms | 144.842 ms | 13.136 ms |
| e2e/N=1000 | ask (query rara) | 10 | 112.559 ms | 123.718 ms | 148.748 ms | 148.748 ms | 12.286 ms |
| e2e/N=1000 | ask --json | 10 | 108.555 ms | 113.311 ms | 141.562 ms | 141.562 ms | 10.954 ms |
| e2e/N=1000 | ask --limit 50 | 10 | 113.909 ms | 126.516 ms | 136.109 ms | 136.109 ms | 7.280 ms |
| e2e/N=1000 | ask --brief | 10 | 108.134 ms | 114.416 ms | 132.855 ms | 132.855 ms | 9.743 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 10 | 109.688 ms | 128.927 ms | 133.400 ms | 133.400 ms | 9.086 ms |
| e2e/N=1000 | ask --around <nota> | 10 | 71.153 ms | 86.522 ms | 93.156 ms | 93.156 ms | 7.983 ms |
| e2e/N=1000 | ask --id <nota> | 10 | 48.289 ms | 58.772 ms | 82.564 ms | 82.564 ms | 11.985 ms |
| e2e/N=1000 | rewind | 10 | 358.791 ms | 362.109 ms | 377.661 ms | 377.661 ms | 6.326 ms |
| e2e/N=1000 | rewind --json | 10 | 362.668 ms | 370.758 ms | 396.169 ms | 396.169 ms | 10.634 ms |
| e2e/N=1000 | rewind --files src/core/** | 10 | 157.241 ms | 164.466 ms | 181.974 ms | 181.974 ms | 8.122 ms |
| e2e/N=1000 | task list | 10 | 13.008 ms | 17.796 ms | 22.717 ms | 22.717 ms | 2.928 ms |
| e2e/N=1000 | task list --ready | 10 | 96.853 ms | 103.965 ms | 132.810 ms | 132.810 ms | 10.689 ms |
| e2e/N=1000 | task list --sort impact | 10 | 16.217 ms | 19.717 ms | 25.213 ms | 25.213 ms | 2.792 ms |
| e2e/N=1000 | task list --full-content | 10 | 17.082 ms | 22.015 ms | 26.824 ms | 26.824 ms | 3.170 ms |
| e2e/N=1000 | task show --id <tarefa> | 10 | 72.969 ms | 82.538 ms | 92.041 ms | 92.041 ms | 6.930 ms |
| e2e/N=1000 | task graph | 10 | 101.121 ms | 122.429 ms | 137.336 ms | 137.336 ms | 13.682 ms |
| e2e/N=1000 | knowledge map --universe | 10 | 185.086 ms | 191.193 ms | 209.385 ms | 209.385 ms | 7.954 ms |
| e2e/N=1000 | knowledge rank --universe | 10 | 106.698 ms | 112.715 ms | 119.877 ms | 119.877 ms | 4.015 ms |
| e2e/N=1000 | knowledge tags | 10 | 79.677 ms | 88.863 ms | 108.359 ms | 108.359 ms | 9.168 ms |
| e2e/N=1000 | knowledge digest --status | 10 | 68.150 ms | 76.189 ms | 99.816 ms | 99.816 ms | 10.299 ms |
| e2e/N=1000 | maintenance doctor | 10 | 2.351 s | 2.464 s | 2.584 s | 2.584 s | 80.394 ms |
| e2e/N=1000 | maintenance doctor --audit | 10 | 2.398 s | 2.475 s | 2.547 s | 2.547 s | 52.176 ms |
| e2e/N=1000 | maintenance learn --universe | 10 | 80.679 ms | 88.059 ms | 103.220 ms | 103.220 ms | 7.956 ms |
| e2e/N=1000 | maintenance compact --universe | 10 | 2.449 s | 2.490 s | 2.520 s | 2.520 s | 30.290 ms |
| e2e/N=1000 | maintenance prune --universe | 10 | 172.348 ms | 178.096 ms | 186.231 ms | 186.231 ms | 4.971 ms |
| e2e/N=1000 | config list | 10 | 46.388 ms | 54.444 ms | 86.383 ms | 86.383 ms | 12.005 ms |
| e2e/N=1000 | config get recall.default_limit | 10 | 2.298 ms | 3.733 ms | 4.061 ms | 4.061 ms | 491.84 µs |
| e2e/N=1000 | write (nova) | 10 | 88.039 ms | 100.799 ms | 115.558 ms | 115.558 ms | 9.725 ms |
| e2e/N=1000 | write (idempotente) | 10 | 86.551 ms | 88.309 ms | 118.511 ms | 118.511 ms | 12.172 ms |
| e2e/N=1000 | config set (projeto) | 10 | 1.566 ms | 3.369 ms | 3.739 ms | 3.739 ms | 783.86 µs |
| e2e/N=1000 | forget (soft) | 10 | 51.732 ms | 54.871 ms | 67.482 ms | 67.482 ms | 5.842 ms |
| e2e/N=1000 | forget --restore | 10 | 52.839 ms | 65.307 ms | 78.510 ms | 78.510 ms | 9.058 ms |
| e2e/N=1000 | sync | 10 | 55.624 ms | 69.715 ms | 84.324 ms | 84.324 ms | 9.194 ms |
