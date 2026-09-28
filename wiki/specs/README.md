# specs — documentação técnica

Referência técnica aprofundada do knudge: como cada subsistema funciona, quais invariantes
sustenta e onde o comportamento vive no código.

## Visão de conjunto

| Documento | Assunto |
|---|---|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | camadas (núcleo puro + portas + adaptadores) e mapa de módulos. |
| [`TOON.md`](TOON.md) | gramática do frontmatter e contrato de bytes. |
| [`DIVERGENCES.md`](DIVERGENCES.md) | bordas conhecidas (Unicode, ordem, lock, atomicidade…) + teste que trava cada uma. |
| [`modelo-de-dados.md`](modelo-de-dados.md) | schema, IDs, hash, chaves canônicas, enums, claims e proveniência. |
| [`matematica.md`](matematica.md) | derivação dos modelos quantitativos: BM25, RRF, Beta/Wilson, FSRS, KL/JS, PageRank/PPR, Louvain, MinHash/LSH, PERT/CPM. |
| [`persistencia.md`](persistencia.md) | `notas/`/`eventos/`/`.idx/`, escrita atômica, lock, rebuild, crash. |
| [`configuracao.md`](configuracao.md) | config em dois níveis e codec TOML próprio. |
| [`git-e-worktree.md`](git-e-worktree.md) | worktree principal, `info/exclude`, `AGENTS.md`, skill, `sync`, `onboard`. |
| [`busca.md`](busca.md) | pipeline de retrieval: filtros → BM25 → âncoras → RRF (+ PPR), views, temporal. |
| [`escrita-e-dedup.md`](escrita-e-dedup.md) | protocolo de escrita idempotente, dedup em duas fases, update/supersede. |
| [`grafo.md`](grafo.md) | arestas, integridade, ciclos, PageRank/PPR, comunidades, ontologia, TMS. |
| [`tarefas.md`](tarefas.md) | hierarquia `epic ⊃ {issue ⊃ task}`, papéis/modos, impacto, plano, fluxo. |
| [`ciclo-de-vida.md`](ciclo-de-vida.md) | shelf-life, retenção (FSRS-like), confiança Beta, decay, drift, clusters. |
| [`saude.md`](saude.md) | validators, `doctor [--fix]`, `audit`, evidência, âncoras, leitura tolerante. |
| [`embeddings.md`](embeddings.md) | provedor plugável, cache, fila lazy, `drain`, sugestões semânticas. |
| [`handoff.md`](handoff.md) | `rewind`: manifest/escopo/working set, orçamento, `context_id`. |
| [`manutencao.md`](manutencao.md) | `diff`, `learn`, `compact` (propostas) e portão de evidência. |
| [`cli.md`](cli.md) | superfície de verbos, contrato de saída, envelope, exit codes, EPIPE. |
| [`erros.md`](erros.md) | `Error`/`ErrorKind`, mapa código→exit, poison, degradação graciosa. |
| [`mcp.md`](mcp.md) | servidor MCP: JSON-RPC 2.0 sobre stdio, gatilhos e tools. |
| [`determinismo.md`](determinismo.md) | garantias de determinismo, fakes, concorrência e crash-injection. |
| [`adrs/`](adrs/README.md) | uma decisão (`Dxx`) por arquivo. |

## Como ler

Cada documento técnico segue a mesma estrutura: **papel**, **invariantes**, **fluxo**,
**contratos** (bytes/ordem/config), **onde vive no código** e **testes**. As decisões `Dxx` são
citadas inline e detalhadas em [`adrs/`](adrs/README.md).
