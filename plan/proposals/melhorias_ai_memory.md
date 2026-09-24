# Melhorias destiladas do `ai-memory` (proposta)

> **Status:** ✅ **implementada na v0.3.2** — os itens 2.1–2.6 e 2.8 viraram decisões fechadas
> `D154`–`D159` (`plan/03_decisoes-fechadas.md`) e código; 2.7 e 2.9 foram descartados. O
> documento fica como registro da análise de origem. Origem: investigação profunda de
> `TMP/ai-memory` (v2.4.0) confrontada com o estado do `knudge` (v0.3.1). Cada item descreve a
> decisão que foi aberta, com CLI, core, contrato/config, determinismo, erros, testes e aceite.
>
> **Regras invioláveis (AGENTS.md):** `make check` verde; `src/` ≤ 300 linhas; sem
> `unwrap/expect/panic/unsafe`; núcleo puro via portas; "propor, nunca agir em silêncio" (D47);
> índice derivado e reconstruível (D15/D27); sem servidor/DB/`tokio full` (R16/R43).
>
> **Documentos irmãos:** `00_panorama.md`, `03_decisoes-fechadas.md` (D01–D153),
> `14_revisao_tecnica.md` (R01–R44), `16_cli_surface.md`, `17_matriz_aceitacao.md`.

---

## 0. Escopo

A investigação do `ai-memory` levantou 9 possíveis contribuições. Duas são **totalmente
descartadas** por decisão do mantenedor; as outras sete viram plano de implementação.

| Item | Tema | Destino |
|---|---|---|
| 2.1 | Renovação de shelf-life ponderada por uso | **Planejar** → `D154` |
| 2.2 | Consulta temporal `as_of` | **Planejar** → `D155` |
| 2.3 | Portão de evidência em propostas | **Planejar** → `D156` |
| 2.4 | Promoção de regras governadas (`AGENTS.md`) | **Planejar** → `D157` |
| 2.5 | Sugestão semântica de arestas/contradições | **Planejar** → `D158` |
| 2.6 | Invariantes transversais de retenção/reescrita | **Planejar** (transversal — §8) |
| 2.7 | Higiene de migração/backup/fail-closed | ❌ **Descartado** |
| 2.8 | Redação tipada de segredos | **Planejar** → `D159` |
| 2.9 | Fronteira de captura (marker file) | ❌ **Descartado** |

### Por que 2.7 e 2.9 saem

- **2.7 (migração prévia verificada, fail-closed schema-ahead, caminho único de leitura de
  config, `{provider,model,dim}` ao lado do vetor):** o `ai-memory` precisa disso porque tem
  **SQLite com migrations V01–V66**. O `knudge` não tem banco: o "schema" é `schema_version`
  por nota + `rebuild` double-buffer (D15/D27) e o meta-header de embeddings já carrega
  `(provider, model, dimension, similarity)` (D79/E11). É redundante; adicionar uma cerimônia
  de backup a um sistema cuja verdade são arquivos versionados em git não paga o custo.
- **2.9 (fronteira de captura: `.ai-memory.toml`, `ignore_paths`, allowlist):** o `knudge`
  **não captura atividade** — a escrita é explícita (duas fases, D26) e a borda mutável já é
  `hooks.pre_record`/`post_record` (D59). O desenho do `marker-file.md` só faz sentido se a
  premissa de captura automática entrar, o que é recusado pela tese ("o usuário é o LLM").

---

## 1. Tabela-resumo

| `Dxx` | Item | Verbo/superfície | Chave(s) de config novas | Novo módulo/arquivo | Esforço | Fase |
|---|---|---|---|---|---|---|
| D154 | Renovação por uso | `ask`/`rewind` (efeito) | `retention.renew_on_use` | `.idx/usage.jsonl` + `lifecycle/renew.rs` | M | 1 |
| D155 | `as_of` | `ask --as-of` | — | `retrieval/temporal.rs` | M | 2 |
| D156 | Portão de evidência | `maintenance learn/compact` | `proposals.gate`, `proposals.min_delta`, `proposals.enforce` | `health/gate.rs` | M | 2 |
| D157 | Promoção p/ `AGENTS.md` | `knowledge promote/*` | `rules.enabled`, `rules.max_promoted`, `rules.min_confidence` | `knowledge/promote.rs` + bloco em `git/agent_md.rs` | G | 3 |
| D158 | Sugestão semântica | `knowledge`/`learn`/`audit` | `suggestions.*` | estende `embeddings/semantic.rs` | M | 3 |
| D159 | Redação tipada | logs (efeito) | — | estende `logging.rs` | P | 1 |

