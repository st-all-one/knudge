# Memória de agentes v2 — identidade, tempo, concorrência e `.knudge/ai`

> **Status:** implementado na 0.6.3 (D219–D224). Divergência aceita: não há migração nem
> retrocompatibilidade — `kd ai migrate` e o fallback para `.agents/memory` foram substituídos por
> D224.
>
> **Origem:** falhas observadas no `kd ai` com memória JSONL: `upsert` O(N²), arquivo único
> disputado por subagentes, ausência de identidade de agente e de ordenação por `created_at`.
>
> **Decisões:** D219–D224 aceitas e registradas em `plan/03_decisoes-fechadas.md`. Revisa/complementa
> D218.
>
> **Regras invioláveis (AGENTS.md):** `make check` verde; `src/` ≤ 300 linhas; sem
> `unwrap/expect/panic/unsafe`; núcleo puro via portas; sem servidor/DB/`tokio full`; índice
> derivado e reconstruível; stdout = dados, stderr = logs.

---

## 0. Resumo

O `kd ai` já lê notas JSONL e já tem `created_at` no schema, mas:

- o `upsert` do JSONL é **O(N²)** (`Vec` + busca linear por linha);
- a fonte é um **arquivo único**, sem contrato para múltiplos subagentes escrevendo ao mesmo
  tempo;
- `provenance` não distingue **agente principal** de **subagente** nem carrega um **id de
  instância**;
- `kd ai` ordena por `id` (hash de conteúdo) e **não expõe** `created_at` na saída;
- a memória ainda mora em `.agents/memory`, fora do diretório do projeto.

A proposta:

1. Migrar a memória para `.knudge/ai/`, com **um shard NDJSON por agente**.
2. Manter **JSONL/NDJSON**, não JSON-LD: append-only, linha a linha, legível e tolerante a
   crash; JSON-LD não é adequado a escrita concorrente incremental.
3. Trocar o `upsert` linear por `BTreeMap` (determinístico e permitido pelo clippy do repo).
4. Adicionar identidade em `provenance`: `agent_id`, `agent_role`, `parent_agent_id`.
5. Expor `created_at` e ordenar com `--sort time`, com desempate determinístico.
6. Criar `kd ai migrate` e um fallback temporário para `.agents/memory`.

---

## 1. Evidência no código atual

| Ponto | Onde | Problema |
|---|---|---|
| `upsert` linear | `crates/knudge-core/src/corpus/source.rs::upsert` | para cada linha, varre o vetor inteiro ⇒ O(N²) |
| Fonte única | `Source::Jsonl(PathBuf)` | um arquivo por área; subagentes concorrentes disputam o mesmo append |
| Ordem | `parse_jsonl` ordena por `id`; `Index::build` reordena por `id` | não há ordem cronológica |
| `created_at` | `schema/frontmatter.rs` + `Note::to_jsonl` | existe no frontmatter, mas não sai em `commands/ai.rs` |
| Identidade | `schema/provenance.rs` | só `entity`/`activity`/`agent`; sem id/role/parent |
| Caminho | `cli/ai.rs::AiArea::default_path` | `.agents/memory/*.jsonl` |
| Saída | `commands/ai.rs::list_json`/`hits` | só `id`, `type`, `statement`/`score` |

Consequência observada: ~10k notas com body pequeno já custam ~2,3 s e ~68 MB por `kd ai`
(medição local). Com escritas concorrentes, o “última linha vence” é uma corrida não
determinística, e cada subagente pode perder a nota do outro.

---

## 2. Decisões propostas

