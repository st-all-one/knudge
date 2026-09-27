# Plano-mestre 0.5.0 — evolução pós-E15 (E16–E19)

> **Status:** guia (não implementado). **Não** substitui os épicos nem as decisões: **aponta**
> onde cada coisa vive, a **ordem** de implementação e os **gates** (teste + bancada) que
> governam a evolução. Toda mudança de comportamento/contrato continua exigindo `Dxx`; toda
> mudança de superfície, uma linha em `16_cli_surface.md`/`17_matriz_aceitacao.md`.
>
> **Escopo:** E16 (qualidade da busca + depreciação), E17 (worker/`--install`), E18 (comandos
> scriptados) e E19 (modelo de conhecimento rico). **Versão:** **0.5.0** (escopo único; ver §5).
> **Base:** E15 ✅ / v0.4.0 ([`19_performance_reforma_cli.md`](19_performance_reforma_cli.md)).

## 0. Como usar

1. Leia este plano **uma vez** para situar ordem, referências e gates.
2. Trabalhe **uma trilha/épico por vez**, na ordem de §4; cada tarefa é um commit.
3. Antes de codar uma tarefa, leia o **épico** (aceite) e a **proposta** (motivação/medição) dela.
4. Toda tarefa fecha com: `make check` verde + teste/golden do aceite + A/B na bancada (se tocar
   o quente) + linha de superfície (se tocar nomes).
5. Se o número não paga (R43) ou regride sem ganho, **reverta** e registre "rejeitado por medição"
   no épico (padrão E15-T12).

## 1. Documentos-fonte (autoridade única)

| Assunto | Autoridade | Papel |
|---|---|---|
| Superfície de verbos/flags | [`16_cli_surface.md`](16_cli_surface.md) | **contrato** de nomes; 1 linha por commit que toca |
| Aceite por verbo | [`17_matriz_aceitacao.md`](17_matriz_aceitacao.md) | pipe/`--json`/erro/exit/estado |
| Decisões de produto | [`../03_decisoes-fechadas.md`](../03_decisoes-fechadas.md) | `D01–D171` + as novas (`D172+`) |
| Políticas de engenharia | [`14_revisao_tecnica.md`](14_revisao_tecnica.md) | `R01–R44` |
| Lints | [`15_clippy_config.md`](15_clippy_config.md) + [`clippy.toml`](clippy.toml) | rigor máximo |
| Performance (baseline) | [`../../bench/RELATORIO.md`](../../bench/RELATORIO.md) | números de v0.4.0 |
| Performance (harness) | [`../../bench/README.md`](../../bench/README.md) | `make bench`, micro/e2e |
| Qualidade (baseline) | `bench/qualidade.md` *(novo, E16/T01)* | Recall@k/MRR/nDCG@k |
| Padrões Rust | [`../../.agents/skill/rust/SKILL.md`](../../.agents/skill/rust/SKILL.md) | hot path, coleções, testes |
| Bordas | [`../../DIVERGENCES.md`](../../DIVERGENCES.md) | 1 linha + teste por borda |
| Revisão integrada | [`../proposals/revisao_integrada.md`](../proposals/revisao_integrada.md) | conflitos C1–C20 resolvidos |
| Índice dos épicos | [`README.md`](README.md) | fases, grafo e status |

**Propostas** (motivação/medição): [`qualidade_busca_depreciacao`](../proposals/qualidade_busca_depreciacao.md)
(E16), [`worker_embeddings_install`](../proposals/worker_embeddings_install.md) (E17),
[`comandos_scriptados`](../proposals/comandos_scriptados.md) (E18),
[`modelo_conhecimento_rico`](../proposals/modelo_conhecimento_rico.md) (E19).

**Épicos** (tarefas/aceite): [`20`](20_qualidade_busca_depreciacao.md),
[`21`](21_worker_embeddings_install.md), [`22`](22_comandos_scriptados.md),
[`23`](23_modelo_conhecimento_rico.md). Cada épico traz uma seção **"Performance e orçamento
(herança de E15)"** com o orçamento e as notas `**Perf:**` por tarefa.

## 2. Princípios não negociáveis

1. **`make check` verde** antes de concluir qualquer tarefa (`fmt` + `clippy -D warnings` +
   `test` + gate de 300 linhas); `make ci` no fecho do épico.