Nenhuma adiciona chave canônica ao TOON (as **25** de D95/D98/D135/D142 permanecem). Todas as
chaves novas são de `config.toml`.

**Ordem de merge sugerida:** PR1 = D159; PR2 = D154; PR3 = D155; PR4 = D156; PR5 = D158;
PR6 = D157. Cada PR fecha com `make check` verde e atualiza `CHANGELOG.md`.

---

## 2. D154 — Renovação de shelf-life ponderada por uso

> **Origem:** `ai-memory` M8 — *access reinforcement*. Toda leitura (`memory_query`,
> `memory_read_page`, o *related-walk*, `memory_explore`) incrementa `access_count`/
> `last_accessed_at` de forma assíncrona, throttled (1×/página/operador/60 s) e fora da FTS.
> A página que o operador usa resiste ao decaimento. O invariante explícito é que reforço
> **só aumenta** retenção — nunca encurta.

### Diagnóstico (evidência no knudge)

A expiração é **puramente derivada de `created_at` + prazo da `classification`**
(`lifecycle/shelf_life.rs::derived_expiry`, D44/D135): uma nota `observational` valiosa
consultada toda semana expira em 30 dias do mesmo jeito. Não há sinal de uso algum — D152
removeu até a observabilidade de busca (`recall_stats`). O `ask`/`rewind`/`knowledge rank` são
read-only e não deixam rastro.

### Decisão proposta

1. Registrar **uso** num arquivo **derivado** (`.idx/usage.jsonl`), nunca em `eventos/` (para
   não poluir `diff`/`learn`/auditoria) e nunca na nota (a nota é a verdade; uso é índice).
2. `retention.renew_on_use` (bool, **default `false`** — identidade preservada no upgrade)
   liga a renovação. Quando ligado: `renewed_ms = max(created_ms, last_seen_ms)` e
   `expiry = renewed_ms + ttl_days`. Sempre no sentido que **só estende**.
3. Instrumentar apenas **leitura determinística**: hits devolvidos pelo `ask` e itens do
   `rewind`. `task show`/`knowledge map` ficam fora (evita amplificação de escrita).
4. Escrever **coalescido no fim da invocação** (mesmo padrão do flush de embeddings, D85/D131),
   sob a porta `Fs` (`write_atomic`), em lote único — sem hot-path write por hit.

### CLI

Sem flag nova. O efeito é comportamento de config (D94). `kd maintenance prune` passa a usar a
expiração efetiva; `kd rewind` pode exibir `fresh:` coerente. `--json` de `rewind` ganha
`renewed`/`last_seen` no manifest (informativo).

### Core (puro)

```rust
// lifecycle/usage.rs
/// Uso acumulado de uma nota (derivado, `[id asc]`).
pub struct Usage { pub id: String, pub hits: u32, pub last_seen_ms: i64 }
pub fn record_usage(existing: &[Usage], cited: &[String], now_ms: i64) -> Vec<Usage>;
```

- `lifecycle/shelf_life.rs`: `derived_expiry` ganha o `last_seen_ms: Option<i64>` (novo
  parâmetro) ou uma função irmã `effective_expiry(classification, created_ms, last_seen_ms,
  policy)`. Puro e testável.
- `store/purge.rs::purge_derived` (D84) remove o id de `.idx/usage.jsonl` — mesmo contrato de
  qualquer derivado.

### Contrato / config (`config/schema/keys.rs`)

```rust
KeySpec { key: "retention.renew_on_use", kind: Kind::Bool, default: Default::Bool(false) },
```