| Id | Decisão |
|---|---|
| **D219** | **Formato canônico = NDJSON** (`{"frontmatter":…,"body":…}` por linha). **JSON-LD não é o formato de escrita**: exigiria documento completo/framing, não é append-friendly e piora leitura incremental. JSON-LD pode existir apenas como export futuro. |
| **D220** | **`.knudge/ai/<area>/<agent_id>.jsonl`**, um shard por escritor; `Source::JsonlDir` lê e mescla shards em ordem determinística. `--source FILE` continua aceito para compatibilidade. |
| **D221** | **Identidade em `provenance`**: `agent` (nome lógico), `agent_id` (instância), `agent_role` (`main`/`subagent`), `parent_agent_id`. `schema_version` sobe para 3; notas v2 leem com defaults. |
| **D222** | **`created_at` é o eixo temporal**: exposto em `kd ai --json`, com `--sort id\|time`; desempate `(created_at, agent_id, id)`. Escritores mantêm `created_at` monotônico por agente. |
| **D223** | **Dedup O(N log N)** com `BTreeMap<String, Note>`; conflito entre shards resolvido por `(created_at, agent_id, ordem do shard/linha)`, determinístico. |
| **D224** | **Migração explícita** `kd ai migrate` de `.agents/memory` para `.knudge/ai`, idempotente, com `--dry-run` e `--remove-old`; fallback de leitura por uma minor. |

---

## 3. Layout proposto

```
.knudge/
  ai/
    README.md                 # contrato curto do diretório
    memory/
      <agent_id>.jsonl        # notas de memória
    session-log/
      <agent_id>.jsonl        # projeção do log de sessão
    events/                   # reservado (journal do agente; fora deste plano)
    .idx/                     # derivado, opcional, reconstruível
```

Regras:

- **Um arquivo por `agent_id`** (processo/sessão de agente). O agente principal e cada
  subagente escrevem no próprio shard; não há arquivo compartilhado no caminho quente.
- `agent_id` precisa ser seguro para nome de arquivo (`[A-Za-z0-9._-]`, ≤ 96 chars). A
  recomendação é ULID/UUIDv7 com prefixo: `omp:01J...` vira `omp-01J....jsonl`; se o id não
  for seguro, usar hash curto determinístico.
- Não há `manifest.json` mutável: o leitor varre `*.jsonl` em ordem lexicográfica. Um manifesto
  compartilhado reintroduziria a disputa de escrita que o shard elimina.
- `--source` aceita arquivo **ou diretório**; o default da área vira o diretório.
- `.knudge/ai` segue a política de persistência de `.knudge` (`knowledge.persist_in_project`).
  O conteúdo continua sendo do **agente**; o knudge só lê.
- `events/` fica reservado para um journal por evento (ordem exata/retomada). Este plano não o
  implementa; `memory/` e `session-log/` são as fontes consultáveis.

Compatibilidade:

```text
novo default memory       .knudge/ai/memory
novo default session-log  .knudge/ai/session-log
legado (fallback)         .agents/memory/notes.jsonl
                          .agents/memory/session-log.jsonl
```

O `Source::JsonlDir` detecta diretório; um arquivo solto continua funcionando via `--source`.

---

## 4. Identidade do agente

### 4.1 Campos

Estender `provenance` (mapa já existente), sem novas chaves de topo:

```json
"provenance": {
  "activity": "derive",
  "agent": "omp",
  "agent_id": "omp:01J9Z6K3W8V4M2Q1",
  "agent_role": "subagent",
  "parent_agent_id": "omp:01J9Z5A0P7H3N4R2"
}
```

| Campo | Semântica | Obrigatório |
|---|---|---|
| `agent` | nome lógico estável (`omp`, `ktu`) — mantém `kd ask --agent` | não |
| `agent_id` | identidade da instância (processo/sessão); único por shard | recomendado |
| `agent_role` | `main` ou `subagent` (enum fechado) | recomendado |
| `parent_agent_id` | id do agente que criou o subagente | só em subagente |

- `agent_id` **não** entra no `id` da nota (o id continua content-addressed por D95).
- `agent` continua sendo o filtro existente; `agent_id`/`agent_role` ganham filtros novos.
- Notas antigas sem os campos: `agent_id = agent` (ou `"legacy"`), `agent_role = "main"`,
  `parent_agent_id = None` na leitura. Não reescrever o corpus na migração além do necessário.
