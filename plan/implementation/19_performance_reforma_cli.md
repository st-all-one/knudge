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

Fecho: T19 (docs/goldens/matriz/CHANGELOG)
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

### E15-T04 ☐ O3 — dedup sem quadrático
- **Escopo:** `propose_merges` com lookup O(1) (`by_id`), `terms` do `statement` cacheados e a
  peneira de postings de O2.1; `index.docs.iter().find` → mapa.
- **Aceite:** `doctor`/`compact` com ganho expressivo no A/B (`N=1167`); proptest de proposta
  idêntica (mesmos pares/ordem); golden de `doctor --json` inalterado.

### E15-T05 ☐ O6.1/O6.2 — `sort_unstable` + capacidade
- **Escopo:** `sort_unstable_by` onde o comparador é total (tiebreak por `id`); `with_capacity`/
  `try_reserve` consistentes.
- **Aceite:** ordem determinística preservada; `make check` verde.

### E15-T06 ☐ O2 — postings (peneira) + BM25
- **Escopo:** `retrieval::postings`, `allowed` como máscara, hoisting de `idf`/`norm`; **manter a
  soma doc-major** (ver `O2.1`).
- **Aceite:** ranking/score **byte-idênticos** (golden + proptest de RRF); `ask` −15–30 %.

### E15-T07 ☐ O2.3 — glob sem alocação
- **Escopo:** matcher de glob iterativo/sem matriz DP (ou `globset` em T12, com testes).
- **Aceite:** testes de glob existentes + novos casos; `ask --anchor`/`rewind --files` sem
  regressão de bytes.

### E15-T08 ☐ O4 — normalize/hash/TOON/JSONL
- **Escopo:** `normalize` fast-path ASCII; `body_hash` incremental; `id` sem `format!`; TOON
  `Cow`/escrita direta; JSONL `with_capacity`/`is_sorted`.
- **Aceite:** proptest de `normalize`/`body_hash`/`note_id` e round-trip TOON; goldens TOON/JSONL
  inalterados.

### E15-T09 ☐ O5 — grafo, views e manifest
- **Escopo:** índice reverso de pai/filho; `compute_views`/SCC memoizados; `next_tasks` com
  `impact` pré-computado; `rank_with` com `confirmers` pré-computado.
- **Aceite:** `rewind`/`task list --ready`/`knowledge map` com ganho; saída idêntica.

### E15-T10 ☐ O6 restante
- **Escopo:** `content_terms` via `len()`, `query_terms` sem `String` extra, `logging::init`
  fast-path, `#[cold]`/`#[inline]` seletivos.
- **Aceite:** testes de retrieval/logging verdes; piso fixo medido menor.

### E15-T11 ☐ O1.6 — carga persistida do `.idx/` com validação
- **Escopo:** carregar `retrieval.jsonl`/embeddings quando válido (invalidação barata por
  `schema_version`/mtime), caindo para rebuild; habilita o formato binário de T12.
- **Aceite:** leitor nunca vê índice parcial (E13-T03); `doctor` reconstrói; `rebuild` byte-idêntico.

- **Aceite:** `make check` + `make ci` verdes; `cargo tree` dentro do orçamento justificado.

### E15-T20 ☐ O8 — `kd task` ponta a ponta
- **Escopo:** corpus único por invocação (O8.1); `compute_views`/`impact` uma vez (O8.2);
  `--scope`/`--tag`/`--anchor`/`--ready`/`--blocked` por postings + índice reverso (O8.3);
  `task graph` em uma passada (O8.4); `task close` subindo ancestrais pelo índice reverso (O8.5);
  `show --history` sem recarregar o corpus (O8.6); `new`/`update` com dedup por peneira (O8.7);
  `plan` com TOON `Cow`/capacidade (O8.8); bancada estendida a `graph`/`close`/`--sort impact`
  (O8.9).
- **Depende de:** T02 (corpus), T06 (postings) e T09 (grafo/views).
- **Aceite:** `task list --sort impact`/`graph`/`close` −20–50 % em N≈1 k no A/B; saída
  byte-idêntica; `bench/e2e.rs` cobre os seis subcomandos.

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

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [ ] Bytes idênticos nos goldens (TOON/JSONL/`prime`/`--json`) salvo T15 (intencional).
- [ ] Ganho medido por A/B em todas as tarefas de performance (`bench/ULTIMO.md` atualizado).
- [ ] `doctor`/`drain` de topo, `--audit` e `knowledge digest` inexistentes; auto-drain não os
      dispara; `kd` sozinho = `kd help` (D171); `kd task` otimizado (O8).
- [ ] Toda tarefa com linha na matriz de aceite e `CHANGELOG.md` atualizado.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`.

## Não-objetivos

- `criterion` como gate (E13-T09 / R43).
- `tokio full`/servidor/DB (R16/R43).
- Aliases de retrocompatibilidade da superfície antiga (D14).
- `target-cpu=native`/PGO no artefato distribuído (só medição local).

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
