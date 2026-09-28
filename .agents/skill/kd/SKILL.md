---
name: knudge
description: Memória durável por projeto via `kd` (knudge). Use ao buscar, gravar, planejar tarefas, retomar contexto e manter a base. Dispare em: knudge, kd, memória, nota, ask, write, task, épico, rewind, handoff.
---

# knudge — uso real

`kd` é a memória do projeto. A **nota Markdown é a verdade**; índice/embeddings/grafo são
**derivados**. `notas/` não se edita à mão.

## Ciclo

```
kd ask → kd write → kd task → kd sync
```

## Regras de ouro

1. **Busque antes de gravar.** `kd ask "<rascunho>"` evita duplicata (dedup: <0.75 cria,
   0.75–0.92 faz merge, ≥0.92 rejeita).
2. **Uma afirmação por nota.** O `statement` é curto, autocontido e vira o `id` — não empilhe
   afirmações nem dependa de contexto externo.
3. **Corpo = o "porquê" que não cabe no statement.** Use quando o statement sozinho não permite
   agir (`decision`/`error`/`risk`). Template (2–4 linhas):
   ```
   Por quê: <motivo/decisão>
   Evidência: <comando, saída, erro, link>
   Consequência: <o que muda na prática>
   ```
   `kd ask` mostra o corpo: 1º hit completo, 2–5 truncado; leia com `--id`/`--full-content`.
4. **Ancore o código.** Toda nota/tarefa sobre um arquivo leva `--anchor PATH` (glob `src/**`
   casa subárvores). `kd ask --anchor PATH` acha pelo arquivo.
5. **Evidência separada do corpo.** Conclusão de tarefa usa `--outcome`; fato/decisão ganha
   âncora. Corpo não substitui evidência.

## Comandos essenciais

```
kd prime                                   # protocolo completo (1x por sessão)
kd ask "<query>" [--limit N] [--brief]     # recuperar; --brief só id|statement
kd ask --id <ID>                           # corpo completo de uma nota
kd ask --anchor src/x.rs                   # por arquivo, sem query
kd write --summary "<afirmação>" [<corpo>|-] --type <fact|decision|error|risk|question>
        [--tag T] [--anchor PATH]
kd write --update <ID> --summary "<...>"   # muda statement → novo id + supersede
kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]
kd task new --summary "<...>" --scope <epic|issue|task> [--parent ID] [--anchor PATH]
kd task close --id <ID> [--outcome S]      # só declara com evidência
kd rewind [--budget N]                     # retomar contexto entre sessões
kd doctor [--fix] [--explain]            # saúde da base
kd sync [--message M]                      # commit de notas/ + eventos/
```

## Anti-padrões

- Gravar sem `kd ask` antes (duplicata) ou `--type task` no `write` (use `kd task`).
- Statement composto/ambíguo, sem âncora quando fala de código.
- Guardar segredo no corpo (o log redige, mas a nota não deve conter segredo).
- Editar `notas/` à mão — use `kd write --update`.

## Saída

stdout = dados, stderr = logs. `--json` = `{success, command, data?, error{code,message,retryable}, warnings?}`.
Busca vazia → `[no_results]` (exit 0).

<!-- knudge:skill:version: 1 -->
