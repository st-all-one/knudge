# E15 — Performance e reforma da superfície do `kd`

> **Épico de evolução (pós-E14).** Consolida os dois planos aprovados:
> [`../proposals/otimizacoes_performance.md`](../proposals/otimizacoes_performance.md) (Ondas
> O1–O8) e [`../proposals/reforma_cli.md`](../proposals/reforma_cli.md) (Fases 1–7).
>
> **Versão alvo:** **0.4.0** (o `[Não publicado]` do `CHANGELOG.md` será finalizado por
> `make update-version VERSION=v0.4.0`; toda entrada deste épico vai para lá).
>
> **Decisões:** D163 (doctor), D164 (help), D165 (verbosidade), D166 (prime compacto),
> D167 (help embutido), D168 (redundância), D169 (docs), D170 (drain), D171 (`kd` = `help`).
> **Políticas:** R20–R23 (logs), R30–R35 (erros), R42–R44 (supply chain/clippy), D84 (derivado
> reconstruível), D130/D145/D146 (superfície ask/digest), D76/D92 (testes/limites de arquivo).

## Objetivo

Deixar o `kd` **extremamente rápido** (sem mudar bytes) e **explícito** (diz o que quer e o que
fez), com `doctor`/`drain` de topo, `prime` compacto, help embutido por verbo, `kd` sozinho = help
e documentação viva e estática em sincronia. `kd task` e todo o seu entorno entram no escopo de
performance (Onda 8).

## Pré-requisitos

- E01–E14 ✅ (`make check` verde).
- Bancada `bench/` e alvo `make bench` (fora do workspace; observação, não gate — E13-T09).
- **Prioridade 1:** o núcleo da reforma (T13 `doctor`+`help`), que **não** espera performance —
  `doctor` é raro e o custo é aceito em troca de consistência/garantia/resolubilidade.
- Performance otimiza os comandos **frequentes** (`ask`/`write`/`rewind`/`prime`/`drain --status`).

## Revisão dos planos (achados e ajustes aplicados)

1. **Auto-drain × verbos novos (crítico).** `commands/idle.rs` hoje pula só
   `Command::Maintenance`; com `doctor`/`drain` de topo, ambos disparariam drain O(N). Ajustado
   em `reforma_cli.md` §7.1 e alvo de O1.5 (pular `doctor`/`drain`/`prime`/`self`).
2. **`kd doctor` é raro e prioriza consistência/garantia/resolubilidade.** O custo da validação
   completa é **aceito**; **não** espera O3. O3 (T04) segue otimizando `compact`/`learn`/`doctor`
   por conveniência, não por gate.
3. **`.idx/` binário sem plataforma de carga.** O7 (`bincode`/`rkyv`) só faz sentido com O1.6
   (carga persistida validada barata) — por isso T11 precede o item binário de T12.
4. **`prime`/docs são território compartilhado.** Goldens de `prime`, matriz, `docs/`, `SKILL.md`
   e `AGENTS.md` são consolidados em T19, mas cada tarefa de CLI atualiza a **linha da matriz** que
   tocar no mesmo commit.
5. **`--force` só com `--digest`** (já no plano): sem `--digest`, `invalid_input` (2).
6. **`Session::index()`/`graph()` viram wrappers** de `Session::corpus()` para não quebrar testes
   (O1.4).
7. **Build flags** (`target-cpu=native`, PGO) e `sha2/asm` ficam **fora** do gate (portabilidade/
   estável); só medição local.
8. **`kd task` é escopo de performance (Onda 8).** `list`/`show`/`graph`/`close`/`plan` reusam
   O1/O2/O5; ampliar a bancada para `graph`/`close`/`--sort impact` (hoje só `list` é medida).
9. **`kd` sozinho = `kd help` (D171).** Deixa de ser um atalho implícito para `prime`; intenção
   explícita exige verbo. `json_prime` passa a usar `--json prime`.

## Sequência de execução

```
Prioridade 1 (começa já): T13 (doctor + help) — não espera performance.

A. Reforma da CLI
T13 → T14 → T15 → T16 → T17 → T18

B. Performance (comandos frequentes)
T01 (feito) → T02 (O1) → T03 (O1.5) → T04 (O3) → T05 (O6.1/2)
            → T06 (O2 postings) → T07 (O2 glob) → T08 (O4 parse)
            → T09 (O5 grafo/views) → T20 (O8 task) → T10 (O6 resto)
            → T11 (O1.6) → T12 (O7 deps)

Fecho: T19 (docs/goldens/matriz/CHANGELOG) → T21 (O9 revisão de coleções)
```

## Tarefas

