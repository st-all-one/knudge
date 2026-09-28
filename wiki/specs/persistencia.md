# Persistência, atomicidade e concorrência

Como o knudge grava, lê e repara dados sem perder conhecimento em crash ou concorrência entre o
CLI (`kd`) e o MCP (`knudge-mcp`).

- Código: `crates/knudge-core/src/store/`, `crates/knudge-core/src/corpus/`
- Decisões: D20–D28, D84, D150, D160
- Bordas: [`DIVERGENCES.md`](DIVERGENCES.md) (lock, atomicidade, ordem)

## Hierarquia de verdade

| Caminho | Papel | Recuperável? |
|---|---|---|
| `notas/<tipo>/<id>.md` | **verdade** (conteúdo canônico) | versionado |
| `eventos/events.jsonl` (+ `events-NNNN.jsonl`) | auditoria append-only | `merge=union` |
| `.idx/` | **derivado** (índice, âncoras, drift, uso, sugestões, checkpoint) | descartável, reconstruível |
| `.knudge/emb_cache.jsonl` | cache vetorial (opt-in versionado, D148) | reconstruível |
| `.locks/` / `*.lock` | lock advisory | efêmero |

Regra: **`notas/` é a verdade; eventos são auditoria; `.idx/` é derivado e nunca fonte da
verdade** (D20/D21/D84). O layout `notas/<tipo>/<id>.md` é derivável do prefixo do id (D150);
há leitura tolerante ao layout plano legado.

## Escrita atômica (D20)

Toda escrita canônica é **tmp + rename no mesmo diretório**:

1. Escreve em `<arquivo>.tmp`.
2. `fsync`/`rename` (troca atômica).
3. Remove resíduos.

`fsync` é feito **em batch** (write vai para o page cache; o fsync ocorre no rebuild/sync — D22).
A porta `Fs` (`write_atomic`, `create_exclusive`, `rename`) é a única via; o adaptador real não
segue symlink (R05).

## Ordem de commit (D21)

Em operação multi-arquivo, a ordem é **nota primeiro, evento depois**. Containers/derivados são
reconstruíveis, então nunca são a fonte. Se o processo morre entre os dois, a nota existe sem
evento — o `doctor`/rebuild reconciliam; o inverso (evento sem nota) seria pior.

## Lock advisory (D23–D25)

- Aquisição por criação **exclusiva** (`O_CREAT|O_EXCL`) do arquivo de lock.
- Conteúdo `{"at": <ms>}` permite detectar **stale** (padrão 30 s).
- **Reclaim nunca apaga lock alheio**: renomeia para um sidecar (claim atômico) e só então remove.
- Retry com jitter limitado; sem jitter, a segunda tentativa já desiste.
- **Ordem documentada: externo = container, interno = nota** (evita deadlock ABBA — D25).
- Liberação é **RAII** (`LockGuard`) — o drop libera mesmo em erro (R05/R10).

## Log de eventos (D96)

- Registro: `{id, op, note_id?, at, actor?, data?}`. O `actor` foi removido (D136).
- `id = evt_<base36(8)>` **derivado do conteúdo** (D95) e usado como chave de **dedup on-read**.
- **Rotação por tamanho**: o segmento ativo `events.jsonl` vira `events-NNNN.jsonl` (default
  1 MiB); leitura percorre todos os segmentos em ordem.
- **Leitura tolerante**: linha ruim → *skip* + *warning* (nunca derruba a leitura).
- **Checkpoint** derivado em `.idx/events.checkpoint` marca o último evento processado
  (`read_since_checkpoint`).
- `revision` é **contador de versões** (default 1; cada `update` incrementa), não CAS (D96).

## Rebuild double-buffer (D27)

O índice derivado vive em `.idx/`; a reconstrução escreve em `.idx.new/` e **troca os diretórios
por `rename`** (atômico). Um leitor concorrente vê o índice **antigo ou o novo, nunca pela
metade** (`Staging` em `store/rebuild.rs`). O rebuild é acionado quando `INDEX_FORMAT` muda
(ex.: `retrieval-v4`, D206) ou por `doctor --fix`.

## Purga, detach e sweep

- **Purga** (D84): remoção de nota (supersede, merge, `compact`, TTL, dedup) dispara
  `purge_derived` — vetores, âncoras, uso, drift e sugestões associados.
- **Detach** (D46/D84): limpeza **referencial** ao remover uma nota (arestas que a citavam).
- **Sweep** (D160): na abertura da sessão (`Session::sweep_residues`) varre `notas/`, `.idx/`,
  `cache/` e `eventos/` removendo `*.tmp`/`*.stale` mais velhos que 30 s, com `warn`.
  **Nunca** varre `*.lock` nem `.locks/` — o reclaim de lock é atômico e vive em `lock.rs`/
  `doctor --fix`; varrer lock poderia roubar um lock vivo.

## Leitura única (corpus)

`Corpus { notes, index, graph }` (`corpus/mod.rs`) lê as notas **uma vez** e deriva índice e
grafo do **mesmo vetor**, sem clonar (E15-T02). `load_notes` atende quem só quer frontmatters
(paralela por `std::thread::scope`, ordem preservada — E15-T12); `load_fresh` reusa o índice de
`.idx/` quando fresco (frescura por `mtime`, E15-T11). A ordem é determinística (`list_ids`
ordena; `Index::build` reordena por `id`), então o resultado é **byte-idêntico** ao de duas
leituras separadas.

## Crash-safety e testes

O fake `FaultyFs` (`ports/fakes/fs/faulty.rs`) injeta falhas de escrita em pontos controlados para
provar que tmp+rename e o double-buffer sobrevivem a crash. `MemFs` é o FS determinístico padrão.
Testes: `store/tests/` (commit, events, layout, lock, note, purge, rebuild, sweep).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| `Store` e layout | `store/mod.rs`, `store/note.rs` |
| Ordem de commit | `store/commit.rs` |
| Eventos | `store/events/{mod,event,log}.rs` |
| Lock | `store/lock.rs` |
| Rebuild | `store/rebuild.rs` |
| Purga/detach/sweep | `store/{purge,detach,sweep}.rs` |
| Leitura única | `corpus/mod.rs` |
