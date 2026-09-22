# Referência — `agnostic-rag-rlm-tool` (arags)

> Revisão profunda do projeto `refs/agnostic-rag-rlm-tool/` (Rust, 9 crates, ~467 arquivos)
> enquanto **segunda abordagem ao mesmo problema de memória** do knudge. O objetivo é
> extrair, em caráter **preventivo/otimizador**, o que agrega ao knudge **mantendo a
> simplicidade e a localidade** do resultado final (arquivos + binário `kd`, sem servidor).
>
> Complementa `00_panorama.md`, `01_gaps-plan-rs.md`, `03_decisoes-fechadas.md` e `04_embeddings.md`.

---

## 1. Veredicto de uma frase

O arags resolve o mesmo problema por um caminho **oposto** ao do knudge (servidor de dados
SQLite+usearch vs. arquivos), mas **já pagou o preço** de vários bugs e decisões que o knudge
ainda vai enfrentar. O valor não está na infraestrutura (que **não** deve ser copiada), e sim
nos **algoritmos puros** (RRF, decay, confiança), nas **invariantes de consistência** (vetor
derivado nunca diverge do canônico), no **lifecycle imutável + supersede** e no **modelo de
evidência/âncoras por hash** — tudo adaptável a arquivos com pouco ou nenhum custo.

---

## 2. O que o arags é (em profundidade)

**Tese:** RAG on-demand, *agent-agnostic*, **plano de dados puro e LLM-free**. O servidor
(`arags-server`) indexa, busca, guarda memória e serve por gRPC; o cliente (`arags-cli`) só usa
o **LLM local do usuário** em 3 pontos: digest de resposta (`ask`), summarize (`persist`) e
síntese de voluntários RLM. Nada de loop recursivo no servidor.

**Crates:** `cli` (cliente gRPC), `core` (tipos + config 2-escorpo + dispatch), `storage`
(SQLite WAL + FTS5 + usearch HNSW), `search` (BM25 + entidade + vetor + RRF), `embedding`
(chunking + MiniLM-L6-v2 em candle INT8), `memory` (projetos, knowledge, persist wiki, decay,
consolidação, transfer), `llm` (abstração OpenAI/Anthropic/Gemini/Ollama), `proto`, `server`.

**Quatro espaços de conhecimento** (cada um com tabela + FTS + índice vetorial próprio):

| Espaço | Unidade | Origem |
|---|---|---|
| A — chunks | trecho de arquivo | indexação mecânica |
| B — QA-cache | pergunta→resposta | uso (`ask`) |
| C — RLM nodes | arquivo/tema/projeto | sumarização bottom-up |
| D — explorations | mapa relacional | mapas que subagentes já produzem |

**Memória:** `MemoryEngine` = `ProjectManager` + `KnowledgeEngine` + `PersistEngine` +
`TransferEngine` + `ConsolidationEngine` + `HistoryManager`. Persistência wiki markdown com
frontmatter YAML (`.arags/wiki/`), decay de saliência, entidades, supersede.

**Números que importam:** busca ~21 ms; ingestão ~30 s/10k arquivos; heap limitado a 100 MB;
MiniLM-L6-v2 fixo, 384 dims, INT8; RRF k=60; debounce de save vetorial 2 s.

---

## 3. Divergências de fundação — o que **NÃO** copiar

| arags | knudge | Por quê manter a divergência |
|---|---|---|
| SQLite + FTS5 + usearch + gRPC | arquivos + `kd` local | simplicidade e localidade; nada de daemon, porta, auth |
| 9 crates + proto + Docker + multi-user/tokens | 1 binário | superfície mínima |
| Modelo de embedding **fixo** (não configurável) | plugável (`04_embeddings.md`) | o knudge quer trocar modelo/corpus idioma |
| Estado volátil no frontmatter (`salience`, `access_count`, `epoch`) | volátil **fora** do conteúdo (`.idx/`, `eventos/`) | notas limpas, git-diff determinístico, TOON barato |
| 4 espaços vetoriais dedicados | 1 `.idx/embeddings.jsonl` | escala do knudge não justifica múltiplos HNSW |
| Servidor LLM-free mas **obrigatório** | LLM totalmente opcional | localidade real (offline, sem processo) |