### E15-T01 ☑ Harness de bancada
- **Objetivo:** medir antes/depois sem `criterion` (não-objetivo E13-T09).
- **Entregáveis:** `bench/` (harness, fixture determinística, micro + e2e) com `make bench` /
  `make bench-quick`; relatório em `bench/RELATORIO.md`.
- **Aceite:** roda fora de `make check`; baseline registrado (`bench/ULTIMO.md`).

### E15-T02 ☑ O1 — leitura única do corpus
- **Escopo:** `core::corpus::Corpus` (`O1.1`; módulo de topo em vez de `store::corpus` para não
  inverter a camada store→retrieval/graph), `Graph::from_notes_ref` (`O1.2`), `embedder::pending`
  sobre `&[Note]` (`O1.3`), `Session::corpus()` + migração de `ask`/`rewind`/`doctor`/`learn`/
  `compact`/`prune` (`O1.4`).
- **Aceite:** outputs byte-idênticos (goldens), `make check` verde; A/B (`make bench`, N≈1 k) com
  `rewind` 362→295 ms (−18 %), `rewind --files` 164→101 ms (−39 %), `ask --anchor` 129→96 ms
  (−26 %); `Session::index()`/`graph()` seguem existindo como wrappers. Testes em
  `core/src/corpus/tests.rs` travam `Corpus::load == Index::from_store + Graph::build`.

### E15-T03 ☑ O1.5 — auto-drain barato e exclusão
- **Escopo:** `KNUDGE_NO_IDLE` checado **antes** de `Session::open` (via `StdEnv` na borda);
  `maybe_drain` pula `prime`/`self` além de `doctor`/`drain`/`maintenance`; `provider=none`/
  `enabled=false` já saem por `drain_once` antes de varrer o corpus.
- **Aceite:** A/B (`bench/e2e-t03.md`, N=1167) `self version` 42,9→3,5 ms (−92 %) e `prime`
  43,6→2,5 ms (−94 %); `idle_skips_prime_and_self_verbs` novo; `idle_lazy_*`/`idle_manual_*`
  seguem verdes; `kd doctor`/`kd drain` não auto-drenam.

### E15-T04 ☑ O3 — dedup sem quadrático
- **Escopo:** `propose_merges` com peneira de postings por posição (máscara reutilizável, sem o
  `find` O(N) e sem `BTreeSet` por doc), conjuntos de termos e `terms` do `statement` cacheados,
  e `Index::score_doc` exposto. A soma doc-major do BM25 fica idêntica.
- **Aceite:** A/B (N=1167) `doctor` 4,87→4,22 s (−13 %) e `compact` 2,44→2,20 s (−10 %); micromb
  `write::propose_merges` denso 1,46 s → esparso 1,6 ms (`bench/micro-t04.md`); proptest
  `sieve_matches_reference` compara com a varredura completa (pares/ordem idênticos); goldens de
  `doctor --json` inalterados. Nota: o corpus sintético é denso (vocabulário compartilhado), então
  o ganho e2e é modesto; a micromb isola o efeito assintótico.

### E15-T05 ☑ O6.1/O6.2 — `sort_unstable` + capacidade
- **Escopo:** `sort_by` → `sort_unstable_by` onde o comparador é **total** (tiebreak por `id` ou
  par único): `bm25`, `rrf`, `anchor`, `rank`, `tags`, `index`/`persist`, `semantic`,
  `suggest`, `integrity`, `next`, `manifest`, `anchors/store`, `retire`, `plan`, `usage`,
  `diff`, `learn::scoped_docs`, `merges`, `dedup`, `promote`, `task/graph`, `task/query`.
  `suggestions` e `learn::learn` **permanecem estáveis**: os comparadores omitem `targets`/`why`
  e não são totais (reordenar mudaria bytes). Capacidade em `build_hits`, `cyclic_components`,
  `strongly_connected`, `finishing_order` e `Store::list_ids`.
- **Aceite:** ordem determinística preservada (goldens/proptest verdes); `make check` verde; A/B
  neutro no e2e (`bench/t05.md`) — ganho é de consistência (sem scratch da stable sort) e de
  alocação, não de tempo de parede neste corpus.

### E15-T06 ☑ O2 — postings (peneira) + BM25
- **Escopo:** `retrieval::postings::Postings` (índice invertido derivado, cacheado por `OnceLock`
  no `Index`, nunca persistido); `score_with` pontua só candidatos com ≥1 termo quando a peneira
  compensa, com fallback por `df` para varrer quando os termos cobrem ≥ metade do corpus;
  `field_sum` faz hoisting de `norm`/`weight` e reusa `idf_from`; `allowed` segue checado por
  candidato (não por máscara — a máscara não paga em corpus denso). A soma doc-major é mantida.
