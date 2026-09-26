# Otimizações de performance — bateria proposta

> **Status:** proposta (não implementada). Baseline medido em
> [`bench/RELATORIO.md`](../../bench/RELATORIO.md) / [`bench/ULTIMO.md`](../../bench/ULTIMO.md).
> Alvo: reduzir o custo por comando **sem mudar bytes nem semântica** (goldens/proptest
> idênticos), respeitando as restrições do `AGENTS.md` (dep nova **só com ganho expressivo e
> justificada** — R43; sem `unwrap/expect/panic/unsafe`; `src/` ≤ 300 linhas;
> `BTreeMap`/`IndexMap` para determinismo; sem indexação/slicing direto).

## 0. Princípios aplicados (`.agents/skill/rust/`)

| Princípio (cap.) | Aplicação |
|---|---|
| Abstrações zero-custo (08) | pipelines de iterador e `fold` compilam igual a loop manual; usar onde legível |
| Pré-alocar (05, 16) | `Vec::with_capacity`/`try_reserve`, `String::with_capacity`, reuso de buffer |
| `sort_unstable` sem estabilidade (05) | onde o comparador termina em `id` (ordem total) |
| `Cow`/`&str` sobre `String` (05, 10) | evitar alocação por linha/token/campo no parse |
| Evitar `format!`/`join` em loop quente (05) | `write!`/`push_str` em buffer já dimensionado |
| Layout cache-friendly, menos alocação por nó (16) | índices planos (`Vec<u32>`), sem `Vec` por entrada |
| `#[inline]`/`#[cold]` (02) | selectivo em hot path / caminhos de erro |
| Sem trait objects no quente (07, 16) | genéricos/monomorfização onde há closure por doc |

Cada item abaixo é **independente** e verificável: bancada antes/depois + `make check`.

---

## Onda 1 — leitura única do corpus (maior ganho, risco baixo)

Hoje cada comando relê e reparseia o corpus várias vezes. `kd rewind` (`N=1167`) faz **4–5
passes**: `session.index()`, `session.graph()`, `load_notes()`, `embedder::pending()` e o
auto-drain. `Index::from_store` e `Graph::build` cada um faz `store.list_ids()` + `read_optional`.

- **O1.1 — `store::corpus` de uma passada.** Novo helper que lê `Vec<Note>` uma vez e deriva
  índice + grafo + corpos do mesmo vetor:

  ```rust
  // crates/knudge-core/src/store/corpus.rs
  pub struct Corpus { pub notes: Vec<Note>, pub index: Index, pub graph: Graph }
  impl Corpus {
      pub fn load(store: &Store<'_>) -> Result<Self> {
          let ids = store.list_ids()?;
          let mut notes = Vec::new();
          let _ = notes.try_reserve(ids.len());
          for id in &ids {
              if let Some(note) = store.read_optional(id)? { notes.push(note); }
          }
          let index = Index::build(&notes)?;      // antes: 2ª leitura
          let graph = Graph::from_notes_ref(&notes)?; // novo, empresta (sem clone)
          Ok(Self { notes, index, graph })
      }
  }
  ```

- **O1.2 — `Graph::from_notes_ref(&[Note])`** (o atual consome e forçaria `notes.clone()`).
  Mesmo resultado; só evita copiar o corpus.
- **O1.3 — `embedder::pending`/`drain` recebem `&[Note]`** já carregado em vez de `store.read`
  de novo (o auto-drain reusa o corpus carregado quando existir).
