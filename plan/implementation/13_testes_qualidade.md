# E13 — Testes e qualidade

> **Transversal.** O knudge não tem um produto de referência; o análogo do *harness
> diferencial* é **golden/snapshot + property tests + stress de concorrência + crash-injection**,
> mais `DIVERGENCES.md` e a **matriz de aceite por tool**.
>
> **Decisões:** D72, D76, D77, D78, D92.
> **Políticas:** R01, R11, R42, R43 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Travar as bordas que mais custam (Unicode, ordem, hash, lock, atomicidade, TOON, RRF) e impedir
que uma feature seja considerada “pronta” sem contrato verificado.

## Pré-requisitos

Transversal — inicia junto com E01 e cresce com os épicos.

## Tarefas

### E13-T01 ☐ Golden / snapshot
- **Objetivo:** snapshots versionados de TOON (round-trip), formatos de saída (`recall` pipe,
  `--json`), mensagens de erro e `prime`.
- **Entregáveis:** corpus + snapshots; runner de comparação.
- **Decisões:** D72, D76.
- **Aceite:** qualquer mudança de bytes/mensagem reprova o snapshot até ser aprovada.

### E13-T02 ☐ Property tests
- **Objetivo:** propriedades dos puros: TOON round-trip; **RRF** (determinismo, união,
  monotonicidade); **decay/confiança** (monotonicidade, [0,1]); **IDs** (idempotência);
  normalização de hash.
- **Entregáveis:** suítes `proptest`.
- **Decisões:** D76, D92.
- **Aceite:** `proptest` roda em CI com seed fixa; contador de casos documentado.

### E13-T03 ☐ Stress de concorrência
- **Objetivo:** CLI + MCP simultâneos, rebuild × `recall`, writes concorrentes no mesmo alvo,
  reclaim de lock.
- **Entregáveis:** testes de stress determinísticos por scheduling.
- **Decisões:** D76, D23–D27.
- **Aceite:** nenhum lost update; nenhum leitor vê índice parcial; sem deadlock ABBA.

### E13-T04 ☐ Crash-injection
- **Objetivo:** matar o processo em pontos-chave (entre tmp/rename, entre nota/evento, durante
  flush do índice) e verificar recuperação.
- **Entregáveis:** harness de crash; `doctor` reconstrói.
- **Decisões:** D76, D20, D21, D85.
- **Aceite:** invariantes “canônico antes do derivado” e “nota antes do evento” preservadas.

### E13-T05 ☐ `DIVERGENCES.md` do knudge
- **Objetivo:** catalogar as bordas: Unicode (NFC/contagem), hash, ordem de chaves, lock,
  atomicidade, TOON, RRF/tie-break, timestamps — com mitigação e teste.
- **Entregáveis:** documento na raiz.
- **Decisões:** D77.
- **Aceite:** cada divergência aponta para o teste que a trava.

### E13-T06 ☐ Matriz de aceite por tool
- **Objetivo:** para cada tool: formato pipe, `--json`, erro, exit code e estado do `.knudge/`
  esperado.
- **Entregáveis:** matriz em markdown.
- **Decisões:** D78.
- **Aceite:** toda tool tem linha; CI valida os itens automatizáveis.

### E13-T07 ☐ Gate de CI
- **Objetivo:** `fmt --check`, `clippy --workspace --all-targets -D warnings`, `test`, gate de
  linhas, doc-tests, `nextest`, todos lendo `clippy.toml`.
- **Entregáveis:** pipeline com `cargo fmt`, `clippy`, `nextest`, `deny`, `audit`, `machete`,
  `typos`.
- **Decisões:** D92. **Políticas:** R42, R44.
- **Aceite:** PR sem o gate não passa; `clippy.toml` e `[workspace.lints]` aplicados.

### E13-T08 ☐ Verificação dinâmica de memória e concorrência
- **Objetivo:** pegar UB e corridas que testes comuns não pegam.
- **Entregáveis:** `cargo miri test` nos crates puros; `loom` nos primitivos concorrentes
  (lock/RRF); fuzz do parser TOON e do leitor JSONL; `cargo geiger` para auditar `unsafe`.
- **Decisões:** D76. **Políticas:** R01, R11.
- **Aceite:** Miri/loom verdes; fuzz sem panic no corpus; `unsafe` só em `embeddings`.

### E13-T09 ☐ Supply chain, cobertura e benchmark
- **Objetivo:** dependências, cobertura e desempenho observáveis.
- **Entregáveis:** `cargo deny` (licenças/advisories) + `cargo audit` + `cargo machete` + `typos`
  no CI; cobertura `llvm-cov`/`tarpaulin`; `criterion` para retrieval (observação, não gate);
  `cargo tree` com orçamento de dependências.
- **Decisões:** D76. **Políticas:** R42, R43.
- **Aceite:** CI falha em advisory/licença proibida; cobertura publicada; benchmark roda.

## Definition of Done

- [ ] Golden, proptest, stress e crash-injection verdes e em CI.
- [ ] `DIVERGENCES.md` e a matriz de aceite publicados e mantidos.
- [ ] Uma feature só é “pronta” com a linha da matriz correspondente verificada.
- [ ] Miri/loom/fuzz e supply chain no CI.

## Não-objetivos

- Benchmark de performance como critério de aceite (é observação, não bloqueio).
