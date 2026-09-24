# 02 — Primeiros passos

## 1. Fundar a memória do projeto

Dentro do seu projeto (na raiz do git):

```bash
kd init
```

O `kd init` cria `.knudge/` (notas, eventos, índice derivado) e escreve um bloco no `AGENTS.md`
do projeto, ensinando o agente a usar o `kd`. A **verdade** são os arquivos Markdown em
`.knudge/notas/`; todo o resto é **derivado** e reconstruível.

## 2. Buscar antes de gravar

```bash
kd ask "como o gateway limita requisições" --brief
```

A saída no terminal (ou no pipe) é `id|statement|score|why`, um hit por linha. Use `--brief`
para gastar menos contexto.

## 3. Gravar conhecimento

```bash
kd write --type fact "Rate limit é 100 rps por chave" \
  --tag gateway --anchor src/gateway.rs
```

O `write` faz **dedup** contra o que já existe:

| Score do `ask` | Ação do `write` |
|---|---|
| `< 0.75` | Cria nota nova |
| `0.75 – 0.92` | Faz **merge** na nota existente |
| `≥ 0.92` | **Rejeita** (duplicata) |

Tipos de conhecimento: `fact`, `decision`, `error`, `risk`, `question`, `def`, `snippet`,
`link`, `meta`, `task`. Para **trabalho**, use `kd task` (o `write` rejeita
`--type task`).

## 4. Planejar e executar

```bash
kd task new "Sync offline-first" --scope epic
kd task new "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact
kd task claim <task> --by agente-a
kd task close <task> --outcome success --note "testes verdes"
```

A hierarquia é fechada: **`plan ⊃ epic ⊃ issue ⊃ task`** (profundidade máxima 4). Fechar exige
**evidência** (`--outcome`).

## 5. Retomar contexto entre sessões

```bash
kd rewind --budget 2000        # manifest + tarefas prontas (next:)
kd rewind --files src/gateway.rs
kd rewind --resume <context_id>  # retoma 1:1
```

## 6. Commitar

```bash
kd sync --message "notas: decisão do rate limit"
```

Versiona `notas/` + `eventos/`. O derivado (`.idx/`, `cache/`) fica fora do git (o `kd init`
cuida do `.gitignore`).

## O ciclo, de novo

```
kd ask → kd write → kd task → kd sync
```

**Regra de ouro:** busque antes de gravar. O `kd ask "<rascunho>"` evita duplicata e aponta a
nota que talvez você só precise atualizar (`kd write --update <ID> "..."`).

## Próximo passo

➡️ [Busca — `kd ask`](03-ask.md) · [Escrita — `kd write`](04-write.md) · [Tarefas — `kd task`](05-task.md)
