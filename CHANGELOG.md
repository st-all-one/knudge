# Changelog

Todas as mudanças relevantes do knudge. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Não publicado]

### Adicionado
- **AGENTS.md** — guia de contribuição do repositório: padrões de desenvolvimento, erros,
  logs, testes, contrato de bytes e checklist de conclusão.
- **E10 — Ciclo de vida, decay e clusters** (Fase 2, concluído):
  - `lifecycle::shelf_life`: TTL por `classification` — `foundational` nunca expira,
    `tactical`/`observational` com prazos configuráveis; `expires_at` explícito vence o
    derivado (D44).
  - `lifecycle::decay`: validade de âncoras (literal existe / glob casa) com varredura do
    projeto limitada; demolição após grace se a fração válida < threshold (D43).
  - `lifecycle::retire`: `retired_at` **derivado** dos eventos (`forget`/`supersede`); purga
    do conteúdo (nota + derivado) só após a janela de retenção (D48/D84).
  - `lifecycle::supersession`: demolição soft protegida — membros de ciclo de
    supersessão/dependência **não** demovem (D45).
  - `lifecycle::plan`: plano puro de demolição combinando shelf-life × decay × ciclos.
  - `lifecycle::clusters`: fase 1 estrutural determinística (`anchor`/`type`/`classification`/
    container) e fase 2 semântica **opcional e off-path**, com similaridade injetada (E11).
  - `not_before` como **28ª chave canônica** (D100): agendamento ortogonal à expiração;
    `compute_views_at` o considera, `compute_views` (prime) não (D56/D57).
  - Correções de contrato: `Classification` ganha `Ord`; `compute_views_at` exportado.
- **E09 — Validação, saúde e leitura tolerante** (Fase 2, concluído):
  - `health::validator`: catálogo `.knudge/validators.toml` (subset TOML — D99) e resolução
    `checks = explícitos ∪ globais ∪ por_âncora`; explícito ausente vira `missing[]` (D54).
  - `health::evidence`: fechamento por **evidência** — grava `evidence` + `outcomes[]`
    (`status/duration/agent/notes/recorded_at`) e **infere** o `outcome` pela severidade
    (`success`/`partial`/`failure`); sem evidência, não fecha (D48/D55).
  - `health::audit`: relatório puro de integridade, ciclos, âncoras quebradas, duplicatas,
    arestas sugeridas faltantes e locks stale (D46).
  - `health::doctor [--fix]`: 10 checks (schema, integridade, ciclos, âncoras, duplicatas,
    locks, config, `body_hash`, eventos, divergência canônico↔derivado) e reparo reversível
    **idempotente** (D19/D84).
  - `health::tolerant`: leitura Postel — chave desconhecida → warning; `type` desconhecido ou
    nota malformada → **skip + orientação**, sem derrubar o comando (D16–D18); `Config::strict`.
  - `health::anchors`: `content_hash` derivado em `.idx/anchors.jsonl` e verify-on-hit —
    `cited` invalida, `context` não; stale **sinaliza**, nunca apaga (D86).
  - `lifecycle::confidence`: confiança **derivada** (`sim × drift × idade + feedback`, pisos,
    `[0,1]`) com proptest de monotonicidade; exposta em `RecallHit.confidence` (D87).
  - Correções de contrato: `task::outcome` usa `notes`/`recorded_at` (D48) e `task` exporta
    `validate_transition`.