2. **Bytes de `notas/` = contrato.** Só mudam com `Dxx` + `schema_version` + rebuild (D04/D95);
   `id`/`body_hash` derivam de `normalize` e **não** mudam por melhoria de busca (D172).
3. **Determinismo.** `BTreeMap`/`IndexMap`; nada de `HashMap`/`HashSet`/`rayon`; toda ordem é
   contrato (TOON/goldens).
4. **Dependência nova só com A/B ≥ 20 %** (R43); `default-features=false`; licença no `deny.toml`.
5. **`criterion` não é gate** (E13-T09); a bancada **observa**, não trava o CI.
6. **stdout = dados, stderr = logs** (R20); `--json` nunca vaza log; EPIPE → exit 0 (D73).
7. **Simplicidade e localidade** (AGENTS §0): recusar infra que o caso local não paga.

## 3. Gates: teste + bancada guiando a evolução

Cada tarefa só fecha com as provas aplicáveis:

| Prova | Ferramenta | Quando |
|---|---|---|
| **Correção** | `cargo test` + `proptest` + golden | sempre |
| **Contrato** | goldens TOON/JSONL/`prime`/`--json` | quando toca bytes/saída |
| **Qualidade** | `bench/qualidade.md` (Recall@k/MRR/nDCG) | quando muda resultado de busca |
| **Latência** | `make bench` (micro + e2e, com/sem `--no-idle`) | quando toca o quente |

**Orçamento (tolerância):** perda de latência é aceitável **só** com ganho de qualidade medido;
micro ≤ **+20 %**, e2e quente ≤ **+15 %** por tarefa (≤ **+25 %** acumulado por épico byte-free;
≤ **+40 %** em E19), **nunca** 2×. Detalhe por épico na seção "Performance e orçamento".

**Regra de reversão:** se a tarefa não move o ponteiro (ou regride sem ganho), **reverte** e
registra a rejeição no épico.

**Evolução dos baselines:** `bench/ULTIMO.md`/`ULTIMO.json` são atualizados a cada tarefa;
`bench/ULTIMO-v0.4.0.*` fica **congelado** como referência da herança de E15; `bench/qualidade.md`
é o baseline de qualidade (E16/T01). Recortes por tarefa em `bench/<tarefa>.md`.

## 4. Ordem de implementação integrada

> Cada item abaixo é uma tarefa (commit). A ordem respeita as dependências resolvidas em
> [`revisao_integrada.md`](../proposals/revisao_integrada.md) §3 (C1–C20).

### Trilha A — Medição e correções baratas (E16)

1. ✅ **E16/T01** — bancada de qualidade (`bench/src/quality.rs`, `make bench-quality`;
   baseline `bench/qualidade.md`).
2. ✅ **E16/T02** — status consistente (`Status::VISIBLE`, D176; teste de regressão).

### Trilha B — Superfície e scripts (E18 + E17)

3. ✅ **E18/T01** — wrapper fino + stream (`commands/script.rs`; checksum SHA-256 no remoto).
4. ✅ **E18/T02** — cross-platform (`bash`/PowerShell; D185).
5. ✅ **E17/T03** — verbosidade/stream do worker (entregue por E18/T01).
6. ✅ **E17/T01** — reconciliação de `endpoint`/`model` (no script; D182).
7. ✅ **E17/T02** — `--status` com probe de endpoint (D182).
8. ✅ **E17/T04** — remover confirmação/`--yes` (D180).
9. ✅ **E18/T03** — `watch-service` como script (consolida E17/T03/T04).
10. ✅ **E18/T04** — superfície `kd drain service` (D186).
11. ✅ **E17/T05** — supply-chain; **E17/T06** — uninstall/GGUF; **E17/T07** — polimento P6–P10.
12. ✅ **E18/T05** — `self upgrade` real; **E18/T06** — auditoria de verbos acionáveis.

### Trilha C — Léxico, confiança e ranking (E16 + E19 R1/R4)

13. ✅ **E16/T03** — acentos; **E16/T04** — alta frequência.
14. ✅ **E19/T01** — Beta (absorve E16/D174); ✅ **E19/T01b** — `drift` persistido (D203); ✅ **E19/T02** — FSRS; ✅ **E19/T03** — obrigatoriedades-soft.
15. ✅ **E19/T04** — PageRank/PPR (D192); ✅ **E19/T05** — comunidades (D193).
16. ✅ **E16/T06** idade (D175); **E16/T07** `contradicts` (D177); **E16/T09** fusão (D179).
17. **E19/T06** — reranking/expansão/fusão.
18. ✅ **E19/T07** — MinHash/LSH (D204); ☐ **E19/T08** — Matryoshka/ANN.