> **Regra de extração:** copiar **matemática pura, invariantes e lifecycle**; nunca a
> infraestrutura de armazenamento/serviço.

---

## 4. Extrações — o que incorporar

### A. Retrieval

| # | Achado no arags | Por que importa | Adaptação ao knudge (simples/local) | Prioridade |
|---|---|---|---|---|
| A1 | **RRF** para fundir BM25 + entidade + vetor, `1/(k+rank+1)`, k=60 | funde canais heterogêneos sem calibrar scores; escala/componível | `recall` funde **BM25**, **âncoras** e **vetor** (quando há) por RRF; canais entram como branches e degradam sozinhos | 🔴 alta |
| A2 | **Tie-break determinístico**: score desc, **`id` asc** (bug real: empate caía na ordem aleatória do `HashMap`; duas queries idênticas davam ordens diferentes) | reprodutibilidade é requisito do knudge (agentes comparam execuções) | ordenar sempre por `(score desc, id asc)`; teste de propriedade "queries idênticas ⇒ saída idêntica" | 🔴 alta |
| A3 | **Canal de entidades** determinístico (regex: funções, structs, paths) + FTS própria, como *tier* de recall | o knudge tem `anchors`; hoje é só um campo, não um canal de busca | tornar `anchors` **canal de recall** (match exato/lexical determinístico), sem LLM | 🟠 média |
| A4 | **Degradação graciosa por tier**: entity/vetor/LLM caem para BM25 em erro, logando `warn` | recall nunca deve falhar por um canal | `recall`/`prime` com fallback para estrutural+lexical; `enabled=false`/`pending` nunca quebram | 🔴 alta |
| A5 | **Normalização de similaridade**: cosseno clampado `[0,1]`; L2 → `1/(1+dist)` | evita negativos e scores fora do contrato | fixar `similarity = cosine` clampada a `[0,1]` no `.idx` e documentar | 🟠 média |
| A6 | **Orçamento de tokens por heurística**: `tokens ≈ palavras × 1.3`, mantém os melhores que cabem, trunca o último, ignora sobra < 100 tokens; `chars/4` para limite de contexto | economiza dependência de tokenizador (localidade) e evita estouro | `prime` com orçamento por `palavras × 1.3` (default 4000), truncando o último; **sem** crate de tokenizer | 🔴 alta |
| A7 | **Truncamento de saída** (~20K chars) | impede que uma nota gigante inunde o contexto | teto de bytes/chars por item em `recall`/`prime`, com marcador de truncado | 🟡 baixa |

### B. Lifecycle de memória

| # | Achado no arags | Por que importa | Adaptação ao knudge | Prioridade |
|---|---|---|---|---|
| B1 | **Salience/decay como função pura**: recência (exp, meia-vida 30d) 0.6 + frequência `1−1/(1+n)` 0.3 + idade 0.1, `should_evict` | ranking de retenção testável e configurável | knudge D43/D45: implementar `decay` puro, **derivado** (nunca no frontmatter); pesos/meia-vida em `config.toml` | 🔴 alta |
| B2 | **Retenção por tipo**: pinned/rules indefinido; análises 90→180→evict; buscas 30→90; sessões 30; TTL explícito | políticas de TTL coerentes por `type` | defaults de TTL/decay por `type` (ex.: `meta`/`question` voláteis; `fact`/`decision` duráveis) em config | 🟠 média |
| B3 | **Consolidação** com `deduplicate` (hash, mantém o 1º), `min_confidence` (padrão 0.3) e **`dry_run`** | limpeza sem apagar às cegas | `compact`/`doctor` com `--dry-run`; dedup por `body_hash`; **propor** (não auto-fundir) por padrão | 🔴 alta |
| B4 | **Imutável + supersede** (não muta): `is_active=0`, `superseded_by=novo`, `retired_at`; purga só após janela de retenção | histórico auditável; nada se perde | já alinhado (`status=superseded`, `superseded_by`); adicionar `retired_at` derivado p/ purga e **nunca** hard-delete do conteúdo | 🟠 média |
| B5 | **Exatamente uma revisão ativa por sujeito** (índice único parcial `WHERE is_active=1`) | impede duplicata ativa após merge | garantir 1 nota ativa por identidade (conteúdo/`supersedes`); o resto vira histórico | 🟠 média |
| B6 | **Metadados temporais**: `version` monotônica, `is_active`, `superseded_by`, `epoch` (por projeto), `created_by`, `model` | time-travel e invalidação em massa barata | `revision` já existe; avaliar `created_by`/`model` (proveniência) e `epoch` (opcional, custo) | 🟡 baixa |

