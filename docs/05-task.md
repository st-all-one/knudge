# 05 — Tarefas: `kd task`

O `kd task` planeja e executa trabalho numa **hierarquia fechada**:

```
epic ⊃ { issue ⊃ task | task }   (epic é a raiz; issue opcional)
```

Conhecimento (fatos/decisões) vai no [`kd write`](04-write.md); trabalho vai aqui.

## Criar

```bash
kd task new "Sync offline-first" --scope epic
kd task new "Resolver conflito de merge" --scope task --parent <epic>
kd task new "Endurecer o parser" --scope task \
  --kind risk --tag parser --anchor src/toon/parse.rs \
  --checks testes --body "rejeitar NBSP"
```

| Opção | Para quê |
|---|---|
| `--scope plan\|epic\|issue\|task` | Nível na hierarquia (**obrigatório**) |
| `--kind error\|question\|risk\|decision\|task` | Espécie (`type`) do item |
| `--parent ID` | Pai na hierarquia |
| `--body TXT` | Corpo |
| `--checks NOME...` | Validators |
| `--tag T...` | Tags |
| `--anchor PATH...` | Âncora a arquivo/glob |

## Listar

```bash
kd task list --ready                 # dependências resolvidas
kd task list --blocked --explain     # + motivo (blocked_by=/cycle)
kd task list --ready --sort impact   # ordena pelo que desbloqueia mais (+unblocks=N)
kd task list --tag parser --anchor src/toon/parse.rs --since 2026-01-01
kd task list --owner agente-a
kd task list --mine                  # só o que o ator atual (KNUDGE_AGENT) reivindicou
```

`--ready` e `--blocked` são mutuamente exclusivos; `--explain` acompanha `--blocked` ou
`--sort impact`.

## Ver contexto

```bash
kd task show <ID> [<ID>...] [--history]
```

Mostra a tarefa com **parent/blocked_by/children**, o épico a que pertence e o progresso.

## Atualizar / fechar / reivindicar

```bash
kd task update <ID> --statement "..." --status in_progress --checks testes --parent <novo>
kd task close <ID> --outcome success --note "testes verdes"   # exige evidência
kd task claim <ID> --by agente-a
kd task claim <ID> --release
```

`close` aceita `--outcome success|partial|failure|abandoned`; sem evidência, a conclusão não é
declarada (D55).

## WBS (árvore do plano)

```bash
kd task graph --program plan/016_new_ui_v2.md    # renderiza o programa externo
kd task graph --root plan_01abc                  # renderiza a árvore de um escopo (épico)
```

Saída por linha: `role|kind|status|owner|mode|progresso`.

## Plano (TOON)

```bash
kd task plan <ID> --prompt [--template feature|bug|refactor]   # imprime o prompt
kd task plan <ID> --submit --from -                            # submete o plano preenchido
kd task plan <ID> --adopt | --reorder N | --release | --review
```

## Fluxo típico

```bash
kd task new "Sync offline-first" --scope epic
kd task new "Conflito de merge" --scope task --parent <epic> --anchor src/sync.rs
kd task list --ready --sort impact
kd task claim <task> --by agente-a
kd task close <task> --outcome success --note "testes verdes"
kd sync
```

## Próximo passo

➡️ [Embeddings](06-embeddings.md) · [Manutenção](07-manutencao.md)
