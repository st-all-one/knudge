# Qualidade de retrieval — baseline (E16/T01)

> Corpus PT-BR sintético e rotulado (**192 notas**, **24 consultas**) gerado
> deterministicamente. Métricas sobre o `recall` com os defaults de produção
> (RRF `k=60`, pesos lexical/âncora/semântico `1/1/30`, sem canal vetorial).
> `criterion` é não-objetivo (E13-T09): a bancada **observa**, não é gate.

| família | n | R@1 | R@3 | R@5 | R@10 | MRR | nDCG@1 | nDCG@3 | nDCG@5 | nDCG@10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| com-acento | 12 | 12.5% | 37.5% | 62.5% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
| sem-acento | 12 | 1.0% | 3.1% | 5.2% | 8.3% | 8.3% | 8.3% | 8.3% | 8.3% | 8.3% |
| **geral** | 24 | 6.8% | 20.3% | 33.9% | 54.2% | 54.2% | 54.2% | 54.2% | 54.2% | 54.2% |
