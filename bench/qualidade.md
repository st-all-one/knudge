# Qualidade de retrieval — baseline (E16/T01)

> Corpus PT-BR sintético e rotulado (**192 notas**, **48 consultas**) gerado
> deterministicamente. Métricas sobre o `recall` com os defaults de produção
> (RRF `k=60`, pesos lexical/âncora/semântico
> `1/2/30`). As famílias
> `working-set` e `sinonimo` exercitam a fusão com o canal de âncoras (working set) e um
> canal vetorial sintético.
> `criterion` é não-objetivo (E13-T09): a bancada **observa**, não é gate.

| família | n | R@1 | R@3 | R@5 | R@10 | MRR | nDCG@1 | nDCG@3 | nDCG@5 | nDCG@10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| com-acento | 12 | 12.5% | 37.5% | 62.5% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
| sem-acento | 12 | 12.5% | 37.5% | 62.5% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
| sinonimo | 12 | 12.5% | 37.5% | 62.5% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
| working-set | 12 | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
| **geral** | 48 | 34.4% | 53.1% | 71.9% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% |