### Trilha D — Ontologia, razão e tarefa (E19 R5+)

19. ✅ **E19/T09** — claims SPO + ontologia + proveniência (**D207**, schema bump 1→2 + rebuild).
20. ✅ **E19/T10** — TMS/defeasible + drift KL/JS (**D208**); ✅ **E19/T11** — flow metrics/caminho crítico (D205).
21. ✅ **E19/T12** — superfície enxuta (≤10 verbos; **D209**: fim do verbo `knowledge`).

### Fecho

22. ✅ **E16/T10** — perf do caminho de busca e de `prune` (walk único + tokenização única);
    ✅ **E16/T11** — stemming (D206, adotado por medição); ✅ **E16/T12**, ✅ **E17/T08**,
    ✅ **E18/T07**, ✅ **E19/T13** — docs/goldens/matriz/`CHANGELOG` (`[0.5.0]`) +
    `make update-version VERSION=v0.5.0`. **0.5.0 fechado.**

## 5. Versionamento (0.5.0)

Por diretriz do usuário, **todo** o escopo E16–E19 entra em **0.5.0** — escopo único, sem corte
0.5.x/0.6.0 (supersede a decisão Q4 de `revisao_integrada.md` §4). As trilhas de §4 são **fases
internas** da mesma release; a Trilha D é a fase final. O bump de `schema_version` de E19/T09 é
**ortogonal** à versão do crate (entra com rebuild + goldens).

> **Fallback documentado:** se o custo da Trilha D for grande demais, ela pode ser destacada para
> 0.6.0 **sem mudar a ordem** (basta mover os itens 19–21 para o próximo ciclo). Nesse caso,
> atualizar o cabeçalho de E19 e o `CHANGELOG` no fecho.
>
> **Resultado (0.5.0):** a Trilha D foi **executada na própria 0.5.0** — **E19/T09** claims SPO +
> ontologia + proveniência (**D207**, `schema_version` 1→2), **E19/T10** TMS/defeasible + drift
> KL/JS (**D208**) e **E19/T12** superfície enxuta (**D209**). Os itens 1–22 entraram na mesma
> versão; o `CHANGELOG` foi unificado em `[0.5.0]`. Restam apenas **E19/T06** (reranking) e
> **E19/T08** (Matryoshka/ANN), dependentes de 2º modelo/escala, com plano em
> [`../proposals/reranking_ann.md`](../proposals/reranking_ann.md).

O `CHANGELOG.md` consolida tudo em `[0.5.0]` no fecho (E16/T12, E17/T08, E18/T07, E19/T13);
`make update-version VERSION=v0.5.0` sincroniza `Cargo.toml`/lock/goldens/`install.sh`/README.

## 6. Matriz de tarefas (ordem, contrato e gate)

| # | Tarefa | Superfície | Bytes | Gate principal |
|---|---|---|---|---|
| 1 | E16/T01 bancada de qualidade | — | — | `bench/qualidade.md` |
| 2 | E16/T02 status consistente | — | — | regressão + golden |
| 3 | E18/T01 wrapper fino + stream | — | — | teste de stream |
| 4 | E18/T02 cross-platform | — | — | smoke por SO |
| 5 | E17/T03 verbosidade/stream | — | — | teste de stream |
| 6 | E17/T01 reconciliação | — | — | teste de divergência |
| 7 | E17/T02 `--status` probe | `--json` aditivo | — | teste de probe |
| 8 | E17/T04 sem confirmação | `--yes` sai | — | matriz |
| 9 | E18/T03 `watch-service` script | — | — | smoke |
| 10 | E18/T04 `drain service` | **sim** | — | matriz + `prime` |
| 11 | ✅ E17/T05–T07 supply/polish | — | — | testes de checksum/`--every` |
| 12 | ✅ E18/T05–T06 `self upgrade`/auditoria | **sim** | — | teste + decisão escrita |
| 13 | ✅ E16/T03/T04 acentos/alta freq. | — | derivado | qualidade + goldens |
| 14 | ✅ E19/T01–T03 R1 (Beta/FSRS/obrig.) | — | — | proptest + A/B |
| 15 | ✅ E19/T04/T05 R4 (PPR/comunidades) | — | — | proptest + A/B |
| 16 | ✅ E16/T06 idade/T07 `contradicts`/T09 fusão | — | — | qualidade + goldens |
| 17 | E19/T06 reranking | `--json` aditivo | — | A/B qualidade |
| 18 | E19/T07/T08 R3 (MinHash/ANN) | — | `.idx/` | proptest + A/B |
| 19 | E19/T09 R5 (SPO/ontologia) | — | **sim** | goldens + rebuild |
| 20 | E19/T10/T11 R6/R7 | `--json` aditivo | — | proptest + teste |
| 21 | E19/T12 superfície enxuta | **sim** | — | matriz |
| 22 | ✅ E16/T10 perf + E16/T11 stemming + fechos | — | derivado | `make ci` + `CHANGELOG` |