### Determinismo

Arquivo ordenado por `id`; `hits` soma, `last_seen_ms = max`; ausência = nunca citada. Nada de
relógio real no domínio (`Clock`).

### Erros

Falha ao ler/gravar o derivado ⇒ `warn` + segue sem renovação (R33); `strict` promove a erro
(D94). Uso órfão (id removido) é podado no próximo purge.

### Testes

- Unidade: nota citada sobrevive a `created_at + prazo`; nota não citada expira; renovação
  nunca antecipa expiração.
- Propriedade: `record_usage` idempotente por `(id, now)` e comutativa entre lotes.
- Integração: `purge_derived` remove o id; rebuild reconstrói índice sem uso (documentado).
- Golden: `rewind --json` com e sem renovação.

### Aceite

Com `renew_on_use=false`, a saída é **byte-idêntica** à de hoje. Com `true`, uma nota
`observational` citada nos últimos `observational_days` sobrevive além do prazo derivado, e o
`prune` a considera viva.

**Revisa:** D44/D135 (shelf-life derivado). **Risco:** escrita por invocação; mitigada pelo
coalescido e pelo teto implícito de `.idx`. **Dúvida aberta:** renovação total (reset) ×
ponderada (`renew_weight`) — ver §11.

---

## 3. D155 — Consulta temporal `as_of`

> **Origem:** `ai-memory` `docs/temporal.md` — janelas de validade por versão (V56/V58/V62) e
> `memory_query { as_of }` com streams de entidade + FTS filtrada. O próprio doc é honesto:
> "seleciona a versão histórica; não reproduz o ranking de T". O `knudge` tem a matéria-prima
> para ir **além** — o pipeline é determinístico.

### Diagnóstico (evidência no knudge)

`eventos/events.jsonl` guarda `{op, note_id?, at, data?}` (`store/events/event.rs`) com
`write/update/supersede/forget/restore/outcome/link/learn/…`; a cadeia
`replaces`/`superseded_by` é caminhável (D46/D48). O `ask` só enxerga o corpus ativo.

### Decisão proposta

1. `kd ask "<q>" --as-of <RFC3339|YYYY-MM-DD>` reconstrói o **conjunto ativo em T** a partir
   dos eventos + estado atual + cadeia de supersessão, e roda o pipeline existente **sobre o
   subconjunto**. Como o knudge é determinístico (BM25 + âncoras + RRF, D35/D81/D124), o
   ranking em T é **reproduzível** — vantagem sobre o `ai-memory`.
2. Uma nota está ativa em T se: (a) `write`/`created_at <= T`; (b) **não** houve
   `supersede`/`forget` sobre ela com `at <= T` (respeitando `restore`). `update` altera a
   versão, mas mantém a nota ativa.
3. Notas cujo conteúdo já foi **purgado** (D48/D84) não são reconstruíveis ⇒ `warning[]`
   explícito e hit omitido (degradação graciosa, R33). Documentar como borda em
   `DIVERGENCES.md`.
4. `as_of` compõe com os **filtros de corpus** (D143/D144) e exige escopo ou `--universe`
   quando a varredura for total.

### CLI

```
kd ask "<q>" --as-of 2026-07-01
kd ask "<q>" --as-of 2026-07-01T00:00:00Z
```

- Texto: banner `as_of=<T>` no cabeçalho e cada hit seguido de `historical`.
- `--json`: campo `as_of` no envelope e `historical: true` por hit. O `why` **não muda**
  (fechado, D39/D121).
- T no futuro ⇒ `invalid_input` (exit 2). T antes do primeiro evento ⇒ `[no_results]` (D152).

### Core (puro)

```rust
// retrieval/temporal.rs
/// Conjunto de ids ativos em `as_of_ms` (ordem canônica), a partir do log de eventos.
/// `Ok` com `warnings` para notas purgadas/irreconstruíveis.
pub fn active_at(events: &[Event], as_of_ms: i64, now: &ActiveCorpus) -> (Vec<String>, Vec<String>);
```