### C. Embeddings (liga direto a `04_embeddings.md`, D79/D80)

| # | Achado no arags | Por que importa | Adaptação ao knudge | Prioridade |
|---|---|---|---|---|
| C1 | **Cache de embedding por SHA-256 do texto**, wrapper que pula inferência em hit, lote com hits/misses mistos, falha de cache degrada para pass-through | com IDs endereçados por conteúdo (D01), é quase de graça | `.idx/emb_cache` (hash→vetor); hit não re-infere; falha nunca é fatal | 🔴 alta |
| C2 | **`vector_status` / `status='pending_vector'` + worker de reconcile** — falha de embedding **marca pendente** no canônico em vez de descartar; re-embeda do texto depois | é exatamente o D80 (fila assíncrona/lazy) com nome e forma | estado derivado `indexed\|pending\|stale` por nota; worker drena; **nunca descarta nota** | 🔴 alta |
| C3 | **Ordem de escrita defensável**: "vetor órfão é aceitável (limpo na manutenção), linha sem âncoras **não**" | define o que um crash pode deixar inconsistente | ao gravar nota+índice+evento: gravar o **canônico primeiro**; índice/embedding depois; reconstruível a qualquer momento | 🔴 alta |
| C4 | **Debounce de persistência vetorial** (dirty flag, ≤1 save/2 s, flush forçado no shutdown) | rajadas (10–20 notas) não devem causar 20 rewrites do índice | `.idx` marca *dirty* e faz flush coalescido; `kd` força flush ao sair; cenário "dark notes" fica barato | 🔴 alta |
| C5 | **Lightweight/fallback embedder determinístico** (SHA-256 → vetor normalizado), sem pesos | testes/CI e modo degradado **sem download de modelo** | `provider = "lightweight"` para testes e offline puro; mantém localidade | 🟠 média |
| C6 | **Harness A/B puro** com Recall@k, nDCG@k, MRR sobre corpus+gold, sem tocar servidor | o jeito concreto de decidir L6 vs L12 vs multilíngue | `kd maintenance eval --ab <modeloA> <modeloB>` sobre um golden pequeno; métricas puras testáveis | 🟠 média |
| C7 | **Quantização INT8** default e `matryoshka_truncate` p/ dims | velocidade/memória sem perder muito | opção de quantização e truncamento de dims no índice (default f32 384) | 🟡 baixa |

### D. Evidência, staleness e confiança (o coração do "projeto vivo")

| # | Achado no arags | Por que importa | Adaptação ao knudge | Prioridade |
|---|---|---|---|---|
| D1 | **Âncoras com `content_hash` + verify-on-hit**: no hit, recheca o hash vigente dos arquivos citados; `stale_reason` granular (`cited` invalida, `context` não) | mede **verdade**, não só similaridade | `anchors` carregam `path`+hash; `kd maintenance doctor` recheca sob demanda; `cited` vs `context`; **nunca** apaga stale — sinaliza | 🔴 alta |
| D2 | **Confiança composta** `sim × drift_factor × age_factor + feedback_weight × feedback`, com pisos (nunca zera); propriedades (monotônica em sim/confirmed; decrescente em drift/idade) | ranking honesto, testável por proptest | **derivada em tempo de consulta** (não armazenar) a partir de evidência+feedback+idade+âncoras | 🟠 média |
| D3 | **Feedback confirm/contradict**; N contradições → auto-stale → review | fecha o ciclo de correção sem LLM obrigatório | `outcomes[]` com confirm/contradict; limiar leva a `stale`/fila de revisão | 🟠 média |
| D4 | **`cache_id` estável (UUIDv7) anti-drift**: o orquestrador passa o ID ao subagente e este recebe **1:1** o mesmo contexto, sem re-busca/re-síntese | handoff reprodutível entre agentes/rodadas | `rewind` emite um **`context_id`** (endereçado pelo conjunto retornado); `kd rewind --resume <id>` devolve idêntico — encaixa no D57 | 🟠 média |
| D5 | **Invalidação por hash das fontes**: cache fica `stale` quando qualquer chunk de origem muda | evidência não pode mentir após mudança | se `evidence`/`anchors` referenciam notas, mudança nelas invalida o derivado; recheck por hash | 🔴 alta |
| D6 | **Margens duplas** `hit_high`/`hit_low` → `strong`/`related`/`none` | evita surfacar match fraco como forte | `recall` retorna grau (forte/relacionado) além do score; default conservador | 🟡 baixa |
| D7 | **Assimetria declarada**: falso-positivo custa mais que falso-negativo ⇒ *precision > recall* | define a política quando em dúvida | documentar como princípio de retrieval/validação do knudge | 🟠 média |

