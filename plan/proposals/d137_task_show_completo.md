# D137 — `show` completo e `list --full-content`

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `kd task show`/`list`. Nome confirmado: `--full-content`. Sem
> execução.

## 1. Diagnóstico (evidência)

- **`show` já tem mais do que aparenta — mas só no `--json`.** O texto
  (`commands/task/query.rs::show_one`) emite `id|statement` + contexto (`pai:`/`bloqueado_por:`/
  `bloqueia:`/`filhos:`/`epico:`/`progresso:`) + histórico. O **corpo** está no JSON, **não** no
  texto. E nem o JSON traz `checks`, `anchors`, `tags`, `outcomes` nem `kind`.
- **`list`** (`commands/task/render.rs::render_row`) emite o pipe `id|scope|status|statement`
  (+ `blocked_by=`/`cycle`/`unblocks=`). Sem corpo.
- **Precedente:** `ask --with-body` anexa o corpo **sob** a linha do hit e só no formato `Full`
  (não no `--brief`) — um modo "conteúdo" multilinha já existe.

## 2. Decisões

1. **`show` completo no texto** (paridade com o JSON): acrescentar `tipo:`, `corpo:` (bloco),
   `checks:`, `ancoras:`, `tags:`, `outcomes:`, além de `scope:`/`status:`. E **enriquecer o
   JSON** com `checks`/`anchors`/`tags`/`outcomes`/`kind` (hoje só há `body`).
2. **`list --full-content`**: renderiza cada item filtrado como **bloco completo** (reusa o
   renderer do `show`), separado por `\n---\n`, **compondo com todos os filtros** que sobraram
   (`--scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor/--sort impact`). O
   `--json` continua `{tasks:[...]}`, com os mesmos objetos do `show`.
3. **`--full-content` não é pipe-safe** (multilinha) — documentar, como o `ask --with-body`; o
   pipe enxuto (`id|scope|status|statement`) continua sendo o default.

## 3. Nome

Confirmado: **`--full-content`** (não `--show`, que colide com o subcomando; nem `--with-body`,
que é estreito demais).

## 4. Composições (exemplos)

```
kd task list --ready --full-content
kd task list --blocked --explain --full-content
kd task list --ready --sort impact --full-content
kd task list --tag parser --anchor src/toon/parse.rs --full-content
kd task show <ID> [<ID>...] [--history]
```

## 5. Raio de alcance (quando executar)

- `commands/task/query.rs` (`show_one`/`list`), `commands/task/render.rs` (`render_row`),
  `cli/task.rs` (`TaskListArgs`), `docs/05-task.md`, `prime.rs`, goldens.
- Atualizar de passagem as referências obsoletas por D134/D135/D136 na `docs/05-task.md`
  (`not_before`, `--owner`/`--mine`, `--since`, `--scope plan`).

## 6. Aceite

- [ ] `show` (texto) inclui corpo/`checks`/âncoras/tags/`outcomes`/`kind`.
- [ ] `list --full-content` compõe com todos os filtros e separa blocos por `\n---\n`.
- [ ] `--json` do `show`/`list --full-content` traz os campos novos.
- [ ] Golden atualizado; `make check` verde.