`RecallQuery` ganha `as_of_ms: Option<i64>`; o pipeline filtra o corpus **antes** do BM25, de
modo que a IDF é recomputada no subconjunto (determinístico).

### Contrato / config

Nenhuma chave nova. `--as-of` é flag de `ask`.

### Determinismo

Reconstrução em `(at, id)` — nunca por mtime; o conjunto é ordenado por id. Sem `Clock` real no
core.

### Erros

Evento malformado ⇒ `warn` e ignora a linha (leitura tolerante, D16–D18). `as_of` inválido ⇒
`invalid_input`.

### Testes

- Golden: cenário `postgres`→`sqlite` — `ask` atual não acha; `ask --as-of` acha a versão de
  junho.
- Propriedade: `ask --as-of <agora>` == `ask` (byte-a-byte, dentro da mesma granularidade ms).
- Fronteira: nota purgada gera `warnings[]` sem quebrar; T no futuro é erro.

### Aceite

`kd ask "postgres" --as-of 2026-07-01` retorna a nota que carregava o termo em T, e
`--as-of now` é idêntico ao `ask` atual. **Depende de:** D154 (para o `rewind` exibir uso
histórico, opcional). **Risco:** eventos rotacionados (`events-NNNN.jsonl`) exigem ler todos os
segmentos; já suportado (`EventLog::segments`).

---

## 4. D156 — Portão de evidência em propostas

> **Origem:** `ai-memory` `docs/auto-improvement-loop.md` / `auto-improve-eval-gates.md` — toda
> proposta validada (schema, confiança, teto, **eval gate**) antes de stage/apply; o gate é um
> executável do operador com contrato JSON `{score_before, score_after, passed}` +
> `min_delta`; um *rejection buffer* evita repetir tentativas ruins.

### Diagnóstico (evidência no knudge)

`learn`/`compact`/`prune` **só propõem** (D47) e o aceite é humano/agente. Já existe um
mecanismo de borda que bloqueia/muta a escrita: `hooks.pre_record` (`commands/write_cmd.rs`,
D59). Falta exatamente o **contrato de evidência** e a ligação com o catálogo
`validators.toml` (D99) — hoje validators rodam só no `task close` (D54/D55).

### Decisão proposta

1. Estender `validators.toml` com um tipo **gate** (validador de conteúdo), contrato JSON:
   - stdin: `{ "op": "create|merge|supersede", "before": {...}, "after": {...} }`
   - stdout: `{ "passed": bool, "score_before": f64, "score_after": f64 }`
2. Config `[proposals]`:
   - `gate = ["nome", ...]` (vazio = desligado; **default**),
   - `min_delta = 0.0`,
   - `enforce = false` (quando `true`, o gate roda no `pre-record`/`pre-prune` e **bloqueia**).
3. `kd maintenance learn`/`compact` ganham `--verify` (**read-only**): roda o gate contra cada
   proposta e anexa `passed`/`score_before`/`score_after` às propostas. Nada é escrito.
4. Aplicação continua explícita (`kd write`/`merge`/`forget`); com `enforce=true`, o gate roda
   antes e bloqueia com `Error::conflict` (exit 4), coerente com o hook bloqueante atual.
5. *Rejection buffer* derivado `.idx/gate.jsonl` (hash da proposta → veredito) para não
   reprocessar o que já falhou; purgado em D84.

### CLI

```
kd maintenance learn --universe --verify
kd maintenance compact --scope <ID> --verify
```

Saída pipe: `strategy|keep|ids|score|gate:passed|before|after`.

### Core (puro)

```rust
// health/gate.rs
pub struct GateOutcome { pub passed: bool, pub score_before: f64, pub score_after: f64 }
/// Avalia um veredito do gate contra `min_delta` (puro; o processo é da borda).
pub fn accept(outcome: &GateOutcome, min_delta: f64) -> bool;
```

A execução do comando fica em `health/validator`/borda (sem shell, timeout + kill de grupo,
como `ProcessHookRunner`, D59/R12).

### Contrato / config

