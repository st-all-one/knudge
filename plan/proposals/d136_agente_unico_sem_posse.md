# D136 — um agente principal; fim da posse e das flags temporais de tarefa

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `kd task list`. Revisa **D114** e **D116**. O1 (`mode`) e O2
> (`actor`) resolvidos. Sem execução.

## 1. Decisões

1. **Um agente principal, sempre.** O eixo de **posse** some:
   - `kd task claim` (e `--by`/`--release`) sai;
   - `task::ownership` sai;
   - `--owner`/`--mine` saem de `kd task list`;
   - `owner` sai da projeção de `kd task graph`;
   - `KNUDGE_AGENT` deixa de ser usado.
   "Estou fazendo isto" passa a ser só `status=in_progress` (já existe, gravado no frontmatter).
   Revisa **D114**.
2. **`--since` sai de `kd task list`.** Coerência com D135 (tarefa não tem tempo). `--since`/
   `--until` continuam em `kd ask` e `kd rewind` — ali o tempo é do conhecimento/handoff.

## 2. Consequências

- **`mode` (D116) reduzido a `{sequential, concurrent, magentic}`.** `supervisor` e `handoff`
  dependem de owner → saem do enum.
- **`actor` removido do schema de evento.** Só o `claim` o gravava; sem posse, o evento passa a
  ser `{id, op, note_id?, at, data?}`.
- `kd task graph` perde a coluna `owner`; `kd task show` perde o `owner:`; `task/mod.rs` perde
  `AGENT_ENV`.
- `created_at` permanece (chave global) — mas não filtra `task list`.

## 3. Decisões confirmadas (O1/O2)

| # | Questão | Decisão |
|---|---|---|
| O1 | `mode` (D116) | reduzir a `{sequential, concurrent, magentic}` |
| O2 | `actor` no schema de evento | remover (`{id, op, note_id?, at, data?}`) |

## 4. Raio de alcance (quando executar)

- **Posse**: `task/ownership.rs` (módulo inteiro), `task/mod.rs` (exports + `AGENT_ENV`),
  `commands/task/mutate.rs` (claim/release), `commands/task/query.rs` (filtros `owner`/`mine`),
  `commands/task/graph.rs` (`owner`), `cli/task.rs` (`--owner`/`--mine`, subcomando `claim`),
  `task/context.rs` se expuser dono, `docs/05-task.md`, `prime.rs`.
- **`--since`**: `cli/task.rs` (`TaskListArgs.since`), `commands/task/query.rs`, docs, `prime.rs`.
- **`mode`**: `task/mode.rs`, `commands/task/graph.rs`, testes.
- **`actor`**: `store/events/event.rs` (campo + serialização), testes de evento.

## 5. Aceite

- [ ] `kd task claim` → subcomando inexistente (exit 2); `--owner`/`--mine` idem.
- [ ] `kd task graph` não emite `owner`; `mode` só assume `sequential|concurrent|magentic`.
- [ ] `actor` não existe mais no evento; eventos antigos com `actor` são lidos sem erro.
- [ ] `kd task list --since` → exit 2; `ask`/`rewind` mantêm `--since`/`--until`.
- [ ] `KNUDGE_AGENT` não é mais lido.
- [ ] Docs, `prime` e goldens atualizados; `make check` verde.