### E. Configuração e processo

| # | Achado no arags | Por que importa | Adaptação ao knudge | Prioridade |
|---|---|---|---|---|
| E1 | **Config 2-escorpo** global+local, merge campo a campo; **`[auth]` só no global** (local nunca carrega credenciais); local gitignored | segurança e precedência previsível | knudge D61 já é global+projeto; adotar **segredos só no global** e projeto sem credenciais | 🟠 média |
| E2 | **Nome canônico lógico de projeto** (não caminho): worktrees do mesmo repo compartilham identidade; rejeita `.`/`..`/caminho absoluto | resolve o D29/D30 (worktree) com semântica, não com path | identificar projeto por **nome lógico** em `.knudge/`, worktrees compartilham `.knudge/` | 🟠 média |
| E3 | **Migrações versionadas** (`schema_version`) + `ANALYZE`; rebuild documentado quando dims/modelo mudam | evolução sem corromper | manter `schema_version` no `.idx/meta`; rebuild documentado (já previsto em D17/D79) | 🟡 baixa |
| E4 | **Disciplina de engenharia**: arquivos ≤300 linhas, proibido `unwrap/expect/panic` em `src`, clippy pedantic `-D warnings`, testes fora do arquivo, **proptest** para RRF/chunking/confiança | robustez do binário `kd` | espelhar no core Rust: puros testados por proptest (RRF, decay, confiança) | 🟠 média |
| E5 | **`ScopedTimer` + logs estruturados com `elapsed_ms`; audit log de invalidações** (`invalidated_by/reason`) | observabilidade e trilha de auditoria | `eventos/` registra supersede/invalidação com ator e motivo (`elapsed_ms` em hot paths) | 🟠 média |
| E6 | **Sanitização de query** para FTS + SQL 100% parametrizado | evita injeção/quebra de busca | se houver FTS/SQLite, `sanitize_fts`; no BM25 próprio, escapar tokens especiais | 🟡 baixa |

### F. Invariantes que previnem bugs caros (resumo executável)

1. **Vetor é derivado e descartável; a nota é a verdade.** Toda remoção (supersede, merge, compact, TTL, dedup) **purga o vetor**. Caso contrário surge divergência de contagem e rebuild total (bug `fa25` do arags).
2. **Falha de embedding = `pending`, nunca perda.** (D80 / C2)
3. **Canônico antes do derivado.** Crash deixa no pior caso vetor órfão, nunca nota ausente. (C3)
4. **Determinismo de ordenação.** Empate resolve por `id`; senão duas execuções idênticas divergem. (A2)
5. **`recall` nunca falha** por canal opcional ausente/falho. (A4)
6. **Stale não se apaga; sinaliza.** Histórico é auditável. (D1/D4/B4)
7. **Nada de estado volátil no conteúdo.** Mantém o git limpo e o TOON barato. (B1/B6)
8. **Escrita em rajada é coalescida** (debounce) e nunca bloqueia. (C4/D80)

---

## 5. Preventivos — armadilhas que o arags documentou