Chaves novas de config apenas. `validators.toml` (D99) ganha o campo `kind = "gate"` — é
**conteúdo**, não config (permanece em `.knudge/`).

### Determinismo

Verificação em lote, ordem canônica dos ids; veredito cacheado por hash de conteúdo (não por
tempo).

### Erros

Comando ausente/timeout/JSON inválido ⇒ `warn` + gate tratado como não-aprovado **sem** travar
a leitura (R33); `strict` promove a erro.

### Testes

- Unidade: `accept` respeita `min_delta`; JSON malformado não aprova.
- Integração: `--verify` não altera `notas/`/`eventos/` (diff vazio); com `enforce=true`,
  `write` bloqueado sai 4.
- Golden: linha de proposta com veredito.

### Aceite

`--verify` é estritamente read-only; `enforce=true` impede a gravação quando o gate reprova, e
`enforce=false` (default) preserva o comportamento atual. **Reusa:** `validators.toml`
(D99), `ProcessHookRunner` (D59), `pre-record` (D47). **Risco:** um gate lento no caminho de
escrita; mitigado por timeout curto e por ser opt-in.

---

## 5. D157 — Promoção de conhecimento para regras governadas (`AGENTS.md`)

> **Origem:** `ai-memory` `docs/design-rules-promotion.md`. As duas ideias fortes: (1)
> **`AGENTS.md` é orçamento escasso** (~150–200 instruções; >200 linhas ⇒ *context rot* e queda
> >30% de adesão); (2) promoção é **subtrativa por default** (evict-to-admit), **nunca
> auto-edita**, com proveniência por linha e rebaixamento reversível.

### Diagnóstico (evidência no knudge)

O `knudge` **já possui bloco gerenciado** em `AGENTS.md` (`git/agent_md.rs`, marcadores
`<!-- knudge:start -->`/`end`, D60) — mas ele é o protocolo **estático** de uso, não populado do
corpus. Existem notas `meta`/`decision` com confiança derivada alta (D87) que nunca chegam ao
contexto always-on.

### Decisão proposta

1. Novo sub-bloco gerenciado dentro do bloco do knudge:
   `<!-- knudge:rules:start -->` … `<!-- knudge:rules:end -->`, escrito **só** por comando
   explícito. Conteúdo do usuário fora dos marcadores é intocado (reusa `git/block.rs::upsert`).
2. Família `kd knowledge promote`:
   - `recommend` — read-only; candidatos ranqueados com `regra | id-fonte | confiança | por quê`.
   - `approve <id>` — promove uma linha; se o bloco estiver no teto, **recusa** e nomeia o
     candidato a sair (decisão do usuário; nada é removido em silêncio).
   - `edit <id>` / `remove <id>` / `list`.
3. Critérios de elegibilidade: `type=meta`/`decision`, `classification=foundational`,
   confiança derivada ≥ `rules.min_confidence`, statement imperativo e curto (≤ 120, já é o
   limite), ampla cobertura de âncoras e **sem** `contradicts` aberto.
4. Teto rígido `rules.max_promoted` (default **15**) — promoção é admission-controlled.
5. Cada linha carrega trailer de proveniência (`— <id> (conf 0.8)`), para auditoria.
6. **Desligado por default** (`rules.enabled=false`): sem a flag, a superfície some.

### CLI

```
kd knowledge promote recommend [--universe]
kd knowledge promote approve <ID>
kd knowledge promote edit <ID> --summary "..."
kd knowledge promote remove <ID>
kd knowledge promote list
```

`approve`/`edit`/`remove` mutam `AGENTS.md` e emitem o diff; `recommend`/`list` são leitura.

### Core (puro)

```rust
// knowledge/promote.rs
pub struct Candidate { pub id: String, pub statement: String, pub confidence: f64, pub reason: String }
/// Ranqueia candidatos (confiança × breadth × frescor), determinístico (conf desc, id asc).
pub fn recommend(notes: &[Note], graph: &Graph, index: &Index, cfg: &RulesPolicy) -> Vec<Candidate>;
/// Bloco de regras a partir dos ids aprovados + proveniência.
pub fn render_block(approved: &[Candidate]) -> String;
```

