# Fontes de corpus

Como o knudge escolhe **de onde** lê as notas, como consulta a memória de um **agente** sem
aprender o seu schema interno, e como essa memória continua a ser **uma só verdade**. É a porta de
entrada do knudge como **eixo único de memória, agent-agnostic** (D218–D224).

- Código: `crates/knudge-core/src/corpus/source.rs`, `corpus/source/jsonl.rs`,
  `crates/knudge-core/src/store/note.rs`
- CLI: `crates/knudge-cli/src/commands/ai.rs`, `crates/knudge-cli/src/cli/ai.rs`
- Guia de uso: [`../usage/20_ai.md`](../usage/20_ai.md)
- Guia de integração do escritor: [`../integration/ai-memory.md`](../integration/ai-memory.md)
- Decisões: D218 (a porta), D219 (NDJSON canónico), D220 (shards), D221 (identidade), D222 (ordem),
  D223 (merge), D224 (sem caminho legado)

## Princípio

> O knudge **indexa** (derivado); o agente **possui** a fonte.

O contrato entre os dois é o **documento** — uma nota no schema do knudge — e **nunca** o schema
interno do agente. Por isso o knudge lê NDJSON de notas (formato que ele próprio define), e não o
log cru de um agente qualquer.

## Layout canônico

```
.knudge/ai/
  memory/<agent_id>.jsonl        # memória do agente (default da área memory)
  session-log/<agent_id>.jsonl   # projeção do log (activity=derive)
  events/                        # reservado (não implementado)
  README.md                      # contrato do diretório (opcional, do agente)
```

- **Um shard por `agent_id`** (processo/sessão), nunca um ficheiro compartilhado no caminho quente.
  O agente principal e cada subagente escrevem no seu próprio ficheiro.
- `.knudge/ai` segue `knowledge.persist_in_project`, como o resto de `.knudge/` (ver
  [`persistencia.md`](persistencia.md)).
- **Sem manifesto mutável**: o leitor varre os `*.jsonl` de primeiro nível do diretório em ordem
  lexicográfica (não recursivo); não há ficheiro de índice da fonte a atualizar nem lock de leitura.
- `events/` está reservado e não é lido.

### Nome do shard

`shard_file_name(agent_id)` é determinístico e seguro para o sistema de ficheiros:

- `agent_id` já seguro — apenas `[A-Za-z0-9._-]`, ≤ 96 caracteres e diferente de `.`/`..` — vira
  `<agent_id>.jsonl` (ex.: `omp-main.jsonl`).
- Qualquer outro id vira `<prefixo-saneado>-<hex8>.jsonl`: os caracteres inseguros viram `-`, o
  prefixo é truncado a 48 caracteres (ou `agent` se ficar vazio) e `hex8` são os **4 primeiros
  bytes do SHA-256 do id original**, em hexadecimal minúsculo. Ex.: `omp main/1` →
  `omp-main-1-5381c52b.jsonl`. O hash preserva a identidade e evita colisões práticas.

O `agent_id` usado no nome é o mesmo gravado em `provenance.agent_id`; um id com caracteres de
controlo ou acima de 128 caracteres é rejeitado no schema (ver abaixo).

## Formato e leitura

NDJSON/JSONL **append-only**, uma nota por linha:

```json
{"frontmatter":{"id":"fact_0a1b2c3d","type":"fact","statement":"O gateway limita a 100 rps por chave","created_at":1761700000000,"body_hash":"1a2b3c4d","schema_version":3,"provenance":{"activity":"write","agent":"omp","agent_id":"omp-main","agent_role":"main"}},"body":"Medido no gateway de produção."}
```

- O envelope é `{"frontmatter": {...}, "body": "..."}` e o codec é `Note::to_jsonl` /
  `Note::from_jsonl`. JSON-LD **não** é formato de escrita.
- `Note::from_jsonl` valida o schema (`Frontmatter::from_value` + `validate`): JSON inválido,
  envelope que não é objeto, `frontmatter` ausente ou nota inválida são rejeitados.
- **Dentro do mesmo shard** vence a última linha com o mesmo `id` (D26): o ficheiro pode ser
  append-only e a correção mais recente sobrepõe-se.
- **Entre shards**, o mesmo `id` resolve pelo maior `(created_at, agent_id, caminho do shard)`
  (D223) — sempre determinístico, independentemente da ordem de leitura.
- **Linha malformada = skip + warning** com o caminho do shard e o número da linha (1-based); a
  leitura nunca falha por causa dela (D18).