| Armadilha real no arags | Lição preventiva para o knudge |
|---|---|
| **`fa25`**: consolidação removia chunks em SQLite mas **não** os vetores do usearch → divergência de contagem → rebuild completo no bootstrap | uma única função de remoção que **sempre** toca canônico **e** derivado; `doctor` detecta divergência |
| **`pending_vector`** criado porque falhas eram **descartadas em silêncio** | nunca “esquecer” uma nota; estado pendente explícito + reconcile |
| **RRF não-determinístico** (ordem de `HashMap`) | sort total com chave de desempate estável |
| **`SQLITE_BUSY_SNAPSHOT`** (read→write promotion sob pool) | no knudge (arquivos): lock de escrita único e curto; leitura nunca promove a escrita |
| **Deadlock de conexão** (re-lock do mesmo mutex não-reentrante) | evitar reentrância de lock; preferir snapshots |
| **Ollama sem connect timeout** travava o bootstrap | todo I/O externo (embedding `http`) com **timeout curto** e falha rápida → cai para `pending`/lexical |
| **Cache podre após refatoração** (QA-cache sem invalidação) | evidência/derivados invalidam por **hash da fonte** |
| **Falso-positivo "login" vs "logout"** no cache semântico | checagem secundária (Jaccard/proveniência) além do cosseno |
| **`estimate_incremental_cost` planejado e nunca chamado** | evitar código morto de otimização; simplicidade |
| **Summary duplicava** por `INSERT` sem `ON CONFLICT` | operações derivadas devem ser **idempotentes** (upsert por chave) |

---

## 6. Decisões fechadas (D81–D92) ✅

> **Aprovadas como recomendado**, com **simplicidade como meta global**. Registradas em `03_decisoes-fechadas.md` (seção Q) e propagadas em `00_panorama.md`/`04_embeddings.md`. As descrições abaixo permanecem como racional da extração.

| ID | Decisão | Origem |
|---|---|---|---|
| **D81** | `recall` funde canais por **RRF** com tie-break `(score desc, id asc)` e degradação graciosa | A1/A2/A4 | ✅ |
| **D82** | Orçamento de tokens do `prime` por heurística **sem tokenizer** (`ceil(len/4)`, D40) | A6 | ✅ |
| **D83** | **Cache de embedding por `body_hash`** + estado `indexed\|pending\|stale`, reconcile e **nunca descartar nota** | C1/C2 | ✅ |
| **D84** | **Purga do vetor em toda remoção** + `doctor` que detecta divergência canônico↔derivado | F1/fa25 | ✅ |
| **D85** | **Flush coalescido (debounce)** do `.idx` e do arquivo de embeddings | C4 | ✅ |
| **D86** | `anchors` com **`path` + `content_hash`** e **verify-on-hit** (`cited` invalida, `context` não); stale **sinaliza**, não apaga | D1/D5 | ✅ |
| **D87** | **Confiança derivada** em tempo de consulta (não armazenada), separada de `confidence` declarada | D2 | ✅ |
| **D88** | `rewind` emite **`context_id`** endereçável para handoff 1:1 (`kd rewind --resume`) | D4 | ✅ |
| **D89** | **`provider = "lightweight"`** (embedder determinístico) para testes/offline | C5 | ✅ |
| **D90** | **`kd maintenance eval --ab`** com Recall@k/nDCG@k/MRR sobre golden | C6 | ✅ |
| **D91** | Projeto identificado por **nome lógico**; segredos **só no global** | E1/E2 | ✅ |
| **D92** | **Disciplina Rust**: ≤300 linhas, sem `unwrap/expect/panic`, proptest nos puros | E4 | ✅ |

---

## 7. Sequência recomendada

1. **Fundação de retrieval** — D81, D82 (determinismo + orçamento).
2. **Derivados confiáveis** — D83, D84, D85 (embedding/quebra de índice).
3. **Evidência viva** — D86, D87 (âncoras hash + confiança derivada).
4. **Handoff e avaliação** — D88, D90.
5. **Localidade e engenharia** — D89, D91, D92.

Tudo o acima é **aditivo** ao panorama e não exige servidor, banco, nem processo de fundo
obrigatório — cabe no binário `kd` e no `.knudge/`.

---

## 8. Em uma frase

Do arags, o knudge deve herdar os **algoritmos e as invariantes** (RRF determinístico, decay
puro, confiança composta, lifecycle imutável+supersede, vetor sempre derivado e reconciliável,
âncoras por hash) e **recusar a infraestrutura** (servidor, SQLite, usearch, múltiplos espaços,
modelo fixo) — trocando-a por arquivos, um modelo plugável com default pinado e um binário único.
