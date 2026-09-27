# Implementação do knudge — índice dos épicos

> Plano de implementação do binário **`kd`** e do diretório **`.knudge/`**, derivado dos
> documentos da raiz de `plan/`: `00_panorama.md` (visão), `01_gaps-plan-rs.md` (bordas),
> `02_decisoes.md` (pontos), `03_decisoes-fechadas.md` (D01–D101 ✅), `04_embeddings.md`
> (vetores) e `05_refs-agnostic-rag.md` (extrações do arags).
>
> Cada arquivo desta pasta é um **épico**; cada épico tem **tarefas** com aceite verificável.
> A ordem dos arquivos é a ordem de dependência.
>
> **Plano-mestre da evolução 0.5.0 (E16–E19):** [`24_plano_mestre_0.5.0.md`](24_plano_mestre_0.5.0.md)
> — referências (autoridade única), ordem integrada e gates (teste + bancada).

---

## Meta global: simplicidade

1. **Arquivos são a verdade**; o índice é derivado e reconstruível.
2. **Um binário `kd`**; sem servidor, sem banco, sem processo de fundo obrigatório.
3. **Núcleo puro** por escopo temático; adaptadores finos.
4. **LLM é opcional em tudo** — `write`/`recall`/`prime`/`audit` funcionam sem modelo.
5. Nada entra no caminho crítico que não pague seu custo (ver `05`, seção 3).

---

## Convenções

- **Tarefa:** `E<épico>-T<nn>` (ex.: `E02-T03`).
- **Status:** ☐ pendente · ◐ em andamento · ☑ pronto. (Marcar no arquivo do épico.)
- **Rastreabilidade:** toda tarefa cita as decisões que implementa (`Dxx`).
- **Aceite:** critério verificável (teste, golden, propriedade, script) — não prosa.
- **Bloqueio:** nada de `D52+` antes de `D01–D18` fechadas (ver matriz em `02_decisoes.md`).
- **Disciplina Rust (D92):** arquivos de produção ≤300 linhas, proibido `unwrap/expect/panic`
  em `src/`, `proptest` nos puros, `cargo fmt --check` + `clippy -D warnings` sempre verdes.

---

## Políticas de engenharia (revisão técnica)

A [`14_revisao_tecnica.md`](14_revisao_tecnica.md) cruza o plano com `rust_skill/` (Rust 1.97,
Edition 2024) e define **R01–R43** — políticas de engenharia, distintas das decisões de produto
`Dxx`. Resumo por eixo:

- **Memória:** `forbid(unsafe_code)` nos crates puros; `unsafe` só no adaptador de embedding
  (`// SAFETY:` + Miri/geiger); `Rc`/`RefCell` proibidos no core; `try_reserve`/caps; RAII.
- **Recursos:** canal bounded + backpressure; timeouts/retry; `eventos` segmentado; tetos de
  cache/índice; streaming; **runtime mínimo** (worker bloqueante, sem `tokio full`).
- **Logs:** stdout=dados / stderr=logs; `tracing` estruturado; **redação** de segredos e corpos;
  correlação por `context_id`; retenção limitada.
- **Erros:** `thiserror` no core, `anyhow` na CLI; `#[non_exhaustive]`; código estável +
  `retryable` + `warnings[]`; poison via `into_inner`; contexto obrigatório; exit 101 = panic.
- **Clippy (R44):** `clippy.toml` + `[workspace.lints]` com rigor máximo — análise das 95
  opções em [`15_clippy_config.md`](15_clippy_config.md) e config pronta em [`clippy.toml`](clippy.toml).

Cada achado traz **estado atual, recomendação, onde aplicar e aceite** em `14_revisao_tecnica.md`.

---

## Fases e épicos

