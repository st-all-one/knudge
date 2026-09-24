# D141 — criação de tarefas em lote (JSONL) e por objeto

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `kd task new`. Espelha `kd write --batch` (D110). O1–O4
> confirmados; inclui **atualizar hierarquia** e **criar vínculos** no lote. Sem execução.

## 1. Motivação

Não há como preencher vários parâmetros de uma vez nem criar várias tarefas/epics rapidamente.
Para **conhecimento** já existe `kd write --batch -` (JSONL, D110); falta o equivalente de
**tarefa**, com hierarquia (`parent`) e vínculos (as 8 `EdgeKind`).

## 2. Proposta

### 2.1 `--params '{...}'` — um item por objeto JSON

`kd task new --params '{ "statement": "S", "body": "…", "scope": "task", "kind": "error",
"anchors": ["src/x.rs"], "tags": ["parser"] }'` — atalho de um item, sem uma flag por campo.
Compartilha o **mesmo parser** do lote.

### 2.2 Lote JSONL — `kd task new --batch FILE|-`

Uma linha = uma **operação**; a presença de `id` decide o tipo:

| Campo | Papel |
|---|---|
| `key` | referência **local** opcional (para `parent`/arestas entre linhas) |
| `id` | **atualiza** um item existente; **ausente** = cria |
| `statement` | a afirmação (≤120) |
| `body` | o **corpo** Markdown (explícito, como pedido) |
| `scope` | `epic\|issue\|task` (criação) |
| `kind` | `task\|error\|question\|risk\|decision` |
| `parent` | id existente **ou** `key` — anexa/re-parenta (aresta `results_in`) |
| `checks` / `anchors` / `tags` | listas |
| `classification` / `status` / `blocks` | opcionais |
| `references` / `depends_on` / `contradicts` / `supports` / `extends` / `replaces` / `rejects` | as 7 arestas explícitas, com ids ou `key`s |

### 2.3 Atualizar hierarquia e criar vínculos

- **Criar**: linha sem `id` → `task::submit` (com `parent` por `key`/id).
- **Atualizar hierarquia**: linha com `id` + `parent` → re-parenta (reusa `kd task update`);
  `status`/`statement`/`checks` também aceitos.
- **Vínculos**: qualquer linha carrega as 7 arestas (ids ou `key`s), criadas via **`write::link`**
  (via única de D126 — sem duplicar a regra). `results_in` vem do `parent`.
- `depends_on` é resolvido ao final (aceita `key` de qualquer linha); `parent` exige que o pai
  **apareça antes** (criação) ou já exista (id).
- **Best-effort** (espelha `write --batch`, R33): linha inválida vira `warnings[]` e o lote
  continua; `--dry-run` só avalia; teto `task.batch_max` (espelha `write.batch_max`).
- **Saída auto-suficiente:** cada item vira uma linha `key|id|scope|status|statement`
  (`key` = `-` quando ausente), para a IA **vincular sem nova consulta**; o `--json` traz
  `items[]` com `action`/`key`/`id`/`scope`/`kind`/`status`/`statement`/`parent`/`edges`
  (arestas já resolvidas) e o mapa `key`→id. Não devolve o corpo (a IA o escreveu).

## 3. Consequências

- Novo `TaskSpec::from_value`/parser de operação (puro) no core; o lote reusa `task::submit`,
  `task update` e `write::link`.
- `plan --submit` (D105) permanece para o caso "um épico + passos atômicos"; o `--batch` é o
  caminho **geral best-effort** (hierarquia livre, várias raízes, re-parent, vínculos).
- Config nova `task.batch_max` (int).

## 4. Confirmados (O1–O4)

| # | Decisão |
|---|---|
| O1 | nome: **`task new --batch`** |
| O2 | **`--params`** entra (atalho de um item) |
| O3 | **best-effort + `--dry-run`** |
| O4 | chave do resumo: **`statement`** |

## 5. Raio de alcance (quando executar)

- `task/spec.rs` (`from_value`), novo `task/batch.rs`, `task/mod.rs`, `cli/task.rs`
  (`TaskNewArgs` + `--params`/`--batch`), `commands/task/create.rs`, `config` (`task.batch_max`),
  `docs/05-task.md`, `prime.rs`, goldens/testes.

## 6. Aceite

- [ ] `--params '{...}'` cria um item com os campos do objeto.
- [ ] `--batch -` cria epic + tasks com `parent` por `key` e `depends_on` por `key`/id.
- [ ] Linha com `id`+`parent` **re-parenta**; linha com aresta **cria o vínculo**.
- [ ] A saída lista `key|id|scope|status|statement` por item e o `--json` traz `key`→id e as
      arestas resolvidas — a IA liga os itens sem nova consulta.
- [ ] `--dry-run` não grava; linha inválida → `warnings[]`; acima de `task.batch_max` → exit 2.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
