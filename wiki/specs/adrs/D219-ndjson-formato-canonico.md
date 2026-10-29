# D219 — NDJSON/JSONL é o formato canônico de escrita; JSON-LD fica para export futuro

- **Status:** Aceita
- **Categoria:** Eixo de memória / formato canônico (D110/D218)

## Contexto

A memória de agentes v2 (D218/D220) precisa de um formato que (a) permita **acrescentar** uma nota
sem reescrever o ficheiro, (b) sobreviva a um escritor interrompido no meio de uma linha, e (c) seja
**diff-friendly** em git (`merge=union`, D31) e legível por qualquer consumidor sem biblioteca
própria. Os candidatos eram: NDJSON/JSONL (uma nota por linha), um array JSON único e JSON-LD.

Um array JSON obriga a reescrever e re-parsear o documento inteiro a cada nota e transforma uma
linha truncada em documento inválido. JSON-LD traz `@context`/`@id`/IRI expansion e acoplaria o
**formato de armazenamento** a uma ontologia externa: o knudge passaria a depender do vocabulário
de contexto e o schema canônico (D04/D13) deixaria de ser a única autoridade de bytes.

## Decisão

**O formato canônico de escrita é NDJSON/JSONL.** Cada linha é uma nota completa no JSON canônico do
knudge:

```
{"frontmatter": {...}, "body": "..."}
```

- O codec é `Note::to_jsonl` / `Note::from_jsonl` (chaves ordenadas, `null` rejeitado — D13/D17).
- A escrita é **append-only**: o escritor acrescenta linhas ao seu shard e nunca reescreve o
  ficheiro inteiro no caminho quente (o shard é por escritor — D220).
- A leitura é **tolerante a linha parcial**: uma linha truncada no fim (escritor em curso ou
  crash) é ignorada com **warning**, nunca derruba a leitura (D18). Linhas completas anteriores
  valem.
- **JSON-LD é rejeitado como formato canônico.** Não há `@context`, `@id` nem expansão de IRIs no
  store. Se um dia houver interoperabilidade com o mundo RDF, JSON-LD será um **export derivado**
  (leitura, nunca escrita) — a fonte continua NDJSON.

O knudge continua a **indexar** (derivado) e o agente a **possuir** a fonte (D218): o formato é o
documento do knudge, não um grafo semântico de terceiros.

## Consequências

- Acréscimo é O(1) e não toca nas linhas anteriores; um crash só pode danificar a **última** linha.
- O ficheiro entra em git com `merge=union` (D31) e linhas de escritores distintos não competem.
- Qualquer linguagem lê a fonte com um `split('\n')` + parser JSON; nenhum consumidor precisa de um
  processador JSON-LD.
- O `id` continua **content-addressed** (D01) e não depende de IRIs nem de contexto: o formato não
  muda a identidade da nota.
- Perde-se, por agora, validação semântica de contexto RDF; um export JSON-LD futuro terá de
  derivá-la do schema e das arestas, não do layout do store.
- `DIVERGENCES.md` (#119) trava o comportamento de linha parcial/malformada.