- **Aceite:** ranking/score **byte-idênticos** (goldens + `score_matches_manual_scan` + proptest
  `sieve_positions_match_scan`); `make check` verde; micro `Postings::build` 4,26 ms e
  `Index::score` neutro (1,368→1,392 ms) no corpus denso; A/B e2e neutro — o vocabulário denso do
  corpus sintético faz o fallback varrer, evitando a regressão que o índice invertido traria a
  `write`/`ask`. O ganho de `ask` é real só em vocabulário seletivo (não medível aqui).

### E15-T07 ☑ O2.3 — glob sem alocação
- **Escopo:** `GlobPattern` compilado (tokens uma vez) + matcher com **matriz DP de uma linha**
  (1 `Vec` em vez de `tokens.len()+1` por par); `match_note` compila a âncora uma vez por nota.
- **Aceite:** testes de glob existentes + `glob_edge_cases` + proptest `glob_matches_reference`
  (oráculo recursivo independente) provam a semântica; `make check` verde; micro `glob_match`
  **223 ns** e `GlobPattern::matches` **138 ns**; e2e dentro do ruído (`ask --anchor`,
  `rewind --files`) — o custo absoluto do glob não domina esses comandos.

### E15-T08 ☑ O4 — normalize/hash/TOON/JSONL
- **Escopo:** `normalize` fast-path ASCII + `normalize_into` (O4.1); `short_hash_parts`/`hex8_value`
  para `body_hash`/`note_id` sem concatenar (O4.2/O4.3); `base36_8` em `[u8; 8]`; lexer TOON com
  `Line<'a> { text: Cow<'a, str> }` e `strip_comment -> Cow` (O4.4); JSONL `with_capacity` +
  `is_sorted` (O4.7). O4.5/O4.6 (emissor direto/`flow` com `Cow`) ficam para T10.
- **Aceite:** proptests de `normalize`/`body_hash`/`note_id` e round-trip TOON/JSONL verdes;
  `make check` verde; micro (N=1167): `normalize` −89 % (1,98→0,22 µs), `body_hash` −84 %
  (2,92→0,47 µs), `note_id` −84 % (1,17→0,18 µs), `base36_8` −63 %, `toon::parse` −21 %,
  `Note::parse` −8 %; e2e dentro do ruído (a leitura do corpus do disco domina).

### E15-T09 ☑ O5 — grafo, views e manifest
- **Escopo:** `Graph` com índice reverso `parents: BTreeMap<String,String>` (O5.1), derivado em
  `from_nodes` (1º vencedor na ordem do `BTreeMap` = semântica anterior); `next_tasks_in`
  pré-computa `impact` uma vez (O5.3); `manifest_at` computa `compute_views` uma vez e repassa às
  variantes `*_in` de `next_tasks`/`manifest_text` (O5.2). O5.4 (`rank_with`) fica para T10.
- **Aceite:** testes de grafo/views/manifest verdes (saída idêntica); `make check` verde; A/B
  (N=1167): `rewind` **319→134 ms (−58 %)** e `rewind --json` **309→113 ms (−63 %)**; `knowledge
  map` estável (não usava o caminho O(N²)).

### E15-T10 ☑ O6 restante
- **Escopo:** `content_terms` por `len()` (O6.3); `query_terms` com `BTreeSet<Cow>` (O6.4);
  `logging::init` fast-path (`off` sem subscriber + `LevelFilter`, O6.5); `#[cold]` nos
  construtores de `Error` e `#[inline]` em `Index::tf`/`len` (O6.6). O5.4 (`rank_with`) foi
  medido (`handoff::rank` = 175 µs) e dispensado por ficar abaixo do ruído.
- **Aceite:** testes de retrieval/logging verdes; `make check` verde; micro: `content_terms`
  2,13→1,91 µs (−10 %); piso fixo estável (dominado por startup do processo).

### E15-T11 ☑ O1.6 — carga persistida do `.idx/` com validação
- **Escopo:** `Index::load_if_fresh`/`open` com **validação de frescor por `mtime`** (índice só é
  servido se for ≥ todas as notas; ausente/desatualizado/ilegível → rebuild + aviso) e
  `Corpus::load_fresh` (lê notas uma vez, reusa ou reconstrói o índice). Nunca serve índice
  parcial (E13-T03). **Não habilitado** por padrão: medido, decodificar o `retrieval.jsonl`
  custa **13,8 ms** (N=1167) contra **8,5 ms** do rebuild, então ligar regride `ask`/`rewind`/
  `task graph` em ~10–15 ms. Fica pronto como gancho do formato binário de T12.