## 7. Rastreabilidade (decisão → épico → tarefa → gate)

| Decisões | Épico | Tarefas | Gate |
|---|---|---|---|
| D172, D173, D175, D176, D177, D179 | E16 | T02–T04, T06, T07, T09 | qualidade + goldens |
| D180–D183 | E17 | T01–T07 | teste + matriz |
| D184–D188 | E18 | T01–T06 | smoke + matriz |
| D189, D190 (absorvem D174/D178) | E19/E16 | E19/T01, T02 | proptest + A/B |
| D191–D201 | E19 | T03–T12 | proptest/goldens + A/B |

As decisões **definitivas** são registradas em [`../03_decisoes-fechadas.md`](../03_decisoes-fechadas.md)
**na implementação** de cada onda (o número `Dxx` sai no commit que a adota).

## 8. Checklist de release 0.5.0

- [x] `make check` + `make ci` verdes; `cargo tree` dentro do orçamento (R43).
- [x] `bench/ULTIMO.md` + `bench/qualidade.md` atualizados; baseline 0.5.0 congelado em
      `bench/ULTIMO-v0.5.0.*` (o 0.3.3 fica em `ULTIMO-v0.3.3.*`).
- [x] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`.
- [x] Bytes de `notas/` **idênticos** para as notas v1 (D207 é aditivo); `schema_version` 1→2 e o
      rebuild de notas antigas é byte-idêntico.
- [x] Superfície: `16_cli_surface.md` + `17_matriz_aceitacao.md` + `prime` em sincronia.
- [x] `DIVERGENCES.md` com 1 linha + teste por borda nova (até #112).
- [x] `CHANGELOG.md` `[0.5.0]` (unificado, inclui a Trilha D); `make update-version VERSION=v0.5.0`.
- [x] `MODULE.md`/`docs/`/`SKILL.md`/`llms.txt`/`README.md`/`ARCHITECTURE.md` atualizados.

> **Trilha D em 0.5.0:** E19/T09 (claims SPO + ontologia + proveniência — **D207**), E19/T10
> (TMS/defeasible + drift KL/JS — **D208**) e E19/T12 (superfície enxuta — **D209**) foram
> **fechados na própria 0.5.0**. As duas pendências restantes — **E19/T06** (reranking) e
> **E19/T08** (Matryoshka/ANN) — dependem de um 2º modelo/escala e têm plano de execução detalhado
> em [`../proposals/reranking_ann.md`](../proposals/reranking_ann.md).

## 9. Riscos globais

| Risco | Mitigação |
|---|---|
| Ganho de qualidade não paga a latência | orçamento por épico (§3); medir antes de adotar; reverter |
| E19/T09 (schema bump) quebra o contrato | onda dedicada e última; `schema_version` + rebuild + goldens |
| Conflito de superfície entre E17/E18/E19 | **um** ponto de verdade (`16_cli_surface.md`); 1 linha por commit |
| Determinismo (PPR/LSH/ANN) | tolerância/seed fixas; ordem canônica; fallback exato |
| Acúmulo de regressão entre tarefas | A/B por tarefa + acumulado por épico; baseline congelado |

## 10. Pontos em aberto

- _(a preencher)_ — o usuário indicou que ainda tem pontos a acrescentar antes do código; cada
  ponto vira uma tarefa/nota no épico correspondente e é refletido na ordem de §4.