### Contrato / config

```rust
KeySpec { key: "rules.enabled", kind: Kind::Bool, default: Default::Bool(false) },
KeySpec { key: "rules.max_promoted", kind: Kind::Int, default: Default::Int(15) },
KeySpec { key: "rules.min_confidence", kind: Kind::Float, default: Default::Float(0.7) },
```

A `meta`-nota de origem **não** é apagada (`remove` só tira a linha do `AGENTS.md`).

### Determinismo

Ranking `(confidence desc, id asc)`; `render_block` estável e idempotente.

### Erros

`approve` no teto ⇒ `conflict` (4) com o candidato a sair; id inexistente ⇒ `not_found` (3);
bloco corrompido ⇒ `schema` (8) com orientação (não sobrescreve).

### Testes

- Unidade: teto recusa; `remove` restaura o `AGENTS.md` original; `recommend` ordenado.
- Golden: bloco com N linhas + proveniência; reexecutar não duplica (idempotente).
- Integração: escrita do usuário **fora** dos marcadores permanece byte-a-byte.

### Aceite

`recommend` é read-only; nenhum comando além de `approve`/`edit`/`remove` toca o `AGENTS.md`;
com `rules.enabled=false`, nada muda. **Reusa:** D60 (`AGENTS.md`), D47 (só propõe).

---

## 6. D158 — Sugestão semântica de arestas/contradições

> **Origem:** `ai-memory` `docs/design-memory-aging.md` A5 + `docs/typed-edges.md`. Pares na
> banda cosseno **0,4–0,75** ("mesmo tópico, não quase-duplicata") viram candidatos a
> `contradicts` resolvido por timestamp — **advisory**, nunca cria/edita/deleta.

### Diagnóstico (evidência no knudge)

Existem sugestões **textuais** conservadoras (`graph/extract.rs` → `.idx/suggestions.jsonl`,
D49/D50/D84) e dedup lexical (Dice 0.75/0.92). Falta o sinal **semântico** de "lacuna de
grafo": vizinhos no espaço vetorial que não são quase-duplicata, não compartilham âncora e não
têm aresta — candidatos a `contradicts`/`extends`/`references`.

### Decisão proposta

1. Estender `embeddings/semantic.rs` com um classificador de banda:
   - `score >= duplicate_threshold` → **merge** (comportamento atual);
   - `contradiction_low <= score < contradiction_high` (default `0.4`–`0.75`) **e** sem aresta
     pré-existente → sugestão `contradicts` (timestamp do mais novo no `why`);
   - similaridade moderada + âncora/tag compartilhada sem aresta → sugestão `link`/`extends`.
2. Emitir via `SuggestionStore` (`.idx/suggestions.jsonl`, D50) — **nunca** vira aresta (D49).
3. Superfície: `kd audit`/`doctor` (arestas sugeridas faltantes, já existe a checagem),
   `kd maintenance learn` (propostas `link`) e o hint MCP `missing_link`.
4. Purgado por `purge_derived` (D84); rebuild reconstrói (D27). Zero-LLM.

### CLI

Sem flag nova obrigatória; `kd knowledge map --semantic` e `kd maintenance learn` passam a
mostrar o novo tipo de sugestão. `suggestions.enabled` (default `true` quando há índice
vetorial) controla o custo.

### Core (puro)

```rust
// embeddings/semantic.rs
pub enum Relation { Duplicate, Contradiction, Link }
/// Classifica o par (determinístico; sem LLM).
pub fn classify_pair(score: f64, has_edge: bool, shares_anchor: bool, cfg: &SuggestionPolicy) -> Option<Relation>;
```

### Contrato / config

```rust
KeySpec { key: "suggestions.enabled", kind: Kind::Bool, default: Default::Bool(true) },
KeySpec { key: "suggestions.contradiction_low", kind: Kind::Float, default: Default::Float(0.4) },
KeySpec { key: "suggestions.contradiction_high", kind: Kind::Float, default: Default::Float(0.75) },
```

### Determinismo