| Fase | Épico | Arquivo | Depende de |
|---|---|---|---|
| **0 — Fundação** | E01 Workspace, núcleo puro e ports | [`01_fundacao_workspace.md`](01_fundacao_workspace.md) | — |
| | E02 Contrato de bytes (TOON, schema, IDs) | [`02_contrato_bytes_toon.md`](02_contrato_bytes_toon.md) | E01 |
| | E03 Store de notas, lock e eventos | [`03_store_notas_eventos.md`](03_store_notas_eventos.md) | E02 |
| | E04 Config, Git e worktree | [`04_config_git_worktree.md`](04_config_git_worktree.md) | E01, E03 |
| | E05 Grafo e arestas | [`05_grafo_arestas.md`](05_grafo_arestas.md) | E02, E03 |
| **1 — MVP** | E06 Retrieval (BM25, âncoras, RRF) | [`06_retrieval_rrf.md`](06_retrieval_rrf.md) | E02, E05 |
| | E07 Escrita e protocolo | [`07_escrita_protocolo.md`](07_escrita_protocolo.md) | E03, E05, E06 |
| | E08 Prime, handoff, diff e learn | [`08_prime_handoff.md`](08_prime_handoff.md) | E06, E07 |
| **2 — Saúde e evolução** | E09 Validação, saúde e leitura tolerante | [`09_validacao_saude.md`](09_validacao_saude.md) | E06, E07 |
| | E10 Ciclo de vida, decay e clusters | [`10_lifecycle_clusters.md`](10_lifecycle_clusters.md) | E09 |
| **3 — Embeddings** | E11 Embeddings (provedor, fila, eval) | [`11_embeddings.md`](11_embeddings.md) | E06, E07 |
| **4 — Interfaces** | E12 CLI, MCP, hooks e distribuição | [`12_cli_mcp_distribuicao.md`](12_cli_mcp_distribuicao.md) | E08, E09 |
| **Transversal** | E13 Testes e qualidade | [`13_testes_qualidade.md`](13_testes_qualidade.md) | todos |
| **5 — MCP** | E14 Transporte JSON-RPC e tools | [`18_mcp_transporte.md`](18_mcp_transporte.md) | E12, E13 |
| **6 — Evolução** | E15 Performance + reforma da CLI | [`19_performance_reforma_cli.md`](19_performance_reforma_cli.md) | E01–E14 |
| **7 — Qualidade** | E16 Qualidade da busca + depreciação | [`20_qualidade_busca_depreciacao.md`](20_qualidade_busca_depreciacao.md) | E01–E15 |
| **8 — Robustez** | E17 Worker de embeddings + `--install` | [`21_worker_embeddings_install.md`](21_worker_embeddings_install.md) | E11, E15 |
| **9 — Scripts** | E18 Comandos scriptados (superfície mínima) | [`22_comandos_scriptados.md`](22_comandos_scriptados.md) | E11, E15 |
| **10 — Conhecimento** | E19 Modelo de conhecimento rico | [`23_modelo_conhecimento_rico.md`](23_modelo_conhecimento_rico.md) | E06, E07, E09, E10, E11, E16 |

**MVP = Fase 0 + Fase 1** (+ leitura tolerante mínima de E09). O resto é incremental.

