# D215 — Dreno em lotes limitados e `--digest` honesto

- **Status:** Aceita
- **Categoria:** 0.5.3

## Contexto

Bloco de robustez do dreno de embeddings (E11). Linhagem: Revisa D131/D170. Bordas em
[`DIVERGENCES.md`](../DIVERGENCES.md) #116.

O `--digest` reportava “fila limpa: nada pendente (indexed=0)” sempre que `indexed == 0`,
inclusive quando o provedor estava fora do ar e **nada** tinha sido indexado — a fila continuava
com `pending > 0` e o aviso de falha no `stderr`. Além disso, acima de `max_pending` o dreno
mandava a fila **inteira** numa única requisição (`queue.truncate(pending_before)`), o que torna
o `--force` (que apaga `.idx/` e redigere tudo) frágil: uma requisição gigante, uma tentativa só.

## Decisão

**Dreno sempre em lotes de `embeddings.batch`; `--digest` itera em lotes enfileirados e reporta o
estado real.** O backpressure (`max_pending`) passa a ser **informativo**: acima do teto emitimos
um aviso, mas o lote continua limitado a `embeddings.batch`. O `kd drain --digest` itera
`drain_once` até `indexed == 0`, então `--force` também redigere em lotes menores — nunca a fila
inteira numa requisição.

O resultado distingue três estados, via `pending`/`clean` no `--json`:

- **fila limpa** — `indexed == 0` **e** `pending == 0` (nada havia para fazer, `clean: true`);
- **indexado agora** — `indexed > 0` (`clean: false`, texto `indexed=… pending=…`);
- **provedor indisponível** — `indexed == 0` com `pending > 0` (`clean: false`, texto explícito).

`--force` nunca é “fila limpa”: reporta reconstrução (`rebuilt: true`). Falha do provedor **não
descarta nota** (D83/R33): ela permanece `pending` e o resultado é parcial com `warnings[]`;
sob `strict` o aviso vira erro (D94).

## Impacto

- `crates/knudge-core/src/embeddings/pipeline.rs` — `queue.truncate(embeddings.batch)` sempre;
  aviso de backpressure reaproveitado.
- `crates/knudge-cli/src/commands/drain/mod.rs` — agrega `pending`, calcula `clean` e escolhe o
  texto (`fila limpa` × `indexado agora` × `provedor indisponível`).
- Testes: `embeddings::tests::pipeline::backpressure_warns_but_keeps_batch_sized`,
  `cli::drain_digest_chunks_backlog_in_small_batches`,
  `cli::drain_digest_reports_pending_when_provider_unreachable`,
  `cli::drain_digest_reports_clean_when_nothing_pending`.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
