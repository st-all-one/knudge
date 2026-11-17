| grupo | operação | n | min | mediana | p95 | máx | desvio |
|---|---|---:|---:|---:|---:|---:|---:|
| micro/fixo | schema::body::normalize (curto ~200B) | 25 | 173 ns | 181 ns | 270 ns | 276 ns | 36 ns |
| micro/fixo | schema::body::normalize (longo ~1.6KB) | 25 | 1.30 µs | 1.35 µs | 1.80 µs | 1.90 µs | 177 ns |
| micro/fixo | schema::body::body_hash | 25 | 380 ns | 397 ns | 514 ns | 546 ns | 46 ns |
| micro/fixo | schema::id::note_id | 25 | 154 ns | 168 ns | 258 ns | 272 ns | 30 ns |
| micro/fixo | schema::hash::short_hash | 25 | 420 ns | 427 ns | 519 ns | 522 ns | 26 ns |
| micro/fixo | schema::hash::base36_8 | 25 | 13 ns | 14 ns | 15 ns | 15 ns | 1 ns |
| micro/fixo | toon::parse (frontmatter) | 25 | 1.11 µs | 1.15 µs | 1.44 µs | 1.45 µs | 111 ns |
| micro/fixo | toon::emit (frontmatter) | 25 | 535 ns | 563 ns | 878 ns | 912 ns | 114 ns |
| micro/fixo | Note::parse (render completo) | 25 | 2.72 µs | 2.85 µs | 3.40 µs | 3.51 µs | 246 ns |
| micro/fixo | Note::render | 25 | 1.62 µs | 1.73 µs | 1.81 µs | 2.24 µs | 125 ns |
| micro/fixo | jsonl::decode | 25 | 640 ns | 660 ns | 699 ns | 760 ns | 24 ns |
| micro/fixo | jsonl::encode | 25 | 377 ns | 384 ns | 390 ns | 495 ns | 23 ns |
| micro/fixo | retrieval::token::tokenize (corpo) | 25 | 221 ns | 229 ns | 244 ns | 256 ns | 8 ns |
| micro/fixo | retrieval::token::tokenize (acentuado) | 25 | 1.85 µs | 1.89 µs | 1.92 µs | 1.96 µs | 24 ns |
| micro/fixo | retrieval::token::content_terms | 25 | 1.52 µs | 1.54 µs | 1.58 µs | 1.61 µs | 23 ns |
| micro/fixo | retrieval::rrf::fuse (3x200) | 25 | 83.01 µs | 85.45 µs | 98.14 µs | 99.96 µs | 4.57 µs |
| micro/fixo | lifecycle::confidence_score | 25 | 14 ns | 15 ns | 16 ns | 17 ns | 1 ns |
| micro/fixo | lifecycle::confidence_score (sem evidência) | 25 | 8 ns | 8 ns | 8 ns | 8 ns | 0 ns |
| micro/fixo | handoff::budget::estimate_tokens | 25 | 34 ns | 37 ns | 37 ns | 42 ns | 2 ns |
| micro/fixo | handoff::budget::apply (1000 linhas) | 25 | 2.85 µs | 2.92 µs | 3.05 µs | 3.09 µs | 64 ns |
| micro/fixo | config::Config::parse | 25 | 2.32 µs | 2.43 µs | 2.76 µs | 2.91 µs | 150 ns |
| micro/fixo | retrieval::anchor::glob_match | 25 | 128 ns | 131 ns | 139 ns | 139 ns | 3 ns |
| micro/fixo | retrieval::anchor::GlobPattern::matches | 25 | 110 ns | 115 ns | 125 ns | 127 ns | 4 ns |
| micro/fixo | embeddings::lightweight::embed (384d) | 25 | 26.50 µs | 26.67 µs | 27.29 µs | 27.69 µs | 256 ns |
| micro/fixo | embeddings::vector::cosine (384d) | 25 | 719 ns | 725 ns | 778 ns | 803 ns | 24 ns |
| micro/N=200 | retrieval::Index::build | 15 | 1.149 ms | 1.169 ms | 1.210 ms | 1.256 ms | 27.80 µs |
| micro/N=200 | retrieval::Postings::build | 15 | 422.05 µs | 440.91 µs | 470.52 µs | 478.34 µs | 16.29 µs |
| micro/N=200 | retrieval::Index::score (BM25) | 15 | 144.22 µs | 146.11 µs | 156.03 µs | 163.71 µs | 5.45 µs |
| micro/N=200 | write::propose_merges (denso) | 15 | 40.502 ms | 41.632 ms | 43.390 ms | 44.854 ms | 1.154 ms |
| micro/N=200 | write::propose_merges (esparso) | 15 | 204.08 µs | 209.59 µs | 254.78 µs | 275.04 µs | 21.08 µs |
| micro/N=200 | retrieval::recall (limit 5) | 15 | 262.11 µs | 270.98 µs | 298.29 µs | 312.82 µs | 14.27 µs |
| micro/N=200 | retrieval::recall (sem limite) | 15 | 532.89 µs | 548.88 µs | 611.04 µs | 693.80 µs | 42.42 µs |
| micro/N=200 | retrieval::rank (confiança) | 15 | 17.04 µs | 18.09 µs | 18.72 µs | 19.07 µs | 678 ns |
| micro/N=200 | lifecycle::structural_clusters | 15 | 115.38 µs | 128.02 µs | 148.13 µs | 168.53 µs | 14.42 µs |
| micro/N=200 | lifecycle::communities | 15 | 643.03 µs | 676.34 µs | 725.51 µs | 742.13 µs | 29.59 µs |
| micro/N=200 | Graph::from_notes | 15 | 322.53 µs | 335.31 µs | 362.06 µs | 369.74 µs | 15.59 µs |
| micro/N=200 | graph::pagerank | 15 | 73.33 µs | 87.23 µs | 104.34 µs | 120.13 µs | 14.21 µs |
| micro/N=200 | retrieval::recall (ppr) | 15 | 381.12 µs | 398.30 µs | 412.20 µs | 448.52 µs | 16.31 µs |
| micro/N=200 | Graph::integrity | 15 | 14.32 µs | 15.22 µs | 19.98 µs | 25.63 µs | 2.97 µs |
| micro/N=200 | Graph::supersession_cycles | 15 | 87.23 µs | 98.83 µs | 106.30 µs | 122.43 µs | 8.89 µs |
| micro/N=200 | Graph::dependency_cycles | 15 | 83.04 µs | 87.93 µs | 99.73 µs | 143.87 µs | 15.16 µs |
| micro/N=200 | retrieval::compute_views | 15 | 100.92 µs | 103.85 µs | 121.80 µs | 138.01 µs | 10.22 µs |
| micro/N=200 | task::impact (1 id) | 15 | 12.15 µs | 12.99 µs | 13.90 µs | 15.16 µs | 749 ns |
| micro/N=200 | task::impacts (todos) | 15 | 12.29 µs | 13.41 µs | 14.18 µs | 21.37 µs | 2.19 µs |
| micro/N=200 | handoff::rank (manifest) | 15 | 33.10 µs | 36.18 µs | 51.68 µs | 55.87 µs | 7.10 µs |
| micro/N=200 | Index::serialize + parse | 15 | 3.438 ms | 3.561 ms | 3.652 ms | 3.656 ms | 60.96 µs |
| micro/N=200 | Index::serialize | 15 | 1.292 ms | 1.344 ms | 1.650 ms | 1.798 ms | 142.50 µs |
| micro/N=200 | Index::parse | 15 | 1.857 ms | 1.906 ms | 1.960 ms | 1.995 ms | 41.59 µs |
| micro/N=200 | write::Draft::to_note | 15 | 1.59 µs | 1.70 µs | 1.84 µs | 1.97 µs | 94 ns |
| micro/N=1000 | retrieval::Index::build | 15 | 7.582 ms | 8.244 ms | 8.604 ms | 8.680 ms | 344.40 µs |
| micro/N=1000 | retrieval::Postings::build | 15 | 3.177 ms | 3.427 ms | 3.552 ms | 3.623 ms | 124.49 µs |
| micro/N=1000 | retrieval::Index::score (BM25) | 15 | 937.13 µs | 1.019 ms | 1.117 ms | 1.142 ms | 65.69 µs |
| micro/N=1000 | write::propose_merges (denso) | 15 | 28.061 ms | 29.058 ms | 30.096 ms | 31.327 ms | 807.74 µs |
| micro/N=1000 | write::propose_merges (esparso) | 15 | 1.265 ms | 1.356 ms | 1.762 ms | 1.809 ms | 161.80 µs |
| micro/N=1000 | retrieval::recall (limit 5) | 15 | 1.348 ms | 1.422 ms | 1.565 ms | 1.592 ms | 73.44 µs |
| micro/N=1000 | retrieval::recall (sem limite) | 15 | 2.939 ms | 3.099 ms | 3.422 ms | 3.664 ms | 211.23 µs |
| micro/N=1000 | retrieval::rank (confiança) | 15 | 91.07 µs | 93.45 µs | 106.30 µs | 109.37 µs | 6.39 µs |
| micro/N=1000 | lifecycle::structural_clusters | 15 | 688.42 µs | 709.73 µs | 739.34 µs | 809.95 µs | 30.19 µs |
| micro/N=1000 | lifecycle::communities | 15 | 3.597 ms | 3.800 ms | 4.041 ms | 4.069 ms | 151.07 µs |
| micro/N=1000 | Graph::from_notes | 15 | 1.642 ms | 1.753 ms | 2.039 ms | 2.166 ms | 152.19 µs |
| micro/N=1000 | graph::pagerank | 15 | 429.04 µs | 440.49 µs | 463.96 µs | 472.69 µs | 12.36 µs |
| micro/N=1000 | retrieval::recall (ppr) | 15 | 2.122 ms | 2.328 ms | 2.536 ms | 2.541 ms | 149.90 µs |
| micro/N=1000 | Graph::integrity | 15 | 106.93 µs | 121.17 µs | 124.95 µs | 133.40 µs | 7.28 µs |
| micro/N=1000 | Graph::supersession_cycles | 15 | 469.05 µs | 488.12 µs | 502.79 µs | 547.41 µs | 18.13 µs |
| micro/N=1000 | Graph::dependency_cycles | 15 | 484.28 µs | 504.18 µs | 558.80 µs | 624.59 µs | 35.30 µs |
| micro/N=1000 | retrieval::compute_views | 15 | 626.27 µs | 660.21 µs | 993.28 µs | 1.037 ms | 130.95 µs |
| micro/N=1000 | task::impact (1 id) | 15 | 102.88 µs | 108.81 µs | 115.73 µs | 117.33 µs | 5.25 µs |
| micro/N=1000 | task::impacts (todos) | 15 | 101.27 µs | 111.40 µs | 175.58 µs | 199.54 µs | 28.51 µs |
| micro/N=1000 | handoff::rank (manifest) | 15 | 189.34 µs | 199.96 µs | 216.23 µs | 220.35 µs | 9.86 µs |
| micro/N=1000 | Index::serialize + parse | 15 | 18.056 ms | 19.172 ms | 20.120 ms | 20.163 ms | 695.28 µs |
| micro/N=1000 | Index::serialize | 15 | 6.904 ms | 7.509 ms | 8.562 ms | 9.608 ms | 729.30 µs |
| micro/N=1000 | Index::parse | 15 | 9.649 ms | 10.581 ms | 11.596 ms | 11.725 ms | 612.06 µs |
| micro/N=1000 | write::Draft::to_note | 15 | 1.56 µs | 1.69 µs | 1.77 µs | 1.79 µs | 74 ns |
| micro/ai | corpus::parse_jsonl (shard N=1000) | 3 | 4.687 ms | 4.932 ms | 5.301 ms | 5.301 ms | 309.20 µs |
| micro/ai-legacy | upsert linear (referência N=1000) | 3 | 22.473 ms | 22.647 ms | 24.022 ms | 24.022 ms | 848.48 µs |
| micro/ai | corpus::parse_jsonl (shard N=10000) | 3 | 55.311 ms | 55.312 ms | 57.386 ms | 57.386 ms | 1.198 ms |
| micro/ai-legacy | upsert linear (referência N=10000) | 3 | 2.537 s | 2.668 s | 3.002 s | 3.002 s | 240.098 ms |
| micro/ai | corpus::parse_jsonl (shard N=100000) | 3 | 635.347 ms | 638.955 ms | 641.279 ms | 641.279 ms | 2.989 ms |
| micro/fixo | lifecycle::freshness_with (N=500) | 15 | 105.88 µs | 106.91 µs | 108.04 µs | 109.60 µs | 986 ns |
| micro/fixo | lifecycle::anchor_coverage (600 globs / 2000 paths) | 15 | 236.915 ms | 245.240 ms | 270.316 ms | 280.660 ms | 13.213 ms |
| micro/fixo | ProposalStore::persist (N=1000) | 15 | 344.57 µs | 354.96 µs | 362.76 µs | 363.29 µs | 6.03 µs |
| micro/fixo | ProposalStore::count (N=1000) | 15 | 423.92 µs | 428.48 µs | 443.86 µs | 459.18 µs | 9.50 µs |
| micro/fixo | ProposalStore::load (N=1000) | 15 | 446.33 µs | 448.21 µs | 450.43 µs | 451.12 µs | 1.21 µs |
| e2e/N=200 | self version (piso de startup) | 8 | 1.170 ms | 1.232 ms | 1.268 ms | 1.268 ms | 31.94 µs |
| e2e/N=200 | self version --json | 8 | 1.179 ms | 1.305 ms | 1.530 ms | 1.530 ms | 118.01 µs |
| e2e/N=200 | --help | 8 | 1.170 ms | 1.281 ms | 1.542 ms | 1.542 ms | 130.47 µs |
| e2e/N=200 | core: Corpus::load_notes | 8 | 3.683 ms | 3.939 ms | 4.233 ms | 4.233 ms | 158.54 µs |
| e2e/N=200 | core: Corpus::load (notes+index+graph) | 8 | 4.879 ms | 5.063 ms | 5.868 ms | 5.868 ms | 317.34 µs |
| e2e/N=200 | prime | 8 | 1.216 ms | 1.591 ms | 1.818 ms | 1.818 ms | 213.11 µs |
| e2e/N=200 | prime --long | 8 | 1.243 ms | 1.518 ms | 1.663 ms | 1.663 ms | 150.78 µs |
| e2e/N=200 | ask (query comum) | 8 | 114.782 ms | 395.894 ms | 572.519 ms | 572.519 ms | 197.692 ms |
| e2e/N=200 | ask (query rara) | 8 | 125.925 ms | 147.985 ms | 157.053 ms | 157.053 ms | 9.056 ms |
| e2e/N=200 | ask --json | 8 | 138.269 ms | 150.414 ms | 158.766 ms | 158.766 ms | 7.854 ms |
| e2e/N=200 | ask --limit 50 | 8 | 142.701 ms | 147.912 ms | 151.604 ms | 151.604 ms | 3.316 ms |
| e2e/N=200 | ask --brief | 8 | 129.024 ms | 143.650 ms | 148.838 ms | 148.838 ms | 6.061 ms |
| e2e/N=200 | ask --type fact --anchor src/** | 8 | 125.869 ms | 146.577 ms | 149.188 ms | 149.188 ms | 7.947 ms |
| e2e/N=200 | ask --around <nota> | 8 | 46.146 ms | 48.805 ms | 51.145 ms | 51.145 ms | 1.560 ms |
| e2e/N=200 | ask --id <nota> | 8 | 41.956 ms | 43.056 ms | 43.690 ms | 43.690 ms | 490.97 µs |
| e2e/N=200 | rewind | 8 | 59.212 ms | 61.815 ms | 62.784 ms | 62.784 ms | 1.168 ms |
| e2e/N=200 | rewind --json | 8 | 60.412 ms | 61.063 ms | 62.437 ms | 62.437 ms | 571.96 µs |
| e2e/N=200 | rewind --files src/core/** | 8 | 60.530 ms | 62.602 ms | 66.395 ms | 66.395 ms | 2.020 ms |
| e2e/N=200 | task list --universe | 8 | 46.369 ms | 47.112 ms | 50.579 ms | 50.579 ms | 1.438 ms |
| e2e/N=200 | task list --ready | 8 | 47.102 ms | 48.691 ms | 50.392 ms | 50.392 ms | 1.070 ms |
| e2e/N=200 | task list --sort impact | 8 | 46.140 ms | 47.935 ms | 49.804 ms | 49.804 ms | 1.199 ms |
| e2e/N=200 | task list --full-content | 8 | 49.902 ms | 51.120 ms | 54.004 ms | 54.004 ms | 1.550 ms |
| e2e/N=200 | task show --id <tarefa> | 8 | 45.638 ms | 47.981 ms | 49.380 ms | 49.380 ms | 1.202 ms |
| e2e/N=200 | task graph | 8 | 45.533 ms | 48.174 ms | 49.505 ms | 49.505 ms | 1.455 ms |
| e2e/N=200 | map --universe | 8 | 63.177 ms | 64.378 ms | 70.814 ms | 70.814 ms | 2.520 ms |
| e2e/N=200 | ask --rank --universe | 8 | 54.702 ms | 56.311 ms | 57.639 ms | 57.639 ms | 894.15 µs |
| e2e/N=200 | ask --tags | 8 | 48.822 ms | 50.136 ms | 51.787 ms | 51.787 ms | 949.07 µs |
| e2e/N=200 | drain --status | 8 | 24.215 ms | 25.417 ms | 26.753 ms | 26.753 ms | 751.32 µs |
| e2e/N=200 | doctor | 8 | 139.808 ms | 142.601 ms | 145.274 ms | 145.274 ms | 1.907 ms |
| e2e/N=200 | doctor --explain | 8 | 140.327 ms | 147.852 ms | 158.399 ms | 158.399 ms | 6.280 ms |
| e2e/N=200 | maintenance learn --universe | 8 | 18.991 ms | 21.243 ms | 26.298 ms | 26.298 ms | 2.261 ms |
| e2e/N=200 | maintenance compact --universe | 8 | 70.289 ms | 74.211 ms | 110.296 ms | 110.296 ms | 13.257 ms |
| e2e/N=200 | maintenance prune --universe | 8 | 16.848 ms | 19.010 ms | 27.935 ms | 27.935 ms | 3.605 ms |
| e2e/N=200 | config list | 8 | 48.077 ms | 58.674 ms | 67.342 ms | 67.342 ms | 6.818 ms |
| e2e/N=200 | config get recall.default_limit | 8 | 52.641 ms | 57.998 ms | 65.653 ms | 65.653 ms | 4.035 ms |
| e2e/N=200 | write (nova) | 8 | 63.902 ms | 66.915 ms | 71.521 ms | 71.521 ms | 2.404 ms |
| e2e/N=200 | write (idempotente) | 8 | 58.901 ms | 64.163 ms | 162.324 ms | 162.324 ms | 34.963 ms |
| e2e/N=200 | write --dry-run (120 âncoras) | 8 | 73.136 ms | 80.675 ms | 83.462 ms | 83.462 ms | 4.427 ms |
| e2e/N=200 | config set (projeto) | 8 | 48.219 ms | 63.547 ms | 69.058 ms | 69.058 ms | 7.304 ms |
| e2e/N=200 | forget (soft) | 8 | 48.188 ms | 51.451 ms | 56.201 ms | 56.201 ms | 2.642 ms |
| e2e/N=200 | forget --restore | 8 | 48.917 ms | 51.591 ms | 53.458 ms | 53.458 ms | 1.657 ms |
| e2e/N=200 | sync | 8 | 44.912 ms | 47.347 ms | 58.532 ms | 58.532 ms | 5.392 ms |
| e2e/N=1000 | self version (piso de startup) | 8 | 1.275 ms | 1.485 ms | 1.752 ms | 1.752 ms | 165.87 µs |
| e2e/N=1000 | self version --json | 8 | 1.334 ms | 1.645 ms | 1.856 ms | 1.856 ms | 171.36 µs |
| e2e/N=1000 | --help | 8 | 1.101 ms | 1.438 ms | 1.872 ms | 1.872 ms | 268.40 µs |
| e2e/N=1000 | core: Corpus::load_notes | 8 | 5.012 ms | 5.665 ms | 6.169 ms | 6.169 ms | 464.04 µs |
| e2e/N=1000 | core: Corpus::load (notes+index+graph) | 8 | 15.048 ms | 16.458 ms | 18.084 ms | 18.084 ms | 1.191 ms |
| e2e/N=1000 | prime | 8 | 1.175 ms | 1.554 ms | 1.706 ms | 1.706 ms | 213.97 µs |
| e2e/N=1000 | prime --long | 8 | 1.074 ms | 1.341 ms | 1.643 ms | 1.643 ms | 170.10 µs |
| e2e/N=1000 | ask (query comum) | 8 | 824.017 ms | 911.351 ms | 921.838 ms | 921.838 ms | 38.334 ms |
| e2e/N=1000 | ask (query rara) | 8 | 906.824 ms | 978.942 ms | 1.016 s | 1.016 s | 37.265 ms |
| e2e/N=1000 | ask --json | 8 | 317.893 ms | 975.000 ms | 1.042 s | 1.042 s | 301.211 ms |
| e2e/N=1000 | ask --limit 50 | 8 | 293.463 ms | 319.000 ms | 326.565 ms | 326.565 ms | 10.301 ms |
| e2e/N=1000 | ask --brief | 8 | 308.490 ms | 319.643 ms | 325.144 ms | 325.144 ms | 5.578 ms |
| e2e/N=1000 | ask --type fact --anchor src/** | 8 | 313.318 ms | 319.747 ms | 326.491 ms | 326.491 ms | 4.397 ms |
| e2e/N=1000 | ask --around <nota> | 8 | 183.226 ms | 189.766 ms | 193.708 ms | 193.708 ms | 3.440 ms |
| e2e/N=1000 | ask --id <nota> | 8 | 162.458 ms | 164.993 ms | 171.868 ms | 171.868 ms | 3.337 ms |
| e2e/N=1000 | rewind | 8 | 231.102 ms | 232.563 ms | 237.174 ms | 237.174 ms | 2.157 ms |
| e2e/N=1000 | rewind --json | 8 | 231.966 ms | 236.037 ms | 240.396 ms | 240.396 ms | 3.002 ms |
| e2e/N=1000 | rewind --files src/core/** | 8 | 228.648 ms | 235.761 ms | 239.693 ms | 239.693 ms | 4.067 ms |
| e2e/N=1000 | task list --universe | 8 | 168.320 ms | 172.888 ms | 182.238 ms | 182.238 ms | 4.583 ms |
| e2e/N=1000 | task list --ready | 8 | 166.018 ms | 180.832 ms | 188.834 ms | 188.834 ms | 8.862 ms |
| e2e/N=1000 | task list --sort impact | 8 | 168.988 ms | 174.310 ms | 190.042 ms | 190.042 ms | 6.522 ms |
| e2e/N=1000 | task list --full-content | 8 | 198.596 ms | 201.404 ms | 205.637 ms | 205.637 ms | 2.281 ms |
| e2e/N=1000 | task show --id <tarefa> | 8 | 182.801 ms | 183.280 ms | 185.703 ms | 185.703 ms | 1.032 ms |
| e2e/N=1000 | task graph | 8 | 172.121 ms | 175.751 ms | 184.912 ms | 184.912 ms | 4.816 ms |
| e2e/N=1000 | map --universe | 8 | 269.937 ms | 274.703 ms | 297.364 ms | 297.364 ms | 12.191 ms |
| e2e/N=1000 | ask --rank --universe | 8 | 209.740 ms | 217.204 ms | 242.935 ms | 242.935 ms | 11.707 ms |
| e2e/N=1000 | ask --tags | 8 | 188.004 ms | 190.781 ms | 195.847 ms | 195.847 ms | 2.680 ms |
| e2e/N=1000 | drain --status | 8 | 92.053 ms | 98.811 ms | 101.971 ms | 101.971 ms | 3.593 ms |
| e2e/N=1000 | doctor | 8 | 238.220 ms | 256.261 ms | 268.580 ms | 268.580 ms | 9.372 ms |
| e2e/N=1000 | doctor --explain | 8 | 236.316 ms | 245.408 ms | 256.284 ms | 256.284 ms | 8.427 ms |
| e2e/N=1000 | maintenance learn --universe | 8 | 33.674 ms | 34.398 ms | 36.552 ms | 36.552 ms | 1.084 ms |
| e2e/N=1000 | maintenance compact --universe | 8 | 66.338 ms | 68.270 ms | 70.500 ms | 70.500 ms | 1.191 ms |
| e2e/N=1000 | maintenance prune --universe | 8 | 35.328 ms | 37.093 ms | 38.523 ms | 38.523 ms | 972.92 µs |
| e2e/N=1000 | config list | 8 | 164.505 ms | 170.649 ms | 179.347 ms | 179.347 ms | 5.857 ms |
| e2e/N=1000 | config get recall.default_limit | 8 | 162.910 ms | 164.639 ms | 172.867 ms | 172.867 ms | 3.220 ms |
| e2e/N=1000 | write (nova) | 8 | 193.476 ms | 196.559 ms | 209.735 ms | 209.735 ms | 5.482 ms |
| e2e/N=1000 | write (idempotente) | 8 | 185.041 ms | 189.613 ms | 196.687 ms | 196.687 ms | 3.960 ms |
| e2e/N=1000 | write --dry-run (120 âncoras) | 8 | 190.830 ms | 196.062 ms | 199.746 ms | 199.746 ms | 2.718 ms |
| e2e/N=1000 | config set (projeto) | 8 | 157.708 ms | 160.520 ms | 161.765 ms | 161.765 ms | 1.151 ms |
| e2e/N=1000 | forget (soft) | 8 | 187.708 ms | 192.134 ms | 207.650 ms | 207.650 ms | 7.710 ms |
| e2e/N=1000 | forget --restore | 8 | 187.780 ms | 189.710 ms | 193.605 ms | 193.605 ms | 2.299 ms |
| e2e/N=1000 | sync | 8 | 164.519 ms | 167.683 ms | 172.481 ms | 172.481 ms | 2.799 ms |
