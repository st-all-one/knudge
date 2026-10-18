# D218 — Fonte JSONL read-only (`Source`) + área `kd ai`

- **Status:** Aceita
- **Categoria:** Eixo de memória / integração de agentes (D207/D214)

## Contexto

O knudge já é, na prática, o eixo de memória de um projeto: `notas/` é a verdade, o índice/grafo
são derivados, o retrieval funde BM25 + âncoras + grafo + vetorial por RRF, e a proveniência
(`provenance.agent`/`activity`, D207) já distingue **quem** produziu cada nota. O que faltava não
era motor — era **porta** e **contrato**.

O caso concreto: um agente (o `ktu`, hoje) mantém a memória num **JSONL** de notas que já usa o
schema do knudge (`Note`/`Frontmatter`), mas **não** em `notas/*.md`. A leitura desse store estava
**reimplementada** fora do knudge, e o `kd` não conseguia consultá-lo. Sem uma fonte partilhada, a
memória continuava a ser **do agente** em vez de ser **do projeto/equipa** — e cada agente novo
teria de reimplementar tudo outra vez.

Dois riscos a evitar: (1) **acoplar** o knudge ao schema interno de cada agente (o knudge passaria
a depender do formato de todos); (2) criar uma **segunda verdade** (ingerir o log do agente como
nota canónica violaria a regra "uma só verdade").

## Decisão

**O corpus passa a ser lido de uma fonte explícita** (`corpus::Source`):

- `Source::NotesDir` — o default: `notas/<tipo>/<id>.md` (a verdade do knudge), inalterado.
- `Source::Jsonl(path)` — um store de linhas `{"frontmatter": {…}, "body": "…"}`, com o codec novo
  `Note::to_jsonl`/`Note::from_jsonl` (JSON canónico do knudge: chaves ordenadas, `null` rejeitado).
  A leitura é **dedup-on-read por `id`** (a última linha vence, D26) e linha malformada é
  *skip + warning* (D18) — nunca derruba a leitura.

**A fonte é read-only.** O knudge **indexa** (derivado); o agente **possui** a fonte. O contrato é o
**documento** (o schema do knudge), **nunca** o schema interno do agente — por isso o `session-log`
entra apenas por **projeção do agente** em notas derivadas (`provenance.activity=derive`), e não
como um domínio novo do knudge.

**Nova área fina `kd ai`** sobre a fonte:

```
kd ai [memory|session-log] [QUERY...] [--source PATH] [--limit N]
```

- a área é **posicional** (`memory` | `session-log`; default `memory`) e o default é **consulta**
  (D130: nunca sai vazio em silêncio);
- **sem** query → **lista** a fonte (`id|tipo|statement`, ordem determinística por `id`);
- **com** query → **recall** sobre a fonte (mesma cascata do `ask`), com o universo **inteiro**
  (a fonte é a memória do agente — não há separação conhecimento/trabalho) e os estados visíveis
  por default (D43);
- `--source PATH` sobrescreve o caminho convencional; os defaults são
  **`.agents/memory/notes.jsonl`** (memória do agente) e **`.agents/memory/session-log.jsonl`**
  (projeção do log de sessão).

**Filtros de proveniência** em `kd ask` (e em `kd ask --rank`): `--agent` / `--activity`, por
igualdade com `provenance.agent` / `provenance.activity`, repetíveis ou com vírgula (D210) e
transportados pelo `--params` (D147). `Meta`/`Filter` ganham os campos e o codec do `.idx/`
persiste-os; um índice anterior a esta decisão lê `None` e continua válido (D15).

**Não há motor novo.** São portas e filtros sobre o que já existia. O **registo de agentes**
(`.knudge/agents.toml`) fica **adiado** até haver **≥2** consumidores da fonte (YAGNI): com um só,
a superfície não se paga.

## Consequências

- A memória deixa de ser **do agente** e passa a ser **do projeto/equipa**: qualquer agente que
  exponha um JSONL `{frontmatter, body}` é consultável pelo `kd` sem código novo, e o mesmo `kd ask`
  serve todos.
- O `ktu-memory` deixa de reimplementar a leitura do JSONL (passa a usar `Source`/`Note::from_jsonl`).
- **Uma só verdade** preservada: o log do agente continua a ser dele; o knudge lê-o/projeta-o
  read-only e indexa o resultado como derivado.
- Nada muda para quem usa só `notas/`: `Source::NotesDir` é o default e o comportamento é idêntico.
- A ordem do `Source::Jsonl` é determinística (ordenação por `id`), pelo que o rebuild do índice
  devolve o mesmo recall.

Revisa/complementa D207 (proveniência) e D214 (fachada/núcleo publicável). Foi 0.6.2.