- `schema_version` 3 no writer; `Frontmatter::validate` aceita 2 e 3 (D15).
- `Provenance::to_value` mantém a ordem dos campos antigos (`entity`, `activity`, `agent`) e
  anexa os novos no fim, para preservar bytes de notas existentes.

### 4.2 CLI e índice

- `kd ai --json` passa a devolver `agent`, `agent_id`, `agent_role`, `parent_agent_id`,
  `created_at` por nota/hit.
- `kd ask`/`kd ai` ganham `--agent-id` e `--agent-role` (repetíveis/vírgula, D210) e
  `--params` equivalentes.
- `Meta`/`Filter` ganham os campos; `INDEX_FORMAT` sobe (`retrieval-v5`) e o codec persiste os
  novos campos. Índice antigo lê `None` e continua válido (D15) ou é reconstruído pelo formato.

---

## 5. Tempo e ordem

`created_at` já é chave obrigatória. O que falta é contrato de uso:

1. **Writer:** `created_at` em ms, monotônico por agente. Se dois eventos caírem no mesmo
   milissegundo, o agente incrementa `created_at` em 1 ms (ou grava o `seq` no journal, fora
   deste plano).
2. **Leitura:** ordem determinística `(created_at, agent_id, id)`.
3. **Saída:** `created_at` (int ms) em `--json`; a saída de texto padrão continua
   `id|type|statement` para não quebrar consumidores.
4. **CLI:** `kd ai ... --sort id|time`:
   - default `id` (compatível);
   - `--sort time` ordena a lista por `(created_at, agent_id, id)`;
   - no recall, o default continua score; `--sort time` reordena os hits já limitados.
5. **`kd ask --json`:** incluir `created_at` e identidade nos hits (sem mudar a ordem default).

Sem isso, `kd ai` não consegue responder “o que aconteceu primeiro”; ele é retrieval, não
transcript. A ordem exata de eventos repetidos continua exigindo o journal do agente.

---

## 6. Dedup e concorrência

### 6.1 `BTreeMap` no lugar do `Vec`

`parse_jsonl` passa a acumular em `BTreeMap<String, Note>`:

- O(log N) por linha, O(N log N) total;
- iteração final por `id`, igual à ordem determinística atual;
- sem `HashMap` (proibido no core pelo `clippy.toml`).

`Source::JsonlDir` lê os arquivos em ordem lexicográfica e mescla no mesmo mapa. Conflito de
mesmo `id` entre shards:

```text
vence o maior (created_at, agent_id, ordem_shard, ordem_linha)
```

- mesmo shard: última linha vence (comportamento atual);
- shards diferentes: `created_at` decide; empate por `agent_id` e ordem do arquivo/linha;
- linha malformada continua *skip + warning*, agora com o nome do shard no aviso.

### 6.2 Escrita concorrente

- Um shard por `agent_id` elimina escrita compartilhada entre agente principal e subagentes.
- Cada linha é escrita com **um único append** (`O_APPEND`) de JSON completo + `\n`.
- O agente pode fazer `fsync` por evento ou em batch; o knudge nunca escreve nesses arquivos.
- Linha parcial no fim (crash) é ignorada com warning na próxima leitura.
- `kd ai` pode ler enquanto há escritores: cada linha é completa; não há lock de leitura.
- Compactação de shards (merge periódico) fica para uma fase posterior; não é necessária para
  correção.

### 6.3 Por que não JSON-LD nem SQLite agora

| Alternativa | Decisão | Motivo |
|---|---|---|
| JSON-LD | rejeitado como formato canônico | exige documento/framing; não é append-line; pior para crash/concorrência |
| SQLite | adiado | resolveria ACID/índice, mas adiciona DB e perde legibilidade; o repo proíbe DB sem decisão explícita; pode virar índice derivado se >100k notas |
| Parquet/Arrow | rejeitado | não é append textual nem legível; overhead para notas pequenas |
| NDJSON sharded | **escolhido** | append-only, diff legível, tolerante a linha parcial, sem lock no hot path |