Pares em ordem canônica (`id asc`); sem pareamento O(N²) — usa o índice vetorial já ranqueado.

### Erros

Sem índice ⇒ canal ausente (sem sugestões), nunca erro; índice stale ⇒ `warn` (D79/D83).

### Testes

- Unidade: fronteiras da banda (`<0.4`, `0.4`, `0.75`, `>0.75`); par já ligado não sugere.
- Propriedade: sugestão nunca cria aresta nem altera nota.
- Integração: `audit` lista a sugestão; `purge_derived` remove quando a origem cai.

### Aceite

Sugestões aparecem em `.idx/suggestions.jsonl` e nas superfícies de auditoria, sem tocar
`notas/`; apagar `.idx/` e reconstruir reproduz as mesmas sugestões.

---

## 7. D159 — Redação tipada de segredos

> **Origem:** `ai-memory` P1 (`design-hindsight-borrowings.md`) — `[REDACTED:<kind>]`
> (`github_token`, `aws_key`, `jwt`) em vez do marcador anônimo, para que o leitor saiba *que
> tipo* de segredo estava ali sem vazá-lo.

### Diagnóstico (evidência no knudge)

`logging.rs::Redactor` substitui valores de `[secrets]` e o resto da linha após chaves
sensíveis por um `REDACTED = "[REDACTED]"` constante (R22). Não há distinção de tipo.

### Decisão proposta

1. Mapear as `SENSITIVE_KEYS` para um rótulo tipado: `authorization`, `api_key`, `token`,
   `password`, `secret`, `bearer` (default `custom` para chave desconhecida).
2. Segredos literais de `[secrets]` → `[REDACTED:secret]`.
3. **Sem impacto em contrato durável:** o `knudge` não persiste corpo redigido (a redação é
   só do *layer* de log, R22) — ao contrário do `ai-memory`, isto é mudança de log, não de
   formato de nota. Mesmo assim, tratar como mudança de contrato observável (golden).

### CLI / core

Sem superfície nova; `Redactor::redact` muda a string produzida.

### Erros

Nenhum; allowlist preservada (nunca redige o que não é segredo conhecido).

### Testes

- Unidade: cada chave sensível gera o rótulo correto; valor nunca aparece.
- Golden: linha de log com `Authorization`/`token`/literal.

### Aceite

Nenhum valor sensível nos logs; o rótulo identifica o tipo; os testes de "não falso-positivo"
continuam válidos (usa prefixo `[REDACTED`).

---

## 8. Invariantes transversais de retenção/reescrita (item 2.6)

Não é feature nova: é o **contrato de segurança** que D154–D158 devem respeitar, com teste. Vira
uma seção da revisão técnica (R45) e um checklist no DoD de cada PR.

| # | Invariante | Como travar |
|---|---|---|
| 1 | **Default identidade**: feature nova é no-op byte-a-byte até o usuário ligar (sem "penhasco" no upgrade), inclusive sem criar derivados. | golden antes/depois + `invariants::default_reads_leave_notes_and_derived_untouched` |
| 2 | **Renovação só estende**: acesso nunca encurta retenção (direção segura). | teste de monotonicidade em D154 (`shelf_life`/`usage`) |
| 3 | **Supersessão vence evidência**: contagem/confiança só sombreia ranking, nunca decide se a correção entra. | D46/D48 já traval; `invariants::supersession_beats_usage` |
| 4 | **Acesso ≠ evidência**: "ainda é usado" e "ainda é verdade" são eixos separados. | D154 vs D87 (confiança derivada) — `usage` ausente de `confidence`/`retrieval` |
| 5 | **Transformação não deleta a fonte**: supersessão/merge/decay/compact só propõem; `purge_derived` (D84) toca só `.idx/`. **Exceção única:** `forget --purge` explícito, após tombstone + retenção, com detach (D32/D48/D84). | teste `restore`/`get --history`; R45 revisado |
| 6 | **Toda varredura executada deixa relatório**: `warnings[]`/`--json`, nada silencioso (R33/D47). | grep de `warnings` nos comandos; `sweep_residues` sem chamador hoje |
| 7 | **Leitura nunca escreve nota**: só derivados (`.idx/`). | `invariants::default_reads_leave_notes_and_derived_untouched` + `renewal_credits_usage_but_never_writes_notes` |

