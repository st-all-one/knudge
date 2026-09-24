# D135 — `--anchor` é o único link externo; fim de `--source`, `--expires-at` e `not_before`

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão das flags de
> `kd task new`/`kd write`. Revisa D44/D56/D57/D100/D119.

## 1. Decisões

1. **`--anchor` é o único link canônico** entre o `.knudge/` e arquivos externos — vale para
   conhecimento e tarefa. O flag `--source` sai de `kd write` e `kd task new`; o fallback para
   `source` em `task/program.rs::program_of` (D119) sai junto — o Programa é resolvido **só** por
   âncora.
2. **`expires_at` sai por completo** — flag **e** chave canônica. A expiração deixa de ser
   configurável e passa a ser sempre **derivada** da `classification` (shelf-life, D44:
   `foundational` nunca; `tactical` 365d; `observational` 30d). O `prune` segue pelo prazo
   derivado; some o override explícito de D44.
3. **`not_before` sai por completo** — flag **e** chave canônica (D100). **Tarefa não tem tempo**:
   só `created_at` (chave global); ela existe como **ação em aberto**, não como item agendado.
   Não há timestamp bloqueando tarefa.

## 2. Consequências

- `CANONICAL_KEYS`: **28 → 26** (saem `expires_at` e `not_before`). Ordem canônica em `TOON.md`
  atualizada.
- **Shelf-life 100% derivado.** `lifecycle/shelf_life.rs` perde o ramo do `expires_at` explícito;
  o prazo é sempre `created_at` + prazo da `classification` (D44 revisada).
- **As views colapsam.** O único motivo do split `compute_views` (estática) × `compute_views_at`
  (dinâmica) era `not_before` (D56/D100). Sem ele, as duas viram uma só; o parâmetro `now_ms` de
  `block_reason` deixa de existir.
- `BlockReason::Scheduled(i64)` sai; sobram `Cycle` e `Dependency`.
- Saem os campos `expires_at`/`not_before` de `frontmatter`, `graph`, `TaskSpec`, `Draft`,
  `update` e os flags de CLI; `task/render.rs` perde o ramo `Scheduled`.
- `created_at` permanece (chave global, já existente).

## 3. Raio de alcance (quando executar)

- **`--source`**: `cli/mod.rs` (`WriteArgs`), `cli/task.rs` (`TaskNewArgs`),
  `commands/task/create.rs`, `commands/write_cmd.rs`, `write/draft.rs` (parse de lote JSONL),
  `write/update.rs`, `task/program.rs` (fallback), docs, `prime.rs`.
- **`expires_at` (completo)**: `schema/keys.rs`, `schema/frontmatter.rs`, `lifecycle/shelf_life.rs`,
  `graph/mod.rs`, `commands/task/create.rs`, `TaskSpec`, `write/draft.rs`, `write/update.rs`,
  `cli/mod.rs`, `cli/task.rs`, `TOON.md`, `docs/04-write.md`/`05-task.md`, `prime.rs`, goldens e
  testes de shelf-life.
- **`not_before` (completo)**: `schema/keys.rs`, `schema/frontmatter.rs`, `graph/mod.rs`,
  `retrieval/views.rs` (merge das views + remove `Scheduled`), `commands/task/render.rs`,
  `commands/task/create.rs`, `task/spec.rs`, `write/draft.rs`, `write/update.rs`, `cli/task.rs`,
  `TOON.md`, `docs/05-task.md`, `prime.rs`, goldens e testes de views/agenda.

## 4. Aceite

- [ ] `kd write --source` / `kd task new --source` → exit 2.
- [ ] `program_of` resolve o Programa só por âncora (teste sem `source`).
- [ ] `--expires-at` inexistente como flag e como chave; expiração derivada da `classification`.
- [ ] `not_before` inexistente como flag e como chave.
- [ ] `CANONICAL_KEYS` com 26; `TOON.md` na ordem atualizada.
- [ ] `compute_views_at`/`BlockReason::Scheduled` removidos; views têm um único caminho.
- [ ] Docs, `prime` e goldens atualizados; `make check` verde.