---

## 7. Migração `.agents/memory` → `.knudge/ai`

### 7.1 Comando

```bash
kd ai migrate [--from .agents/memory] [--to .knudge/ai] [--dry-run] [--remove-old] [--json]
```

- `--from` default é `.agents/memory`; `--to` default é `.knudge/ai`.
- Sem `--remove-old`, os arquivos antigos ficam intactos.
- `--dry-run` reporta contagens, ids duplicados e warnings sem escrever.
- `--json` devolve `{from,to,areas,notes,duplicates,warnings,removed}`.

### 7.2 Passos

1. Resolver a raiz do worktree principal (mesmo `Project::resolve` de D218).
2. Ler `notes.jsonl` e `session-log.jsonl` com o parser tolerante atual.
3. Enriquecer `provenance` ausente:
   - `agent_id = provenance.agent` ou `"legacy"`;
   - `agent_role = "main"`;
   - `parent_agent_id` ausente.
4. Deduplicar por `id` com a mesma política da leitura.
5. Agrupar por `agent_id`, escrever cada shard em `*.tmp` + rename atômico.
6. Se `--remove-old`, remover os arquivos antigos **somente** após todos os shards novos
   existirem e serem re-lidos com sucesso.
7. `kd ai` novo default lê `.knudge/ai`; se ausente, lê o legado com warning
   `"memória legada; rode kd ai migrate"` por uma minor.

### 7.3 Idempotência e segurança

- Reexecutar a migração lê o destino existente, mescla por `id` e não duplica linhas.
- Não migrar com agentes ativos: exigir `--force` ou um lock `.knudge/ai/.migrate.lock`.
- Nunca apagar o legado por default; `--remove-old` é explícito.
- Migração não reescreve `body_hash`/`id`; só acrescenta proveniência ausente.

---

## 8. Mudanças por arquivo

### Core

- `corpus/source.rs` — `Source::JsonlDir`, `BTreeMap`, conflito, warnings com shard.
- `corpus/mod.rs` — `Source::path`/`dir` e `from_notes` inalterados.
- `schema/provenance.rs` — novos campos, parse/serialize, validação.
- `schema/frontmatter.rs` — validação de `agent_role`/`parent_agent_id`; schema 3.
- `retrieval/filter.rs` — `Meta.agent_id/agent_role/parent_agent_id`; `Filter`.
- `retrieval/index/persist/codec.rs` + `retrieval/index/mod.rs` — campos e `INDEX_FORMAT`.
- `retrieval/mod.rs` — ordenação temporal opcional, sem mudar o default.
- `schema/keys.rs` — **sem** novas chaves de topo; `provenance` é mapa.

### CLI

- `cli/ai.rs` — `AiArgs` com `--sort`, `--agent-id`, `--agent-role`; default paths.
- `commands/ai.rs` — `Source::JsonlDir`, `created_at`/identidade no JSON, `migrate`.
- `cli/mod.rs` — subcomando `ai migrate` ou `AiArgs.migrate` (decidir na implementação;
  recomenda-se subcomando explícito).
- `output.rs`/envelope — sem mudança de contrato; campos novos em `data`.

### Docs/contrato

- `wiki/specs/fontes.md` — layout, shards, identidade, ordem, concorrência.
- `wiki/specs/modelo-de-dados.md` + `TOON.md` — schema 3, `provenance` estendido.
- `wiki/usage/20_ai.md` — novos caminhos, `--sort`, `migrate`.
- `plan/implementation/16_cli_surface.md` e `17_matriz_aceitacao.md` — flags/saída/migração.
- `wiki/specs/adrs/` — ADR D219–D224.
- `wiki/CHANGELOG.md`, `llms.txt`, `SKILL.md` — atualizar após implementação.

---

## 9. Tarefas e aceite