Travado de ponta a ponta em [`crates/knudge-cli/tests/invariants.rs`](../../crates/knudge-cli/tests/invariants.rs).

---

## 9. Fases de execução

| Fase | PRs | Depende de | Fecha com |
|---|---|---|---|
| **1 — baratos** | D159 (redação tipada) → D154 (renovação por uso) | — | `make check` |
| **2 — temporal/evidência** | D155 (`as_of`) → D156 (gate) | D154 (opcional) | `make check` |
| **3 — curadoria** | D158 (sugestões) → D157 (promoção) | D158 → D157 | `make check` |

**Paralelizável:** D159 × (qualquer); D155 × D156 (arquivos distintos).
**Acoplamentos:** D157 depende de D158 (candidatos melhores com contradição detectada);
D154 alimenta o `fresh:` de D155.
**Transversal:** §8 em todas as fases.

Cada PR atualiza, além do código: `MODULE.md` do crate tocado, `CHANGELOG.md`,
`docs/` do verbo e a matriz em `plan/implementation/17_matriz_aceitacao.md`. Se tocar borda,
linha em `DIVERGENCES.md` com o teste que a trava (D155 purgado, D154 uso derivado perdido).

---

## 10. Fora de escopo (recusado por premissa)

- **Captura automática por lifecycle hooks** (prompts/tool calls) — recusada: viola o protocolo
  de duas fases (D26) e a tese "o usuário é o LLM".
- **LLM interno** (consolidação, `dream`, reranker, `answer`) — recusado por R16/R43.
- **SQLite/FTS5, servidor HTTP, web UI, multi-user, `tokio full`** — recusados em `05`, §3.
- **Cross-project / workspace / mensagens / handoffs claim-once** — D136 (um agente, sem posse).
- **`ai`-style tiers (`working/episodic/semantic/procedural`, `pinned`, `_slots/`)** — duplicam
  `classification` + `status` (D44/D48).
- **`expires_at`/`not_before` no frontmatter** — removidos de propósito (D135).

---

## 11. Perguntas abertas (para o mantenedor)

1. **D154 — renovação total × ponderada.** Reset completo (`renewed_ms = max(created,
   last_seen)`) é simples e previsível, mas uma única citação compra uma janela inteira.
   Introduzir `retention.renew_weight ∈ [0,1]` (0,5 = metade do intervalo) ou manter reset?
2. **D154 — escopo de instrumentação.** Incluir `knowledge rank` e o *related-walk* do `ask
   --around`, ou só os hits do `ask`/`rewind` (menos amplificação de escrita)?
3. **D155 — saída.** O `historical` deve aparecer no `why` (hoje fechado, D39) ou num campo
   separado do `--json` (proposta atual)? A segunda preserva o contrato do pipe.
4. **D156 — enforcement.** `proposals.enforce=true` no `pre-record` reusa `Error::conflict`
   (exit 4), ou merece um `ErrorKind` próprio? (R30 diz que o `code` é contrato de máquina.)
5. **D157 — teto default.** ~15 regras / ~40 linhas é o teto certo, ou o knudge quer algo mais
   rígido (ex.: 10)?
6. **D158 — banda.** Fixar 0,4–0,75 (herdado do `ai-memory`) ou calibrar na bancada `bench/`
   antes de fechar a decisão?

---

## 12. Referências

- Investigação de origem: `TMP/ai-memory/docs/{ARCHITECTURE,design-memory-aging,
  auto-improvement-loop,design-rules-promotion,typed-edges,temporal,admission-webhooks}.md`.
- Estado atual do `knudge`: `ARCHITECTURE.md`, `plan/03_decisoes-fechadas.md` (D01–D153),
  `plan/implementation/14_revisao_tecnica.md` (R01–R44).
- Superfície: `plan/implementation/16_cli_surface.md`; aceite: `17_matriz_aceitacao.md`.