- **E08 — Prime, handoff, diff e learn** (MVP, concluído):
  - `handoff::rewind`: família de estado/handoff — manifest (~30 tokens), escopo (container) e
    working set (âncoras), com ranking por trust-tier
    (`star*100 + foundational*50 + tactical*20 + observational*10` — D57).
  - `handoff::budget`: orçamento sem tokenizer (`ceil(chars/4)`, default 4000), truncando o
    último item e ignorando sobra < 100 tokens (D40/D82); `apply_into` escreve direto no destino
    (streaming) sem montar saída gigante.
  - `handoff::scope`: auto-context-scope a partir dos arquivos tocados e auto-flip
    (`>100 notas` ou `>5 containers` — D41).
  - `handoff::context`: `context_id` derivado e guardado em `.idx/contexts/`; `--resume` devolve
    **bytes idênticos** (D88).
  - `maintenance::diff`: passado derivado da auditoria de eventos por intervalo/escopo (D21/D33).
  - `maintenance::learn`: propostas determinísticas (`create_note`/`merge`/`supersede`/`link`) a
    partir de eventos + âncoras — nunca escreve (D33/D47).
  - `maintenance::compact`: propõe `concat`/`keep_latest`/`merge_outcomes`; aplica só sob aceite,
    fundindo no `keep` e esquecendo (soft) os demais (D47).
  - `task`: hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task` (máx. 4), `plan`/`epic` como
    `container` sem verdade própria, pai por marcador no corpo + aresta `results_in`, `blocks`
    1-based; ciclo de vida `adopt`/`release`/`review`, `outcome` e `reorder` (D52/D53/D93).
- **E07 — Escrita e protocolo** (MVP, concluído):
  - `write::Draft`: rascunho tipado que vira frontmatter válido; opcionais vazios **omitidos**
    (D05); `scope` só para `task`/`container` (D93).
  - `write::dedup`: decisão em três faixas (`<0.75` cria, `0.75–0.92` merge, `≥0.92` rejeita —
    D26), similaridade **Dice** sobre termos em `[0,1]` calibrada para os limiares, configurável
    por `[dedup]` (D80).
  - `write::write`: idempotente por conteúdo (D01) — retry devolve o mesmo id sem duplicar;
    `task`/`container` rejeitados; merge funde tags/âncoras/corpo e incrementa `revision`;
    rejeição não escreve.
  - `write::update`: `Patch` versionado; mesma chave → edita no lugar; `type`/`statement` novos
    → **supersede** (novo id + `replaces`/`superseded_by` — D01/D48); `history` caminha a cadeia.
  - `write::lifecycle`: `forget`/`restore` soft (`status`), `link` de arestas explícitas com
    ponteiro reverso em `replaces` (D46/D49/D52); transições protegidas.
  - `write::propose_merges`: reconciliação só **propõe** quase-duplicados — nunca funde em
    silêncio (D47/D80).
- **E06 — Retrieval: BM25, âncoras e RRF** (MVP, concluído):
  - `retrieval::token`: tokenização **ASCII explícita** `[a-z0-9_]` (`café` → `caf` — D36),
    com `Cow` no caminho quente e termos de consulta deduplicados.
  - `retrieval::index`: índice derivado `.idx/retrieval.jsonl` (uma linha JSON por nota) com
    frequências por campo (`statement`/`body`/`tags`); reconstruível **byte a byte**; ausente →
    reconstrói e grava; teto de tamanho emite aviso (D15/D27).
  - `retrieval::bm25`: BM25 (`k1=1.5`, `b=0.75`) com **IDF por campo**, **peso por tipo** e
    **boost por confirmação** `1 + 0.1*(success + partial*0.5)` (D35/D37/D38).
  - `retrieval::anchor`: canal de âncoras por `path`/`id` com globs `?`/`*`/`**` (D81/D86).
  - `retrieval::rrf`: fusão `1/(k+rank+1)` com `k=60` e desempate `(score desc, id asc)` (D81).
  - `retrieval::filter`: filtros determinísticos (`type`/`classification`/`status`/`tags`/`anchors`)
    aplicados antes do BM25; `container` resolvido no grafo via `depends_on` transitivo (D41).
  - `retrieval::views`: `ready`/`blocked` computadas do `depends_on` transitivo; ciclo de
    dependência = `blocked` (D53).
  - `retrieval::why` + contrato `id|statement|score|why` (4ª coluna com conjunto fechado — D39);
    `get(ids)` devolve corpo só dos ids pedidos.
  - Degradação graciosa: canal falho retorna resultado parcial + `warnings`; `strict` (D94)
    promove a erro (E06-T07).
- **E05 — Grafo e arestas** (Fase 0, concluído):
  - `schema::edge`: enum fechado `EdgeKind` (8 valores) com chaves de frontmatter em
    `snake_case`, `Edge` e `EDGE_KEYS` (D49/D51).
  - `schema::frontmatter`: as 8 chaves de aresta entram na **ordem canônica** (após
    `superseded_by`, antes de `revision` — **27 chaves**, D98), com `string_list`, `edges()` e
    validação de formato dos ids.
  - `graph`: projeção das notas (`Graph`), `link()` idempotente (rejeita id inválido e
    auto-aresta) e `expand` BFS determinística que só percorre o **explícito** (D49).
  - `graph::integrity`: arestas penduradas, auto-arestas e bidirecionalidade
    `replaces ↔ superseded_by` (D46).
  - `graph::cycles`: SCC (Kosaraju iterativo, sem dependências) para supersessão (`replaces`)
    e dependência (`depends_on`); `cycle_members()` protege os membros de demolição (D45).
  - `graph::extract` + `graph::suggestions`: extração conservadora (ids, wikilinks, verbos) com
    armazenamento derivado em `.idx/suggestions.jsonl` — separado e auditável (D49/D50).
  - `store::purge` passou a podar ids dentro de listas JSONL (não só descartar a linha), para
    limpar `targets` de sugestões de um alvo removido (D84).
  - Decisão **D98** registrada.
- **E04 — Config, Git e worktree** (Fase 0, concluído):
  - `config`: config em dois níveis (global template + projeto com precedência — D61), schema
    fechado com tipos/enums/defaults, merge profundo, `set/unset/list/get` com validação (D64),
    poda de ancestrais vazios e sanitização de segredos (D91).
  - `config/toml`: codec TOML próprio (subset) com leitura que **preserva ordem** (diff mínimo)
    e escrita canônica (D63/D97); comentários, seções, chaves pontilhadas, strings de uma
    linha, números, listas multilinha; rejeita `[[...]]`/multilinha/`null`.
  - `git::project`: resolve o **worktree principal** (`--git-common-dir`), compartilha `.knudge/`
    entre worktrees ligados, usa o top-level do **submódulo** e valida o nome lógico (D29/D91).
  - `git::exclude`: exclusão idempotente via `.git/info/exclude` (nunca `.gitignore` — D30),
    versionando `notas/`/`eventos/` e excluindo só o derivado, com reversão ao alternar o modo
    (D34); no-op fora de repo.
  - `git::attributes` + `git::block`: bloco `merge=union` para `eventos*.jsonl` em
    `.gitattributes`, delimitado por marcadores e idempotente (D31/D60).
  - `git::agent_md`: gera/atualiza o `AGENTS.md` do projeto-alvo com version marker, sem
    duplicar nem sobrescrever o conteúdo do usuário (D57/D60).
  - `git::onboard`: cria a árvore `.knudge/`, clona o global **literalmente** (ou usa defaults),
    aplica exclude/attributes/AGENTS e é idempotente (D62).
  - `git::sync`: guard de worktree com `git -C <raiz>`, commit de `notas/`+`eventos/` e mensagem
    gerada do último evento (D32).
  - Porta `Git` ampliada (`common_dir`, `top_level`, `superproject_root`, `run`) e adaptador
    `StdGit` sem shell (R12); `MemFs` passou a modelar diretórios explícitos.
  - Decisão **D97** registrada.
- **E03 — Store, notas e eventos** (Fase 0, concluído):
  - `jsonl`: codec JSON próprio (`encode`/`decode` canônicos, chaves ordenadas e sem
    dependência externa) e leitor de linhas tolerante a CRLF/linhas em branco.
  - `store`: `Note` (frontmatter + corpo) com render/parse byte-exato, `Store` (write atômico,
    list, read, `update` com `revision`, `remove`) e ordem de commit **nota → evento** (D21).
  - `store/events`: `Event` com `id` derivado do conteúdo (`evt_<base36(8)>`), `EventLog`
    append-only com **dedup on-read** (D26/D28), tolerância a linha malformada, rotação por
    tamanho (`events-NNNN.jsonl`), checkpoint derivado (`.idx/events.checkpoint`) e `history`.
  - `store/lock`: lock advisory por arquivo-alvo (`create_exclusive`), stale 30 s, reclaim por
    rename sidecar e liberação RAII (D23–D25/R05).
  - `store/rebuild`: `Staging` double-buffer (`.idx.new/` + rename atômico — D27).
  - `store/purge`: `purge_derived` remove o id de `*.jsonl` e `index.json` em toda remoção
    (D84), usado por `Store::remove`.
  - `store/sweep`: varredura de resíduos `*.tmp`/`*.lock`/`*.stale` por idade, com `warn`
    (R10), **sem** remover lock fresco de processo vivo.
  - Porta `Fs` estendida com `append`, `create_exclusive`, `rename`, `sync`, `modified_ms`,
    `is_dir` e `remove_dir_all`; `MemFs` virou um FS fiel (tmp+rename, diretórios implícitos) e
    `FaultyFs` injeta falhas para testes de crash.
  - `ARCHITECTURE.md` §5 documenta a persistência; decisão **D96** registrada.
- **E02 — Contrato de bytes: TOON, schema e IDs** (Fase 0, concluído):
  - `schema/types`: enums fechados `NoteType` (11), `Scope`, `Classification`, `Status`.
  - `schema/hash`: hash curto `SHA-256 → u32`, `hex8` e `base36(8)` (D95).
  - `schema/body`: `normalize` (NFC + trim + colapso) e `body_hash` (D06).
  - `schema/id`: `id` endereçado por conteúdo (`type + U+001F + statement`) e validador (D01–D03).
  - `schema/text`: contagem de `statement` em escalares Unicode, limite 120 (D08).
  - `schema/frontmatter`: ordem canônica das 19 chaves, omissão de opcionais (D05), leitura
    tolerante com warning (D16) e validação (T01/T07).
  - `toon`: parser/emissor próprios (`lex`/`flow`/`parse`/`emit`), round-trip byte-exato,
    comentários, escapes, zero bytes, `1.0 → 1` e detecção de `schema_version` (D74/D75).
  - `TOON.md` publica a gramática e as regras de bytes.
- **E01 — Fundação** (Fase 0, concluído):
  - Workspace Cargo (`knudge-core`, `knudge-cli`, `knudge-mcp`) com `edition = "2024"`,
    MSRV `1.97`, `[workspace.lints]` (R44), perfil de release (R41) e `make check` (E01-T01/T03).
  - Portas determinísticas (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`),
    adaptadores `std` e fakes (`FixedClock`, `SeqRng`, `FakeEnv`, `FakeGit`, `MemFs`,
    `RecordingLogger`) (E01-T02).
  - Modelo de erro `Error`/`ErrorKind` com mapa código→exit, `thiserror`, poison via
    `into_inner` e contexto de I/O com path (E01-T06; R30–R35).
  - `Timestamp` UTC com milissegundos, formatação/parsing próprios e round-trip por proptest
    (E01-T02; D07).
  - Redação de segredos no layer de log (allowlist + chaves sensíveis) (E01-T07; R22).
  - Binário `kd` com a **superfície v2** (E01-T04; `16_cli_surface.md`): `kd` = `prime`,
    `--json` (`{success, command, data?, error?, warnings?}`), exit codes estáveis e
    `EPIPE → exit 0`.
  - Política de memória: `#![forbid(unsafe_code)]`, rejeição de symlink em `.knudge/` (E01-T08).
  - `ARCHITECTURE.md`, `MODULE.md` por crate e módulos temáticos de `knudge-core` (E01-T05).