- **Bônus (correção da bancada):** `timed` passa a exigir exit 0; `task list` medido com
  `--universe`; `config get/set` com `--key/--value`; `forget` idempotente. Sem isso, comandos
  inválidos viravam “ganhos” falsos.
- **Aceite:** testes `load_fresh_*` (reusa, reconstrói por mtime, reconstrói corrompido) e
  `open_rebuilds_when_absent` verdes; `make check` verde; rebuild byte-idêntico; baseline
  corrigido em [`t11.md`](../../bench/t11.md) e regressão da carga em [`t11-cached.md`](../../bench/t11-cached.md).

- **Aceite:** `make check` + `make ci` verdes; `cargo tree` dentro do orçamento justificado.

### E15-T20 ☑ O8 — `kd task` ponta a ponta
- **Escopo:** `task::impacts` (O8.2) substitui as chamadas por id em `next_tasks` e
  `task list --sort impact`; `Session::notes`/`Corpus::load_notes` (O8.1) e `task list`/`task
  graph` derivam grafo e notas do **mesmo** vetor (sem releitura); `task graph` resolve raízes e
  renderiza a partir do mapa em memória (O8.4). O8.5/O8.6/O8.8 (close/show/plan) já se beneficiam
  de O1/O5; O8.3/O8.9 ficam como refinamento incremental.
- **Depende de:** T02 (corpus), T06 (postings) e T09 (grafo/views).
- **Aceite:** testes de impacto/`task` verdes (proptest `impacts_matches_per_id_impact`);
  `make check` verde; A/B (N=1167): `task graph` 115→77 ms e `task list --ready` 99→76 ms; micro:
  `task::impacts` (todos os ids) ≈ 150 µs = custo de **um** `impact`. Nota: os números de
  `task list`/`--sort impact`/`--full-content` desta bancada eram **falhas rápidas** (sem
  `--universe`); corrigidos em T11 (`--universe` + `timed` com assert de exit).

### E15-T13 ☑ Fase 1 — `kd doctor` de topo + `kd help` (D163/D164)
- **Escopo:** `Command::Doctor(DoctorArgs{fix,explain})`; remover `maintenance doctor` e `--audit`;
  `commands/doctor/` combina `doctor`/`doctor_fix` + `audit`, com `próximos:` e `--explain`
  (`esperado`×`encontrado`×`ação`); `kd help` == `kd --help`.
- **Garantia:** `healthy` só com **zero achados**; advisórios são `degraded`; validação máxima
  (13 checks + auditoria), mesmo que custe tokens/tempo.
- **Resolubilidade:** `--explain` cobre **todo** achado; `--fix` re-audita e lista o **residual**
  com o comando exato para concluir.
- **Depende de:** nada (independente de performance — é raro e o custo é aceito).
- **Aceite:** testes migrados para `kd doctor`; novo teste de `--explain` (esperado/encontrado/
  ação) e de garantia (`healthy` ⇔ zero achados); `kd maintenance` sem `doctor` → exit 2; linha da
  matriz e `prime` atualizados.

### E15-T14 ☑ Fase 2 — `kd drain` de topo (D170)
- **Escopo:** `Command::Drain(DrainArgs{digest,status,force})`; sem flag = help; `--digest` (log
  mínimo) e `--digest --force` (apaga `.idx/` e redigeri tudo); `--status` rico + recomendação;
  remover `KnowledgeCommand::Digest`.
- **Depende de:** T03 (auto-drain).
- **Aceite:** testes de `knowledge digest` migrados; `--force` sem `--digest` → exit 2; benchmark
  do `--status`.

### E15-T15 ☑ Fase 3 — `prime` compacto (D166)
- **Escopo:** `PrimeFormat::{Compact,Long}`, default Compact, `--long` completo; `init` usa
  Compact.
- **Aceite:** goldens `prime.txt`/`json_prime.json` regenerados com intenção; `kd prime`
  byte-idêntico; `kd` sozinho = `kd help` (D171).

### E15-T16 ☑ Fase 4 — verbosidade (D165)
- **Escopo:** `init`/`self`/`config`/`sync`/`maintenance` explícitos (o que fizeram) e
  `maintenance` com `próximos:`; logs `info` via porta `Logger`, stdout=dados.
- **Aceite:** testes de saída ajustados; `--json 2>/dev/null` continua JSON válido.

### E15-T17 ☑ Fase 5 — help embutido + `kd` = `help` (D167/D171)
- **Escopo:** `arg_required_else_help` + `long_about`/`after_help` por verbo com exemplos,
  “quando NÃO usar” e escopo explícito (`--universe`); `kd` sozinho imprime o help (D171) e
  `--json` sem verbo é `invalid_input` (2).
