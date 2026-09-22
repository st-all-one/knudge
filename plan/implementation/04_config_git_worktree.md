# E04 — Config, Git e worktree

> **Fase 0.** Onde o conhecimento mora, como o LLM o descobre e como ele não polui o
> repositório. Config em dois níveis, `.git/info/exclude`, worktree principal, nome lógico
> de projeto e `sync`.
>
> **Decisões:** D29, D30, D31, D32, D33, D34, D57, D59 (registro), D60, D61, D62, D63, D64, D91, D97.
> **Políticas:** R05, R12 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

`onboard()` idempotente que cria/atualiza `.knudge/`, aplica a exclusão correta, gera o
`AGENTS.md` e deixa a config pronta — com **segredos só no global** e **worktrees compartilhando**
o mesmo conhecimento.

## Pré-requisitos

E01, E03.

## Tarefas

### E04-T01 ☑ Config em dois níveis
- **Objetivo:** global (template) + projeto (efetivo, **precedência**); **clone literal** na
  instanciação; writer TOML com **ordem canônica e quoting estáveis**; `config set/unset`
  valida contra o schema, poda ancestrais vazios e revalida.
- **Entregáveis:** loader/merger; writer estável; `config get/set/list/unset`.
- **Decisões:** D61, D62, D63, D64.
- **Aceite:** precedência projeto→global; re-rodar `onboard` não sobrescreve projeto;
  `config set` produz diff mínimo e rejeita valor inválido.

### E04-T02 ☑ `.knudge/` no worktree principal + identidade lógica
- **Objetivo:** resolver o diretório de conhecimento no **worktree principal**
  (`git rev-parse --git-common-dir`); submódulo não conta; **nome lógico** de projeto em vez
  de path; worktrees do mesmo repo **compartilham** `.knudge/`; **segredos só no global**.
- **Entregáveis:** resolvedor de raiz; validação de nome (rejeita `.`, `..`, caminho absoluto);
  doc de precedência de credenciais.
- **Decisões:** D29, D91.
- **Aceite:** dois worktrees enxergam o mesmo `.knudge/`; config local com `[secrets]` é
  ignorada; teste de submódulo.

### E04-T03 ☑ `.git/info/exclude` e `persist_in_project`
- **Objetivo:** exclusão **idempotente** via `.git/info/exclude` (nunca `.gitignore`);
  `persist_in_project=true` versiona `notas/`/`eventos/` e exclui derivados
  (`.idx/`, `cache/`, `*.lock`); `false` exclui o `.knudge/` inteiro; fora de repo = no-op.
- **Entregáveis:** gerenciador de exclude; reversão quando volta a `true`.
- **Decisões:** D30, D34.
- **Aceite:** aplicar/reverter não duplica linhas; `git status` mostra só o esperado; teste
  fora de repositório.

### E04-T04 ☑ `onboard()` idempotente
- **Objetivo:** marcadores `<!-- knudge:start --> … <!-- knudge:end -->` + **version marker**;
  gera/atualiza `AGENTS.md` com o protocolo (uma vez).
- **Entregáveis:** `onboard()`; template do `AGENTS.md`; detector de versão.
- **Decisões:** D57, D60.
- **Aceite:** re-rodar não duplica; version marker antigo dispara atualização do bloco.

### E04-T05 ☑ `sync()`
- **Objetivo:** commit de `notas/` + `eventos/`; **mensagem gerada do evento**; guard de
  worktree (não commitar no worktree errado).
- **Entregáveis:** `sync()`; gerador de mensagem.
- **Decisões:** D32.
- **Aceite:** commit no worktree certo; mensagem determinística e auditável.

### E04-T06 ☑ `.gitattributes` e derivados ignorados
- **Objetivo:** `merge=union` para `eventos.jsonl`; `.idx/`, `cache/`, `*.lock` gitignored.
- **Entregáveis:** escrita de `.gitattributes` e do ignore de derivados.
- **Decisões:** D31.
- **Aceite:** merge simulado do log não conflita; derivados nunca entram no commit.

## Definition of Done

- [x] `onboard` idempotente e worktree-aware.
- [x] Exclusão e `persist_in_project` testados nos quatro quadrantes.
- [x] `sync` gera commit correto no worktree certo.

## Não-objetivos

- Hooks de ciclo de vida (E12).
- Migração de seeds/mulch (recusada por D70).