- **Ficheiro/diretório ausente** ⇒ corpus vazio, sem aviso e com exit `0`. Ficheiro ilegível ou
  não-UTF-8 gera warning e é ignorado.
- Dedup O(N log N) com `BTreeMap<String, Note>`; a saída final é ordenada por `id`, o mesmo
  contrato de `Index::build` e do índice derivado.
- A leitura pode correr **durante** escritas: cada linha é completa; uma linha parcial no fim do
  ficheiro (escrita ainda a meio) é ignorada com warning naquela leitura e entra na leitura
  seguinte assim que o escritor a completa. Não há lock de leitura.

## `Source` — a porta

| `Source` | Onde lê | Papel |
|---|---|---|
| `NotesDir` (default interno) | `notas/<tipo>/<id>.md` | a verdade do knudge, inalterada |
| `Jsonl(path)` | um ficheiro NDJSON explícito | `--source FILE` |
| `JsonlDir(path)` | todos os `*.jsonl` de um diretório | `--source DIR` / default da área |

```rust
let source = Source::jsonl_dir(".knudge/ai/memory");
let (corpus, warnings) = source.load(&store)?;   // notas + índice + grafo

let one_file = Source::jsonl("/tmp/export.jsonl");
let chosen = Source::at(store.fs(), "--source recebido");
```

- `Source::at(fs, path)` escolhe `JsonlDir` quando o caminho é um diretório e `Jsonl` caso contrário;
  um caminho inexistente é tratado como ficheiro (o corpus fica vazio, sem erro).
- `Source::load_notes` devolve `Loaded = (Vec<Note>, Vec<String>)`; `load` acrescenta índice e grafo.
- `kd ai` usa `Source::at` para `--source` e `Source::jsonl_dir` para o default da área.

## Schema 3 e identidade

- `SCHEMA_VERSION = 3`; notas v2 continuam válidas (`validate` aceita 2 e 3). O bump é aditivo: os
  bytes de notas v2 não são reescritos.
- `provenance` é um mapa opcional, **sem novas chaves de topo**:

  | Campo | Papel | Contrato |
  |---|---|---|
  | `agent` | nome lógico (`omp`, `ktu`, …) | filtro histórico; string trimada |
  | `agent_id` | instância (processo/sessão) | ≤ 128 caracteres, sem controlo; filtro principal |
  | `agent_role` | papel da instância | enum fechado `main` \| `subagent` |
  | `parent_agent_id` | agente que criou o subagente | recomendado só em `subagent`; ≤ 128, sem controlo |

  `entity`/`activity` mantêm o papel PROV-lite de D207.
- **Ordem de serialização**: `entity`, `activity`, `agent`, `agent_id`, `agent_role`,
  `parent_agent_id`; campos ausentes são omitidos. `to_value` só emite identidade presente, pelo
  que notas v2 mantêm os bytes.
- **Defaults de leitura** (sem reescrever a nota): `agent_id` = `agent_id` ou `agent` ou
  `"legacy"`; `agent_role` = `main`; `parent_agent_id` = ausente.
- Validação: `agent_id`/`parent_agent_id` acima de 128 caracteres ou com caracteres de controlo
  **rejeitam a nota**; `agent_role` fora do enum também. Strings vazias após trim são rejeitadas.
- **`agent_id` não entra no `id`**: o id continua content-addressed por `type + statement`
  (D01–D03). A mesma afirmação escrita por dois agentes tem o mesmo `id`; a proveniência distingue
  quem a produziu.

## Filtros de proveniência

`kd ask` filtra as notas de `notas/` por **quem** as produziu (D207/D221):

- `--agent NOME` — igualdade com `provenance.agent` (nome lógico);
- `--activity ATIVIDADE` — igualdade com `provenance.activity` (`write`/`learn`/`import`/`derive`);
- `--agent-id ID` — igualdade com `provenance.agent_id` (instância);
- `--agent-role PAPEL` — `main`/`subagent`.

São listas de seleção (repetição ou vírgula) e o `--params` transporta os mesmos nomes `agent`,
`activity`, `agent_id` e `agent_role`. Os filtros de identidade aplicam os **defaults de leitura**
de D221: `agent_id` cai para `agent` (ou `legacy`) e `agent_role` cai para `main`, de modo que uma
nota v2 é filtrável como na lista do `kd ai`. A filtragem é **estrutural** — ocorre antes da estatística,
como todos os filtros (D41/D53). `Meta`/`Filter` carregam os campos e o índice `retrieval-v5`
persiste-os; um índice anterior é rejeitado e reconstruído.

