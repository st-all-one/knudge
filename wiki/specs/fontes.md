# Fontes de corpus

Como o knudge escolhe **de onde** lê as notas, como consulta a memória de **outro** agente sem
aprender o seu schema interno, e como essa memória continua a ser **uma só verdade**. É a porta de
entrada do knudge como **eixo único de memória, agent-agnostic** (D218).

- Código: `crates/knudge-core/src/corpus/source.rs`, `crates/knudge-core/src/store/note.rs`
- CLI: `crates/knudge-cli/src/commands/ai.rs`
- Decisões: D218 (revisa D207/D214)

## Princípio

> O knudge **indexa** (derivado); o agente **possui** a fonte.

O contrato entre os dois é o **documento** — uma nota no schema do knudge — e **nunca** o schema
interno do agente. Por isso o knudge lê JSONL de notas (formato que ele próprio define), e não o log
cru de um agente qualquer.

## As duas fontes

| `Source` | Onde lê | Papel |
|---|---|---|
| `NotesDir` (default) | `notas/<tipo>/<id>.md` | a verdade do knudge, inalterada |
| `Jsonl(path)` | um ficheiro JSONL, **read-only** | o store de um agente externo |

```rust
let source = Source::jsonl(".agents/memory/notes.jsonl");
let (corpus, warnings) = source.load(&store)?;   // notas + índice + grafo
```

`Source::NotesDir` é o default: quem usa só `notas/` não nota diferença. `Source::load_notes`
devolve `Loaded = (Vec<Note>, Vec<String>)` — as notas e os avisos da leitura.

## O codec JSONL

Uma linha por nota, com o JSON canónico do knudge (chaves ordenadas, `null` rejeitado — D05):

```json
{"frontmatter":{"id":"fact_01abc234","type":"fact","statement":"…","created_at":1700000000000,"schema_version":2,"provenance":{"agent":"ktu","activity":"write"}},"body":"…"}
```

- `Note::to_jsonl` emite; `Note::from_jsonl` reconstrói e **valida** o schema (`Frontmatter::from_value`
  + `validate`).
- **Dedup-on-read por `id`** (D26): a última linha vence — o store pode ser append-only sem índice.
- **Linha malformada = *skip + warning*** (D18): a linha má nunca derruba a leitura; o aviso diz
  qual a linha e porquê.
- Ficheiro ausente ⇒ corpus **vazio** (sem erro nem aviso).
- A ordem é determinística (`id` asc), pelo que o rebuild do índice devolve o mesmo recall.

## `kd ai` — a área de consulta

```
kd ai [memory|session-log] [QUERY...] [--source PATH] [--limit N]
```

| Aspecto | Contrato |
|---|---|
| Área | posicional: `memory` (default) ou `session-log`; valor fora do conjunto ⇒ exit 2 com a lista |
| Sem query | **lista** a fonte (`id\|tipo\|statement`), ordem determinística |
| Com query | **recall** sobre a fonte (mesma cascata do `ask`) |
| Universo | **inteiro** — a fonte é a memória do agente, não há separação conhecimento/trabalho |
| Estados | visíveis por default (D43) |
| Fonte | `--source PATH`; default `.agents/memory/notes.jsonl` / `.agents/memory/session-log.jsonl` |
| Saída | pipe `id\|statement\|score`; `--json` = `{area, source, query, data}` + `warnings[]` |

A área é **read-only**: não há `--add` nem escrita na fonte. Escrever na memória do agente é do
agente; o `kd` só a indexa.

## Filtros de proveniência

`kd ask` (e `kd ask --rank`) aceitam `--agent` e `--activity`, por **igualdade** com
`provenance.agent`/`provenance.activity` (D207). São listas de seleção como as restantes (D210):
repetição ou vírgula, e o `--params` transporta-os.

```bash
kd ask --agent ktu "rate limit do gateway"      # o que o ktu registou
kd ask --activity derive --type meta "sessao"   # o que foi projetado de logs
```

`Meta`/`Filter` carregam `agent`/`activity` e o codec do `.idx/` persiste-os; um índice anterior a
D218 lê `None` e continua válido (D15). A filtragem é **estrutural** — acontece antes da estatística,
como todos os filtros (D41/D53).

## O contrato de projeção (o log do agente)

O log de sessão **não** é um corpus de notas: é uma sequência de eventos com semântica própria. O
knudge **não** o conhece. Quem o projeta é o **agente**:

1. o agente lê o seu próprio log (o formato é dele);
2. emite **notas derivadas** no schema do knudge — uma afirmação por nota, `anchors` a apontar o
   código — com `provenance.activity = derive` e `provenance.agent = <o agente>`;
3. escreve-as no seu store JSONL (ex.: `.agents/memory/session-log.jsonl`);
4. o `kd ai session-log` consulta-as como qualquer outra fonte.

Regras: **read-only** para o knudge, **derivada** (a nota nunca substitui o log), **uma só verdade**
(o log continua a ser do agente — I3 do lado do agente). Assim se responde "o que fizemos sobre X?"
por **significado + âncoras + grafo**, e não por leitura sequencial de logs.

## Multi-agente

Nada aqui é novo: o lock por ficheiro-alvo, os ids por conteúdo (dedup) e o `merge=union` dos
eventos já tornam **vários** agentes seguros. A `Source` só acrescenta a **porta** para os que não
escrevem `.md`, e os filtros de proveniência dão a **consulta** por produtor.

O **registo de agentes** (`.knudge/agents.toml`) fica adiado até haver ≥2 consumidores (YAGNI).

## Testes

| Comportamento | Teste |
|---|---|
| JSONL deriva o mesmo corpus que `notas/` (golden de índice/grafo) | `corpus::tests::jsonl_source_reads_the_same_corpus_as_notas_dir` |
| Dedup-on-read (última linha vence) e ordem determinística | `corpus::tests::jsonl_source_dedups_on_read_last_line_wins` |
| Linha malformada = skip + warning; ficheiro ausente = vazio | `corpus::tests::jsonl_source_skips_malformed_lines_with_warning`, `jsonl_source_absent_file_is_an_empty_corpus` |
| Codec `to_jsonl`/`from_jsonl` (round-trip + rejeições) | `store::tests::note::note_round_trips_through_one_jsonl_line`, `jsonl_line_rejects_non_records` |
| Filtros de proveniência (match e exclusão sem proveniência) | `retrieval::tests::filter::provenance_filters_*` |
| `kd ai` (lista, recall, áreas, `--source`, tolerância, área inválida) | `crates/knudge-cli/tests/ai.rs` |
| `kd ask --agent`/`--activity` (+ vírgula, `--params`, `--rank`) | `crates/knudge-cli/tests/ai.rs::ask_filters_by_agent_and_activity` |
