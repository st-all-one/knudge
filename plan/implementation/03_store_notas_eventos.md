# E03 — Store de notas, lock e eventos

> **Fase 0.** `notas/` é a verdade; o resto é derivado. Aqui entram escrita atômica, ordem de
> commit, lock advisory (CLI + MCP são dois processos), `eventos.jsonl` append-only com dedup
> on-read, `revision` e o rebuild double-buffer.
>
> **Decisões:** D15, D20, D21, D22, D23, D24, D25, D26, D27, D28, D48 (revision), D84, D96.
> **Políticas:** R05, R10, R13, R34 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um store de arquivos **crash-safe e concorrente-seguro**, com eventos auditáveis e um índice
derivado que **nunca** é fonte da verdade.

## Pré-requisitos

E02.

## Tarefas

### E03-T01 ☑ Escrita atômica e fsync em batch
- **Objetivo:** gravar nota em `notas/<id>.md` via **tmp + rename no mesmo diretório**;
  `fsync` em batch (write no page cache; fsync no rebuild/sync).
- **Entregáveis:** `core/store` com `write_atomic`; política de fsync.
- **Decisões:** D20, D22.
- **Aceite:** crash-injection entre tmp/rename deixa o arquivo antigo **ou** o novo; nunca
  parcial. Golden de bytes pós-restart.

### E03-T02 ☑ Ordem de commit multi-arquivo
- **Objetivo:** **nota primeiro, evento depois**; container é derivado.
- **Entregáveis:** orquestração de commit; testes de crash no meio.
- **Decisões:** D21.
- **Aceite:** crash entre os dois deixa **nota sem evento** (recuperável por `doctor`), nunca
  evento apontando para nota inexistente.

### E03-T03 ☑ Lock advisory por arquivo-alvo
- **Objetivo:** `O_CREAT|O_EXCL`, **stale 30s**, retry com jitter; reclaim por **rename
  sidecar + inode/mtime**; **nunca apagar lock alheio**; ordem de aquisição documentada.
- **Entregáveis:** `core/lock`; doc da ordem (externo=container, interno=nota).
- **Decisões:** D23, D24, D25.
- **Aceite:** dois processos não perdem update; reclaim não remove lock válido; stress de
  concorrência; proptest de não-deadlock com a ordem documentada.

### E03-T04 ☑ `eventos.jsonl` append-only
- **Objetivo:** log de eventos append-only, **dedup on-read**, `merge=union` (D31), linha
  malformada **skip + warning**.
- **Entregáveis:** writer/reader de eventos; dedup on-read.
- **Decisões:** D26, D28.
- **Aceite:** merge simulado não duplica; leitura tolera linha ruim sem derrubar o comando.

### E03-T05 ☑ `revision` e update versionado
- **Objetivo:** `revision` é **contador de volatilidade** (não CAS); `update` incrementa;
  histórico caminhável.
- **Entregáveis:** campo `revision`; leitura de histórico.
- **Decisões:** D48.
- **Aceite:** `revision` inicia em 1 e incrementa a cada `update`; `history(id)` retorna os eventos da nota.

### E03-T06 ☑ Rebuild double-buffer do índice
- **Objetivo:** reconstruir `.idx/` em `.idx.new/` + rename; `fsync` no rebuild.
- **Entregáveis:** `core/rebuild`.
- **Decisões:** D27.
- **Aceite:** um leitor concorrente nunca vê índice pela metade; teste rebuild × `recall`.

### E03-T07 ☑ Purga do derivado em toda remoção
- **Objetivo:** função única que remove uma nota do derivado (supersede, merge, `compact`,
  TTL, dedup) — o **vetor/índice** some junto; nunca só o canônico.
- **Entregáveis:** `remove_derived(id)` usada por todos os caminhos; checagem no `doctor`.
- **Decisões:** D84.
- **Aceite:** após qualquer remoção, o índice não contém o id; `doctor` detecta e reporta
  divergência canônico↔derivado.

### E03-T08 ☑ Limpeza RAII e varredura de resíduos
- **Objetivo:** nenhum `*.tmp`/`*.lock` órfão sobrevive a erro ou crash.
- **Entregáveis:** `TempFile`/`LockGuard` com `Drop` (remove/libera); `fsync` **antes** do drop;
  ordem de drop documentada (nota → evento → lock); varredura de resíduos na inicialização
  (idade > threshold) com `warn`, **sem** remover lock de processo vivo.
- **Decisões:** D20, D23, D24. **Políticas:** R05, R10, R34.
- **Aceite:** crash/erro não deixa resíduo; órfão antigo removido; lock vivo preservado.

### E03-T09 ☑ Rotação e checkpoint de `eventos.jsonl`
- **Objetivo:** o log de eventos não cresce para sempre.
- **Entregáveis:** segmentação `eventos/<yyyy-mm>.jsonl` (ou por tamanho) + **checkpoint de
  offset** lido; `compact` pode consolidar; dedup on-read e tolerância mantidos.
- **Decisões:** D26, D28. **Políticas:** R13.
- **Aceite:** corpus grande não carrega o log inteiro; leitura pela segmentação; dedup mantido.

## Definition of Done

- [x] Crash-injection cobre tmp/rename, nota/evento e rebuild.
- [x] Lock e dedup passam em stress de concorrência.
- [x] Nenhum caminho de remoção deixa derivado órfão.
- [x] Sem resíduo de tmp/lock; log de eventos segmentado.

## Não-objetivos

- Índice invertido/BM25 (E06).
- Decay/TTL/purge de inativos por tempo (E10).