**Status:** E01–E14 ✅ (`make check` verde; 458 testes). Projeto completo pelo plano; evolução
segue as decisões `Dxx` e as políticas `Rnn`. **E15 ✅** (performance + reforma da CLI) concluído —
plano em [`../proposals/`](../proposals/) e épico em
[`19_performance_reforma_cli.md`](19_performance_reforma_cli.md). **E16 ✅** (qualidade da busca +
depreciação de conhecimento; **T01/T02/T03/T04/T06/T07/T09/T10 ✅**) — plano em
[`../proposals/qualidade_busca_depreciacao.md`](../proposals/qualidade_busca_depreciacao.md) e
épico em [`20_qualidade_busca_depreciacao.md`](20_qualidade_busca_depreciacao.md). **E17 ◐**
(robustez do worker de embeddings + `--install`; **T01/T02/T03/T04/T05/T06/T07 ✅**) — plano em
[`../proposals/worker_embeddings_install.md`](../proposals/worker_embeddings_install.md) e épico
em [`21_worker_embeddings_install.md`](21_worker_embeddings_install.md). **E18 ◐** (comandos
scriptados: superfície mínima e cross-platform; **T01/T02/T03/T04/T05/T06 ✅**) — plano em
[`../proposals/comandos_scriptados.md`](../proposals/comandos_scriptados.md) e épico em
[`22_comandos_scriptados.md`](22_comandos_scriptados.md). **E19 ◐** (modelo de conhecimento rico;
**T01 ✅** Beta — D189; **T01b ✅** `drift` — D203; **T02 ✅** retenção FSRS — D190; **T03 ✅** data contract
soft — D191; **T04 ✅** PageRank/PPR — D192; **T05 ✅** comunidades — D193; **T06 ☐** reranking;
**T07 ✅** MinHash/LSH — D204; **T08 ☐** Matryoshka/ANN; **T09 ✅** claims/ontologia — D207;
**T10 ✅** TMS/drift — D208; **T11 ✅** flow metrics — D205; **T12 ✅** superfície enxuta — D209) — plano em
[`../proposals/modelo_conhecimento_rico.md`](../proposals/modelo_conhecimento_rico.md), plano
**detalhado das duas pendentes** em [`../proposals/reranking_ann.md`](../proposals/reranking_ann.md)
e épico em [`23_modelo_conhecimento_rico.md`](23_modelo_conhecimento_rico.md).

**Riscos da memória durável:** a análise crítica (deriva do curador, envenenamento, drift de
sumarização, diluição atencional, limites de host) está em
[`../proposals/riscos_memoria_duravel.md`](../proposals/riscos_memoria_duravel.md) — confirma as
defesas já presentes e deixa **dois candidatos** (check read-only de contradições no `doctor`;
`rewind --digest`). Confiança permanece **mecanismo** (E19/T01 Beta), nunca campo (D142).

**Evolução 0.5.0 (E16–E19):** a ordem integrada, o orçamento de performance (herança de E15) e
os gates de teste/bancada estão no [plano-mestre](24_plano_mestre_0.5.0.md). E19 (R1–R7) entra
inteiro em **0.5.0** (escopo único; supersede o corte 0.5.x/0.6.0). Cada épico tem uma seção
**"Performance e orçamento (herança de E15)"** com notas `**Perf:**` por tarefa.

**Superfície CLI:** o contrato dos verbos do `kd` está congelado em
[`16_cli_surface.md`](16_cli_surface.md) (v2, inspirada no Docker; decisões D57/D69/D88/D93/D94).
O aceite por verbo (pipe/`--json`/erro/exit/estado) vive em
[`17_matriz_aceitacao.md`](17_matriz_aceitacao.md).

---

## Grafo de dependências

```
E01 ── E02 ── E03 ──┬── E04
                    ├── E05 ──┐
                    │         ▼
                    └──────► E06 ── E07 ──┬── E08 ──┐
                                          ├── E09 ──┼── E12
                                          │    └── E10
                                          └── E11
E13 (testes) atravessa todos
E14 (MCP stdio) depende de E12 + E13
E15 (performance + reforma CLI) depende de E01–E14
E16 (qualidade da busca + depreciação) depende de E06/E07/E09/E10/E11/E15
E17 (robustez do worker de embeddings + `--install`) depende de E11 + E15
E18 (comandos scriptados: superfície mínima) depende de E11 + E15
E19 (modelo de conhecimento rico) depende de E06/E07/E09/E10/E11 + E16
```

---

## Definição de pronto global

Um épico só fecha quando:

- [ ] `cargo fmt -- --check` e `clippy --workspace -- -D warnings` passam.
- [ ] `cargo test --workspace` verde, incluindo as propriedades do épico.
- [ ] Nenhum arquivo de produção passa de 300 linhas; zero `unwrap/expect/panic` em `src/`.
- [ ] As decisões citadas nas tarefas têm teste ou golden que as trava.
- [ ] Docs do escopo (`MODULE.md`) e `CHANGELOG.md` atualizados.
- [ ] A matriz de aceite das tools afetadas (E13-T06) foi atualizada.
- [ ] **Logs nunca em stdout**; stdout estritamente dados (R20).
- [ ] **Nenhum segredo/corpo** aparece em log, com teste de redação (R22).
- [ ] Erros usam o `ErrorKind` e o código→exit está mapeado (R30/R31/R35).
- [ ] `unsafe` continua confinado (R01) e recursos têm teto/timeout (R11/R12/R14).

---

## Rastreabilidade decisão → épico

| Decisões | Épico |
|---|---|
| D01–D03 (IDs) | E02, E07 |
| D04–D13, D74–D75 (contrato de bytes / TOON) | E02 |
| D14–D18 (versionamento / leitura tolerante) | E02, E07, E09 |
| D19 (doctor --fix) | E09 |
| D20–D22 (escrita atômica / crash) | E03, E07 |
| D23–D28 (lock, dedup, rebuild, container) | E03, E07, E10 |
| D29–D34 (git, worktree, persistência) | E04, E08 |
| D35–D41, D81 (retrieval, ranking, RRF) | E06, E08 |
| D42, D79–D80, D83, D85, D89–D90 (embeddings) | E07, E11 |
| D43–D48 (ciclo de vida, confiança, evidence) | E07, E08, E09, E10 |
| D49–D51 (grafo e arestas) | E05, E07 |
| D52–D56 (tarefas e planos) | E07, E08, E09, E10 |
| D57–D60 (prime, hooks, onboard) | E04, E08, E12 |
| D61–D64 (config) | E04 |
| D65–D70 (arquitetura, distribuição) | E01, E12 |
| D71–D73 (saída e erros) | E12 |
| D76–D78, D82, D84, D86–D88, D91–D92 | E03, E08, E09, E13 |
| D95 (hash/IDs, gramática TOON) | E02 |
| D33, D40–D41, D47, D52–D53, D57–D58, D82, D88 (rewind, task, compact, learn) | E08 |
| D96 (registro de evento, rotação, `revision`) | E03 |
| D97 (subset TOML, guard `git -C`, `onboard`) | E04 |
| D98 (chaves de aresta, ciclo de supersessão, sugestões derivadas) | E05 |
| D16–D19, D43, D46, D48, D54–D55, D84, D86–D87 (validators, evidence, audit, doctor, âncoras, confiança) | E09 |
| D99 (catálogo de validators em TOML) | E09 |
| D28, D43–D45, D47–D48, D52, D56 (shelf-life, decay, purga, ciclos, clusters) | E10 |
| D100 (`not_before`: agendamento × expiração) | E10 |
| D42, D79–D80, D83–D85, D89–D90, D101 (provedor HTTP, cache, fila, purge, flush, eval, lightweight) | E11 |
| D163–D171 (doctor/help/verboso/prime/help embutido/redundância/docs/drain/`kd`=help) | E15 |
| Ondas O1–O9 (performance: corpus único, postings, dedup, parse, grafo, deps, `task`, coleções) | E15 |
| D172–D179 (qualidade da busca, depreciação, `contradicts`, fusão) | E16 |
| D180–D183 (worker de embeddings, verbosidade, reconciliação, supply-chain) | E17 |
| D184–D188 (comandos scriptados, `drain service`, `self upgrade`) | E18 |
| D189–D201 (confiança Beta, retenção FSRS, PPR, reranking, ontologia, TMS) | E19 |

---

## Fora de escopo (por decisão)

- **Migração de seeds/mulch** — D70: o knudge é independente; sem `migrate-from-*`.
- **FFI/WASM** — D68: apenas planejamento; hoje MCP + CLI.
- **Servidor multiusuário, auth, Docker, banco vetorial** — recusados em `05`, seção 3.
- **Tokenizador externo, modelo fixo embutido** — recusados: heurística `ceil(len/4)` e
  provedor plugável.