### Decidido (superfície CLI v2)
- **D57** reescrita: `prime` é protocolo estático byte-idêntico; estado/handoff → `rewind`.
- **D88** ajustada: `rewind` emite `context_id`; `kd rewind --resume <id>`.
- **D93**: `task` com `scope` fechado (`plan|epic|issue|task`), hierarquia máx. 4.
- **D94**: `strict` é config de projeto (`[behavior] strict`), sem flag.
- **D95**: hash curto `SHA-256 → u32`; chave/derivação de `id` e gramática TOON v1 (`TOON.md`).
- **D96**: registro de evento (`id` derivado + dedup on-read), rotação por tamanho e
  semântica de `revision`.
- **D97**: subset TOML (ordem preservada/canônica), guard `git -C` do `sync` e degradação do
  `onboard` sem config global.
- **D98**: arestas como chaves de frontmatter (27 chaves na ordem canônica), ponteiro reverso
  `superseded_by`, ciclo de supersessão sobre `replaces` e sugestões derivadas.

### Testes
- 323 testes de unidade no `knudge-core` (erro, tempo/proptest, redação, fakes, symlink,
  schema/hash/ID/arestas, TOON, JSONL/JSON, store, config/TOML, git/onboard/sync,
  grafo/integridade/ciclos/sugestões, retrieval/token/BM25/âncoras/RRF/views,
  escrita/dedup/update/supersede/forget, rewind/orçamento/context_id, diff/learn/compact,
  tarefas/hierarquia/ciclo de vida, validators/evidência/audit/doctor/âncoras/confiança,
  shelf-life/decay/purga/ciclos/clusters).
- 8 testes de integração do binário (`--help`, `kd == kd prime`, `--json`, exit codes,
  EPIPE, comando desconhecido).