| Tarefa | Conteúdo | Aceite |
|---|---|---|
| **T01** Schema/identidade | `provenance` + schema 3 + validação + goldens | notas v2 leem com defaults; v3 round-trip byte-exact |
| **T02** Fonte/dir | `Source::JsonlDir`, `BTreeMap`, merge/conflito | 1 shard e N shards dão o mesmo corpus; linha má = warning |
| **T03** Concorrência | shards por `agent_id`; stress de escritores | N threads escrevem; leitura vê todas as notas; sem linha parcial |
| **T04** CLI/tempo | `created_at`/identidade no JSON; `--sort time`; filtros | golden de `--json`; ordem determinística |
| **T05** Migração | `kd ai migrate` idempotente + fallback | dry-run/remove-old; sem perda de `id`/`body_hash` |
| **T06** Performance | benchmark 1k/10k/100k; BTreeMap | registrar antes/depois em `bench/`; sem regressão >10% |
| **T07** Docs | specs/usage/CLI/CHANGELOG/ADR | links e matriz de aceite atualizados |

Ordem: T01 → T02 → T03 → T04 → T05 → T06/T07.

---

## 10. Testes mínimos

- `parse_jsonl_dedups_with_last_line_wins` (existente) continua.
- `jsonl_dir_merges_shards_deterministically` — ordem de arquivos/ids.
- `jsonl_dir_same_id_conflict_uses_created_at_then_agent_id`.
- `jsonl_dir_ignores_malformed_line_with_shard_warning`.
- `jsonl_dir_reads_while_writer_appends` — stress com `thread::scope`.
- `provenance_round_trips_agent_identity` + `old_v2_note_defaults_identity`.
- `kd ai --json` inclui `created_at`/`agent_*`; `--sort time` ordena.
- `kd ai migrate --dry-run/--remove-old` idempotente.
- `make check` verde; arquivos ≤300 linhas; sem `unwrap/expect/panic`.

---

## 11. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| Schema 3 quebrar nota antiga | on-read defaults; `validate` aceita 2/3; goldens v2 |
| Índice antigo sem campos novos | bump `INDEX_FORMAT` (`retrieval-v5`); rebuild derivado |
| Migração perder dados | escrever tmp+rename; reler destino; `--remove-old` só após sucesso |
| Muitos shards | compactação futura; limite de shards por agente; documentar |
| Ordem entre subagentes | `created_at` monotônico + `agent_id`; ordem exata só no journal |
| `created_at` empatado | incremento monotônico no writer; desempate determinístico |
| `.knudge/ai` no git | seguir `persist_in_project`; documentar que é memória do agente |
| Escrita concorrente no mesmo shard | um shard por `agent_id`; se dois processos usarem o mesmo id, append atômico + conflito determinístico |

---

## 12. Quem resolve o quê

- **knudge-core/CLI:** schema, `Source::JsonlDir`, `BTreeMap`, merges, ordenação, saída,
  migração, ADRs, testes e benchmark.
- **Agente/plugin:** escrever no novo layout, um shard por `agent_id`, fornecer
  `agent`/`agent_id`/`agent_role`/`parent_agent_id`, `created_at` monotônico, append por evento.
  O plugin omp precisa trocar `DEFAULT_MEMORY_SOURCES` e emitir a identidade.
- **Usuário/projeto:** rodar `kd ai migrate` uma vez; decidir `persist_in_project`.
- **Fora do knudge:** o journal/streaming por evento e a recuperação pós-crash continuam sendo
  responsabilidade do agente. O layout `events/` fica reservado, mas não é implementado aqui.

---

## 13. Fora de escopo

- JSON-LD como formato de escrita.
- SQLite como store canônico (pode virar índice derivado se a escala exigir).
- Compactação automática de shards e retenção (fase posterior).
- Journal de eventos/streaming e replay pós-crash (plano do agente/plugin).
- Ordenação global entre múltiplos hosts/relógios; o escopo é um projeto/worktree.
