# Implementação do knudge — índice dos épicos

> Plano de implementação do binário **`kd`** e do diretório **`.knudge/`**, derivado dos
> documentos da raiz de `plan/`: `00_panorama.md` (visão), `01_gaps-plan-rs.md` (bordas),
> `02_decisoes.md` (pontos), `03_decisoes-fechadas.md` (D01–D101 ✅), `04_embeddings.md`
> (vetores) e `05_refs-agnostic-rag.md` (extrações do arags).
>
> Cada arquivo desta pasta é um **épico**; cada épico tem **tarefas** com aceite verificável.
> A ordem dos arquivos é a ordem de dependência.

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

**MVP = Fase 0 + Fase 1** (+ leitura tolerante mínima de E09). O resto é incremental.

**Status:** E01–E13 ✅ (`make check` verde; 423 testes). Projeto completo pelo plano; evolução
segue as decisões `Dxx` e as políticas `Rnn`.

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

---

## Fora de escopo (por decisão)

- **Migração de seeds/mulch** — D70: o knudge é independente; sem `migrate-from-*`.
- **FFI/WASM** — D68: apenas planejamento; hoje MCP + CLI.
- **Servidor multiusuário, auth, Docker, banco vetorial** — recusados em `05`, seção 3.
- **Tokenizador externo, modelo fixo embutido** — recusados: heurística `ceil(len/4)` e
  provedor plugável.
