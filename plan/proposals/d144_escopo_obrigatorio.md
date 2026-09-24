# D144 — escopo obrigatório em `learn`/`compact`/`prune` e `task list`

> **Status:** decisão fechada e **implementada** (v0.3.0). Aplica o princípio de **D143** ("nada de operação sem escopo").

## 1. Decisão

Nenhum desses comandos varre o corpus sem **escopo explícito**; sem escopo → erro
(`invalid_input` = 2, D130).

1. **`maintenance learn` / `compact` / `prune`** exigem ao menos um de:
   `--tag T...` / `--anchor PATH...` / `--type T...` / `--class C...` / `--scope <CONTAINER>`,
   ou **`--universe`** (varredura explícita do projeto inteiro).
2. **`kd task list`** exige ao menos um filtro:
   `--scope` / `--status` / `--kind` / `--parent` / `--ready` / `--blocked` / `--tag` / `--anchor`,
   ou **`--universe`** (panorama geral).
   `--sort impact` e `--explain` **não** contam como escopo (são modificadores).

## 2. Consequências

- O default deixa de listar/varrear tudo; a IA **declara o que quer** (D143 §2.1).
- `--universe` é o escape explícito e auditável para o projeto inteiro.
- `learn`/`compact`/`prune` reusam o **mesmo predicado de filtro** do `knowledge map`/`ask`
  (uma fonte de verdade para `--tag`/`--anchor`/`--type`/`--class`).
- `task list` já tem os filtros; a mudança é **exigir** um deles (ou `--universe`).

## 3. Raio de alcance (quando executar)

- `cli/maintenance.rs` (`Learn`/`Compact`/`Prune` args: filtros + `--universe`),
  `commands/maintenance/mod.rs` e `extra.rs` (aplicar o filtro antes de propor),
  `cli/task.rs` (`TaskListArgs`: `--universe` + validação de escopo),
  `commands/task/query.rs` (`validate_list_args`), `docs/07-manutencao.md`/`05-task.md`,
  `prime.rs`, goldens/testes.

## 4. Aceite

- [x] `maintenance learn|compact|prune` sem filtro e sem `--universe` → exit 2 (com orientação).
- [x] `kd task list` sem filtro e sem `--universe` → exit 2; `--universe` lista tudo.
- [x] `--sort impact`/`--explain` sozinhos não satisfazem o escopo.
- [x] Filtros restringem de fato o corpus varrido (teste com 2 espaços de tags distintas).
- [x] `make check` verde; docs/`prime`/goldens atualizados.