- **O1.4 — `Session::corpus()`** e migrar `ask`/`rewind`/`doctor`/`learn`/`compact`/`prune`/**`task`**
  para ele. `Session::index()`/`graph()` viram wrappers de conveniência (mantidos para testes).
- **O1.5 — auto-drain:** checar `KNUDGE_NO_IDLE` **antes** de `Session::open()`; não rodar em
  `prime`/`self` e **nos verbos de manutenção** (`doctor`/`drain`, quando promovidos); pular
  quando `embeddings.enabled=false` ou `provider=none` (hoje `Session::open`
  e, com `http`, uma varredura O(N) acontecem mesmo em `self version`).

**Ganho esperado:** `rewind` 362 ms → ordem de 100–140 ms; `ask` −25–35 %; `prime`/`self version`
−65 % (≈ −35 ms). **Risco:** baixo (mesmos dados, uma vez).

---

## Onda 2 — retrieval: índice invertido e globs

- **O2.1 — postings derivados (como **peneira**, não como ordem de soma).** `Index` ganha um índice
  invertido **em memória** (`termo+field → posições`), reconstruído no `build` e nunca
  persistido/contrato. `score`/`score_with` deixam de varrer **documentos sem overlap** e passam a
  iterar só os candidatos que compartilham ≥1 termo de conteúdo, mantendo a **ordem de soma
  atual** (doc-major, field-major, term-major):

  ```rust
  // crates/knudge-core/src/retrieval/postings.rs
  pub struct Postings { /* Field -> term -> Vec<u32> (posições em Index::docs, ordenadas) */ }
  impl Postings {
      pub fn build(index: &Index) -> Self;
      /// posições de docs que compartilham ao menos um termo (ordenadas, únicas)
      pub fn sieve(&self, terms: &[Cow<'_, str>]) -> Vec<u32>;
  }
  ```

  O `allowed` (hoje `BTreeSet<String>`, O(log N) com comparação de string por doc) vira uma
  **máscara `Vec<bool>` por posição**, montada uma vez. Fórmula, pesos, IDF e desempate
  `(score desc, id asc)` **inalterados**.

  > **Por que não acumular term-major** (uma passada sobre postings somando em cada doc): isso
  > muda a **ordem das somas em `f64`** e, com ela, o último bit do score — e o `score` é
  > serializado no `--json`. Rejeitado para manter bytes idênticos. A peneira preserva a soma
  > doc-major exata de hoje.

  **Ganho:** `ask` deixa de pontuar docs sem overlap (ganho cresce com vocabulário seletivo);
  prepara O3. **Risco:** médio, com proptest/golden de ranking cobrindo.

- **O2.2 — hoisting no BM25.** Em `field_sum`, `length`, `avg`, `norm` e `weight` são recalculados
  **por termo**; `doc.tf` + `idf` fazem duas buscas em `BTreeMap` por termo. Hoist para fora do
  loop de termos e acumular direto. **Ganho:** ~20–35 % do custo lexical. **Risco:** nulo.

- **O2.3 — `glob_match` sem alocação.** Hoje aloca uma matriz DP `Vec<Vec<bool>>` por par (padrão,
  texto) e `glob_tokens` realoca o padrão a cada chamada. Trocar por: (a) `GlobPattern` compilado
  uma vez; (b) matcher linear com backtracking para `*`/`?` (O(1) de espaço) e DP só quando há
  `**`; (c) reuso em `filter::anchor_matches`, `anchor::rank`, `manifest`. **Ganho:** significativo
  em `ask --anchor`, `rewind --files`, `doctor` (âncoras). **Risco:** médio; travar com os testes
  de glob existentes + novos casos.

- **O2.4 — `fuse` (opcional, baixa prioridade).** `Accum.contribs: Vec<f64>` aloca uma vez por id;
  `Fused.id` clona a `String` do hit. Reduzir exigiria mudar o tipo público `Vec<f64>`/`String`
  (contrato interno), então fica como item **opcional** de menor prioridade. Medido: ~180 µs para
  3×200 — relevante apenas se os canais crescerem.

---

## Onda 3 — dedup: matar o O(N²) de `doctor`/`compact`

> **Nota (revisão):** `doctor` é **raro** e o custo da validação completa é aceito
> (consistência/garantia/resolubilidade). O3 beneficia `doctor` e, sobretudo, `compact`/`learn`
> — mas **não é pré-requisito** de nenhum verbo; é melhoria, não gate.

- **O3.1 — lookup O(1).** Em `propose_merges`, o `index.docs.iter().find(...)` por hit é O(N);
  trocar por `BTreeMap<&str, &NoteDoc>` construído uma vez. **Risco:** nulo.
- **O3.2 — `term_set` cacheado.** `dice` reconstrói `BTreeSet<&str>` dos dois docs a cada par;
  pré-computar por doc uma vez.
- **O3.3 — scoring por peneira (O2.1) + `terms` cacheados.** Com a peneira, o laço externo para
  de pontuar docs sem overlap. Além disso, `index.score(&doc.statement, …)` **re-tokeniza o
  `statement` de cada doc a cada chamada**; pré-computar os `terms` de cada doc uma vez (ou usar
  o próprio `doc.fields[Statement]`) remove N tokenizações.

  > **Limite honesto:** o top-10 por BM25 é exato; a peneira remove só os scores **zero**. Em
  > vocabulário denso, o pior caso segue quadrático — eliminá-lo exigiria mudar a semântica de
  > seleção de candidatos (fora do escopo "sem mudar comportamento"). A meta é **reduzir o
  > constante e pular o zero-overlap**, não prometer sub-quadrático.

  **Ganho esperado:** `doctor` de 2,46 s para a faixa de centenas de ms (o teto de `learn`, que
  limita a 64 docs, é 88 ms). **Risco:** médio; `learn` serve de referência de teto.
- **O3.4 — `write` não reconstrói a nota.** `write` faz `draft.to_note()` e `propose` faz
  `draft.to_note(0)` de novo (normalize/hash duas vezes). Passar a `Note` já construída para uma
  variante interna de `propose`.

**Ganho esperado:** `doctor` 2,46 s → dezenas de ms em 1 k notas (o ganho mais dramático).
**Risco:** médio; `learn` já limita e serve de referência de teto.

---

## Onda 4 — escrita/parse: `normalize`, hash, TOON, JSONL

- **O4.1 — `normalize` com fast-path ASCII.** `input.nfc().collect()` aloca e normaliza mesmo
  quando a entrada é ASCII (maioria). Se `input.is_ascii()`, pular NFC (identidade em ASCII) e só
  trim/colapsar, com `String::with_capacity(input.len())`. Adicionar `normalize_into(&mut String)`
  para reuso de buffer. **Ganho:** ~3,3 µs → <1 µs por statement/corpo ASCII; impacta `note_id`,
  `body_hash` e todo `write`. **Risco:** baixo (NFC de ASCII é identidade; travar com proptest).
- **O4.2 — `body_hash` incremental.** Fazer `Sha256::update` em três pedaços (statement, `\n`,
  corpo) sem concatenar `String`. **Risco:** nulo (mesmos bytes).
- **O4.3 — `id` sem `format!`.** `note_id`/`hex8`/`base36_8` com buffer (`[u8; 8]` em `base36_8`,
  `write!` em `String::with_capacity(8)`). **Risco:** nulo.
- **O4.4 — TOON `Line.text: Cow<'_, str>`.** `split_lines`/`strip_comment` alocam `String` por
  linha (≈2 alocações × N linhas por nota). Devolver `Cow::Borrowed` quando não há comentário/
  escape. `Vec::with_capacity` na lista de linhas. **Ganho:** direto em `Note::parse` (6,6 µs/nota).
- **O4.5 — emissor escreve direto.** `emit_flow` monta `Vec<String>` + `join`; escrever no `&mut
  String` com `push_str`/`write!`. `is_plain` evita dois `parse::<f64>` com scan manual.
- **O4.6 — `flow::split_key`/`split_top_level`** devolvem `&str`/`Cow` em vez de `String`.
- **O4.7 — JSONL** `encode` com `with_capacity`; pular `keys.sort()` quando `is_sorted()`;
  `encode_str` reserva. `Note::render` com `with_capacity(front+body+8)`.

**Ganho:** `Note::parse` e `write` −20–40 % no caminho quente. **Risco:** baixo; TOON/JSON têm
golden + proptest de round-trip.

---

## Onda 5 — grafo, views, manifest

- **O5.1 — índice reverso de pai.** `Graph::parent`/`has_parent` fazem varredura reversa O(V+E);
  `scope_of`/`hierarchy_scope`/`boundaries` chamam `parent` por doc. Construir no `from_notes` um
  `BTreeMap<String, String>` (filho→pai, `or_insert` na ordem do `BTreeMap` = mesma semântica "1º
  vence") e `children` se necessário. `parent`/`has_parent` O(1); `scope_of` O(profundidade).
  **Ganho:** `knowledge map`, `rewind`, `task graph`, `doctor` (integridade). **Risco:** baixo.
- **O5.2 — `compute_views` memoizado.** `compute_views` roda SCC (`dependency_cycles`) + DFS por
  tarefa; `manifest_at` chama `next_tasks` **e** `manifest_text`, cada um chamando `compute_views`
  → 2× por `rewind`. Computar uma vez e passar adiante. `block_reason` recomputa o SCC por id
  (O(N) por tarefa em `--explain`) — memoizar. **Risco:** baixo.
- **O5.3 — `next_tasks` sem `impact` no comparador.** `impact(graph, id)` (travessia) é chamado
  dentro do `sort_by`; pré-computar `(impact, created, id)` uma vez e ordenar. `sort_unstable_by`.
- **O5.4 — `rank_with`** pré-computa `confirmers` e o mapa de âncoras do working set.

**Ganho:** `rewind`/`task list --ready`/`knowledge map` −20–40 %. **Risco:** baixo.

---

## Onda 6 — consistência transversal (micro, semântica intacta)

- **O6.1 — `sort_by` → `sort_unstable_by`** onde o comparador é **total** (tiebreak por `id`):
  `bm25`, `rrf`, `anchor::rank`, `merges`, `manifest`, `doctor` etc. A stable sort aloca; a
  unstable não. Determinismo preservado (ordem total).
- **O6.2 — `with_capacity`/`try_reserve`** de forma consistente em `build_hits`, `candidates`,
  `views`, `cycles`, `clusters`, `list_ids`.
- **O6.3 — `content_terms`:** `term.chars().count() >= 2` → `term.len() >= 2` (tokens são ASCII por
  construção, D36) — evita varredura de chars por termo.
- **O6.4 — `query_terms`:** evitar `String` por token no `BTreeSet<String>` de dedup; dedup com
  comparação emprestada (`&str`) — consultas são curtas.
- **O6.5 — `logging::init`:** não construir o subscriber quando `level == "off"`/`--quiet`;
  caminho rápido sem `EnvFilter` para nível simples. Remove ~alguns ms do piso fixo.
- **O6.6 — `#[cold]`** em construtores de erro quentes (`Error::invalid_input` em parse) e
  `#[inline]` em `is_word`/`tf`/`len`/`type_weight`.

---

## Onda 7 — dependências com ganho expressivo (permitidas)

Regra de adoção (cada item exige **A/B na bancada**):

1. Declarar em `[workspace.dependencies]` e consumir com `.workspace = true`;
   `default-features = false` quando der.
2. `cargo deny`/`audit`/`machete` verdes; licença na allowlist do `deny.toml`.
3. Partir de **≥ 20 %** de ganho ponta-a-ponta no alvo (ou remover complexidade
   significativa sem regressão) e **nunca** alterar bytes de saída.
4. Determinismo: hash map **nunca** é iterado para produzir saída (sempre `sort`/ordem por
   posição); `BTreeMap` continua onde a ordem importa.
5. Nada que introduza I/O ou async no `knudge-core`; `rayon`/maps/parsers são puros.

| Crate | Onde | Ganho esperado | Licença | Risco |
|---|---|---|---|---|
| **`rayon`** | ler+parsear corpus (O1) e varreduras independentes (`doctor`, `propose_merges`, `checks`) | **3–8×** na parte paralelizável (12 threads); `rewind`/`ask`/`doctor` −30–60 % | MIT/Apache | médio (determinismo; coletar por posição e ordenar por id) |
| **`rustc-hash`** (`FxHashMap/FxHashSet`) **+ `hashbrown`** | `allowed`/`by_id`/postings/dedup/grafo | O(1) + hashing rápido; `score`/`propose_merges` −20–35 % | Apache/MIT | médio (não iterar p/ saída; ajustar `disallowed_types`) |
| **`smallvec`** | `contribs`, arestas por nó, buckets de postings, candidatos/termos | menos alocações no parse/retrieval (−10–25 % onde há muitos `Vec` curtos) | MIT/Apache | baixo |
| **`compact_str`** | ids/chaves curtas em `Index`/`Graph`/postings | ids inline (≤24 B) sem heap; menos alocações em corpus grande | MIT | médio (tipos internos; manter `Deref<Target=str>`) |
| **`memchr`** | `split_lines`/`strip_comment`, lexer TOON, parser JSON, tokenizer, `normalize` | scan de bytes SIMD; parse −20–40 % | Unlicense/MIT | baixo |
| **`globset`** | substitui o `glob_match` DP custom; compila padrões do working set 1× | `ask --anchor`/`rewind --files` **−50 %+** | Unlicense/MIT | médio (semântica `?`/`*`/`**`; travar com testes de glob) |
| **`mimalloc`** (allocator global) | `knudge-cli`/`knudge-mcp` (não no core) | **5–15 %** transversal em carga “alocativa” (1 linha) | MIT | baixo (binário maior) |
| **`bincode`/`postcard`/`rkyv`** | formato **binário** do `.idx/` derivado (retrieval/embeddings/cache) e load validado (O1.6) | `Index::serialize+parse` 22 ms → 2–5 ms; evita rebuild a cada comando | MIT/MIT+Apache | baixo (`.idx/` é derivado, reconstruível; não é contrato TOON) |
| **`itoa`/`ryu`** | emissor de inteiros/floats | formatação mais rápida no TOON/JSON | MIT/Apache | baixo para `itoa`; **validar** `ryu` vs `Display` (pode mudar float) |
| `ahash` | alternativa ao `rustc-hash` | hashing rápido com hardening | MIT/Apache | baixo (escolher **um** hasher) |
| `simd-json`/`sonic-rs` | decodificação JSON SIMD do `write --batch` | alto no lote | Apache/MIT | **alto**: aceita `null`/duplicatas/números além do contrato → só se isolado na entrada com validação estrita depois |

Notas de implementação:

- **`rayon` primeiro, isolado no adaptador.** A leitura de N arquivos é o gargalo do `rewind`
  (O1). Alternativa **zero-dep**: `std::thread::scope` com chunks fixos — menos ganho, sem dep.
  Em ambos, o determinismo vem de coletar `(posição, Note)` e reordenar por `id` (ou por
  `list_ids`), nunca da ordem de conclusão.
- **Hasher determinístico.** `FxHashMap`/`hashbrown` só substituem `BTreeMap`/`BTreeSet` internos
  onde **não há iteração para saída**. Onde há (postings, propostas), manter `BTreeMap`/`Vec`
  ordenado. Ajustar `clippy.toml`/`disallowed_types` com `reason` se necessário.
- **Persistência binária** só depois de O1.6 (load com validação): sem isso, um `.idx/` mais
  rápido não é exercitado em cada comando. Manter a versão/`schema_version` no cabeçalho e
  cair para o JSONL canônico se a versão não bater.
- **Build flags** (não é dep): `target-cpu=native` no perfil local e **PGO** (`cargo-pgo`) são
  ganhos reais em parsing/hash, mas quebram portabilidade do artefato distribuído — medir só
  na bancada local.

---

## Onda 8 — `kd task` ponta a ponta

`task` é o verbo mais composto: lê o corpus, monta grafo/views, filtra subárvores e atualiza
progresso de ancestrais. Hoje cada subcomando refaz esse trabalho. Otimizar **tudo que o envolve**,
sem mudar bytes:

- **O8.1 — corpus único por invocação.** Todos os subcomandos usam `Session::corpus()` (O1.1);
  `task list`/`show`/`graph`/`close`/`update` deixam de chamar `index()`+`graph()`+`load_notes()`
  separadamente.
- **O8.2 — views/impact uma vez.** `compute_views`/SCC e `impact` pré-computados por invocação
  (O5.2/O5.3); `--sort impact` e `--ready`/`--blocked` reusam o mesmo cálculo.
- **O8.3 — filtros por postings + índice reverso.** `--scope` (subárvore `results_in`), `--tag`,
  `--anchor`, `--type`, `--status` usam a peneira de postings (O2.1) e o índice reverso de pai
  (O5.1), não uma varredura por filtro.
- **O8.4 — `task graph` em uma passada.** Reusa o índice reverso (O5.1) e acumula `done/total` sem
  recompor a floresta por programa.
- **O8.5 — `task close`.** O progresso dos ancestrais sobe pelo índice reverso (O5.1) numa única
  caminhada; não reconstrói grafo nem recomputa views por ancestral.
- **O8.6 — `task show --history`.** Eventos lidos uma vez (tail) e ids resolvidos por caminho
  conhecido, sem recarregar o corpus inteiro.
- **O8.7 — `task new`/`update`.** Reusam o corpus/índice carregado; dedup do enunciado pela peneira
  de O3 (não O(N) por nota).
- **O8.8 — `task plan --prompt/--submit`.** Cabeçalhos TOON com `Cow`/escrita direta (O4.3) e
  capacidade pré-alocada no parse (O4.4).
- **O8.9 — bancada.** Estender `bench/e2e.rs` com `task new`, `task list --universe`,
  `task list --sort impact`, `task graph`, `task show` e `task close` em N=200/1000 — hoje só
  `task list` é medido; `graph`/`close`/`--sort impact` podem esconder gargalos.

**Ganho:** `task list --sort impact`/`task graph`/`task close` −20–50 % em N≈1 k. **Risco:** baixo
(reusa O1/O2/O5, sem contrato novo).

---

## Matriz de impacto × risco

| Onda | Onde | Ganho esperado (N≈1 k) | Risco |
|---|---|---|---|
| O1 | `rewind`, `ask`, auto-drain | `rewind` −60 %, `prime` −65 % | baixo |
| O2 | `bm25`, `rrf`, `anchor` | `ask` −15–30 %, `--anchor` −50 % | médio |
| O3 | `merges`, `doctor` | `doctor` 2,46 s → centenas de ms | médio |
| O4 | `normalize`, TOON, JSONL | `write`/`parse` −20–40 % | baixo |
| O5 | `Graph`, `views`, `manifest` | `rewind`/`map` −20–40 % | baixo |
| O6 | transversal | −5–15 % + consistência | baixo |
| O7 | deps (`rayon`,`hashbrown`,`memchr`,`globset`,`mimalloc`…) | −20–60 % por alvo, conforme gate | variável |
| O8 | `task` (list/show/graph/close/plan) | `task list --sort impact`/`graph`/`close` −20–50 % | baixo |

> As ondas se **somam** onde não disputam o mesmo trecho (O1+O3+O5 atacam córregos distintos do
> `rewind`; O2 alimenta O3).

## Guardrails (cada item)

1. `make check` verde (fmt + clippy `-D warnings` + test + gate 300 linhas).
2. Sem `unwrap/expect/panic/unsafe`; dep nova **só** pela Onda 7 (A/B + licença +
   `[workspace.dependencies]`); sem indexação sem `#[allow(reason)]`.
3. **Goldens byte-idênticos** (`prime`, `--json`, TOON); **proptest** de round-trip TOON, RRF
   (determinismo/monotonicidade/união) e `normalize`/`body_hash`/`note_id`.
4. Bancada A/B (`make bench`) registrando o delta; se um item não mover o ponteiro, é revertido.
5. Se a mudança tocar uma **borda** (Unicode, ordem, hash), linha em `DIVERGENCES.md`; se mudar
   contrato/decisão, `Dxx` novo (nenhum item aqui muda TOON/schema/exit codes).

## Ordem de execução recomendada

1. **O6.1** (`sort_unstable`) e **O1** — baixo risco, ganho grande e imediato.
2. **O2.2/O4** — micro-otimizações locais, isoladas.
3. **O2.1 + O3** — o pacote algorítmico (postings + dedup), validado por benchmark e proptest.
4. **O2.3/O5/O8** — globs, grafo/views e `task` ponta a ponta.
5. **O6 restante + O1.5** — polimento e piso fixo.
6. **O7 por último e uma por vez** — `memchr`/`smallvec`/`rustc-hash` (baixo risco) → `globset`
   → `rayon` → `mimalloc` → `bincode`/`rkyv`; cada uma com A/B e remoção imediata se não pagar.

Cada passo: PR pequeno, `make bench` antes/depois, `make check` verde.
