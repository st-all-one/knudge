# D147 — `--params '{...}'` universal e stdin/heredoc universal

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Generaliza **D140** (conteúdo por stdin/heredoc) e **D141** (`--params`). Sem
> execução.

## 1. Decisões

1. **`--params '{json}'` universal.** `kd write`, `kd ask` e `kd task new` (D141) aceitam um
   **objeto JSON** com os campos do comando, enviando o conjunto completo de uma vez — útil para
   scripts e chamadas programáticas (MCP). `--params -` lê o objeto de **stdin**.
2. **stdin/heredoc universal.** Vira regra do `kd` (estende D140): onde há **conteúdo** (posicional),
   `-` lê de stdin e, **sem posicional com stdin não-TTY**, lê do stdin — cobrindo pipe e heredoc
   (`<<'EOF'`). Sem TTY e sem entrada → erro (`invalid_input`, D130). Aplica a `write` (corpo),
   `ask` (consulta) e demais comandos de conteúdo.
3. **Vias exclusivas.** `--params` e o posicional `-` não podem ler o mesmo stdin ao mesmo tempo;
   com `--params -`, o conteúdo vem do próprio objeto (`body`).

## 2. Campos por comando

| Comando | `--params` (objeto) |
|---|---|
| `write` | `type`, `statement`, `body`, `tags`, `anchors`, `classification`, `status` (mesmo schema do `--batch`, D110) |
| `ask` | `query`, `type`/`class`/`tag`/`status`/`scope`/`anchor`/`since`/`until`, `limit`, `brief`, `full_content`, `with_task`, `id`/`around`/`via`/`depth` |
| `task new` | schema de D141 (`key`, `statement`, `body`, `scope`, `kind`, `parent`, `checks`, `anchors`, `tags`, `classification`, `status`, `blocks`, arestas) |

## 3. Consequências

- `--params` é o **item único** de um lote: `write --params` = uma linha de `write --batch`;
  `task new --params` = uma linha de `task new --batch`. **Mesmo parser** (uma fonte de verdade).
- O conteúdo por stdin/heredoc é o padrão **universal** do `kd`, não exclusivo de `write`.
- Sem flags novas além de `--params`; a superfície fica estável.

## 4. Raio de alcance (quando executar)

- `cli/mod.rs` (`WriteArgs`, `AskArgs`), `cli/task.rs` (`TaskNewArgs`), `commands/write_cmd.rs`,
  `commands/ask/*`, `commands/task/create.rs`, parser compartilhado de `--params`
  (reuso do `Draft::from_value`/`TaskSpec::from_value`), docs (`03-ask`, `04-write`, `05-task`),
  `prime.rs`, goldens/testes.

## 5. Aceite

- [ ] `kd write --params '{...}'` cria a nota com os campos do objeto.
- [ ] `kd ask --params '{...}'` executa a consulta com os filtros do objeto.
- [ ] `--params -` lê o objeto de stdin; `write - <<'EOF'`/`cat | write` gravam o corpo.
- [ ] `ask -` lê a consulta de stdin; sem TTY e sem entrada → exit 2.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