- **Aceite:** `kd` sem args mostra help (exit 0); `kd <verbo>` sem args mostra help; `kd help
  <verbo>` funciona; `ask` vazio mantém exit 2 (D130) com texto de ajuda; `json_prime` em
  `--json prime`.

### E15-T18 ☑ Fase 6 — redundância (D168)
- **Escopo:** remover `visible_alias="anchors"` e aliases residuais de `body`; um vocabulário por
  conceito; corpo documentado num só lugar.
- **Aceite:** testes de regressão para cada alias removido (exit 2).

### E15-T19 ☑ Fase 7 — documentação (D169)
- **Escopo:** `16_cli_surface.md`, `17_matriz_aceitacao.md`, `docs/*`, `SKILL.md`, `AGENTS.md`,
  `llms.txt`, `README.md`, `CHANGELOG.md`, `MODULE.md`.
- **Aceite:** grep por `maintenance doctor`, `--audit`, `knowledge digest`, `--re-digest`, `kd`
  sem argumentos = `prime` vazio; `make check` verde.

### E15-T21 ☐ O9 — revisão de coleções (fecho)
- **Objetivo:** varrer o código pelos padrões de `.agents/skill/rust/05-collections.md` e
  incorporar só o que dá ganho **sem mudar bytes**; registrar o que é rejeitado.
- **Escopo (incorporar):** `entry` onde há `contains_key` + `insert`/`get_mut`
  (`toon::flow::insert`, `config::toml::parse::insert_leaf` + auditoria em `schema`/`health`/
  `store`); chave **emprestada** (`&str`/`Cow`) em mapas temporários que hoje fazem `to_string()`
  só para servir de chave (extensão de O6.4); `Vec::with_capacity`/`try_reserve` residual;
  reconferir `sort_unstable_by` (comparador total) e `#[cold]`/`#[inline]` (T05/T10).
- **Escopo (avaliar sem aplicar cego):** `swap_remove` só onde a ordem **não** é contrato e há
  re-sort total depois; no knudge a ordem é contrato (TOON/goldens) — documentar cada caso.
- **Rejeitado (registrar):** `HashMap`/`HashSet` (proibidos por `clippy.toml`; determinismo exige
  `BTreeMap`/`IndexMap`); `par_lines`/`rayon` como padrão local (contraria R43 e o determinismo;
  só pela Onda 7, com gate de dependência).
- **Depende de:** T05/T10 (padrões já aplicados) — é o fecho que varre o residual.
- **Aceite:** `make check` verde; nenhum byte alterado (goldens/proptest); micro do que mudou;
  decisão escrita (incorporado/rejeitado) por padrão neste épico.

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [ ] Bytes idênticos nos goldens (TOON/JSONL/`prime`/`--json`) salvo T15 (intencional).
- [ ] Ganho medido por A/B em todas as tarefas de performance (`bench/ULTIMO.md` atualizado).
- [ ] `doctor`/`drain` de topo, `--audit` e `knowledge digest` inexistentes; auto-drain não os
      dispara; `kd` sozinho = `kd help` (D171); `kd task` otimizado (O8).
- [ ] Toda tarefa com linha na matriz de aceite e `CHANGELOG.md` atualizado.
- [ ] O9 revisada: cada padrão da skill de coleções tem decisão (incorporado/rejeitado) e o
      residual varrido.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`.

## Não-objetivos

- `criterion` como gate (E13-T09 / R43).
- `tokio full`/servidor/DB (R16/R43).
- Aliases de retrocompatibilidade da superfície antiga (D14).
- `target-cpu=native`/PGO no artefato distribuído (só medição local).
- `HashMap`/`HashSet` e `par_lines`/`rayon` como padrão local (determinismo/R43).

## Riscos

| Risco | Mitigação |
|---|---|
| O3 não fecha o quadrático em vocabulário denso | peneira + `terms` cacheados reduzem constante; `learn` (cap 64) é o teto; medir |
| `doctor` completo lento | esperado: é raro; prioriza garantia/resolubilidade; O3 é melhoria opcional |
| `prime` compacto quebra cache | versão em `data.version`; goldens no mesmo commit |
| verbosidade vaza para `--json` | texto extra só em `data`; stdout máquina inalterado |
| dep não paga o custo | A/B ≥20 % + `cargo tree`; reverter sem hesitar |
| loop no `drain --digest` | parar em `pending == 0` ou sem progresso |
| `--digest --force` apaga `.idx/` | 100 % derivado (D84); `warnings` listam o removido |