```bash
kd ask --agent ktu "rate limit do gateway"       # o que o ktu registou
kd ask --activity derive --type meta "sessao"    # o que foi projetado de logs
kd ask --params '{"query":"jsonl","agent":["ktu"]}'
```

## Tempo e ordem

- `created_at` em **milissegundos** é o eixo temporal; cada escritor mantém-no monotónico.
- Ordem determinística: `(created_at, agent_id, id)`.
- `kd ai --sort id|time` — default `id`. No recall o default é **score**; `--sort time` reordena
  apenas os hits já limitados.
- `created_at` e a identidade aparecem no `--json` de `kd ai` e nos hits de `kd ask --json`.

## `kd ai` — a área de consulta

```
kd ai [AREA] [QUERY...] [--source PATH] [--limit N] [--sort id|time]
      [--agent-id ID...] [--agent-role main|subagent ...] [--params JSON|-]
```

| Aspecto | Contrato |
|---|---|
| Área | posicional: `memory` (default) ou `session-log`; fora do conjunto ⇒ exit `2` |
| Fonte | `--source` (ficheiro **ou** diretório; `Source::at`); default `.knudge/ai/<área>/` |
| Sem query | **lista** (`id\|tipo\|statement`), na ordem `--sort` |
| Com query | **recall** sobre a fonte (mesma cascata do `ask`; universo inteiro; estados visíveis) |
| Filtros | `--agent-id`/`--agent-role` (repetíveis ou vírgula), também no `--params` |
| `--params` | objeto JSON que define a consulta inteira: `query`, `area`, `source`, `limit`, `sort`, `agent_id`, `agent_role` |
| Saída de texto | lista `id\|tipo\|statement`; recall `id\|statement\|score`; vazio `[no_results]` |
| `--json` | envelope padrão; payload `{area, source, sort, query, data}` e `warnings[]` no envelope; `data.notes[]`/`data.hits[]` com `created_at` e identidade |

A área é **read-only**: não há `--add` nem escrita na fonte. Escrever é do agente; o `kd` só lê,
deriva índice/grafo e responde.

## Índice derivado

- `INDEX_FORMAT = "retrieval-v5"`; o codec persiste `agent_id`, `agent_role` e `parent_agent_id`
  (opcionais) em `Meta`.
- `Meta`/`Filter` carregam esses campos; os filtros de identidade são **estruturais** (antes da
  estatística, como todos os filtros).
- Um índice **v4 é rejeitado e reconstruído** — é derivado, nunca fonte da verdade.
- `kd ask` aceita `--agent-id`/`--agent-role` (e os mesmos em `--params`); os hits JSON incluem a
  identidade e `created_at`.

## Sem caminho legado

- O default é sempre `.knudge/ai/memory` / `.knudge/ai/session-log`; outra localização exige
  `--source` explícito.
- Ficheiros fora desse layout não são descobertos automaticamente.
- Índices anteriores a `retrieval-v5` não são lidos: são reconstruídos a partir da fonte.

## Testes

| Comportamento | Teste |
|---|---|
| Shards mesclam de forma determinística | `corpus::tests::jsonl_dir_merges_shards_deterministically` |
| Conflito do mesmo `id` entre shards: `created_at` e depois `agent_id` | `corpus::tests::jsonl_dir_same_id_conflict_uses_created_at_then_agent_id` |
| Linha malformada = skip + warning com o shard | `corpus::tests::jsonl_dir_ignores_malformed_line_with_shard_warning` |
| Leitura concorrente com escritor a fazer append | `corpus::tests::jsonl_dir_reads_while_writer_appends` |
| Nome de shard seguro e determinístico | `corpus::tests::shard_file_name_is_safe_and_deterministic` |
| `Source::at` distingue ficheiro de diretório | `corpus::tests::source_at_detects_file_and_directory` |
| Identidade de agente faz round-trip em `provenance` | `schema::tests::semantic::provenance_round_trips_agent_identity` |
| Nota antiga ganha defaults sem reescrever bytes | `schema::tests::semantic::old_note_defaults_identity_without_rewriting_bytes` |
| Identidade malformada é rejeitada | `schema::tests::semantic::malformed_agent_identity_is_rejected` |
| Filtros de identidade e papel | `retrieval::tests::filter::provenance_filters_match_agent_identity_and_role` |
| `kd ai` (lista, recall, sort, filtros, `--params`, identidade, shards) | `crates/knudge-cli/tests/ai.rs` |
