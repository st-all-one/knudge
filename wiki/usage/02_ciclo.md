# 02 · O ciclo e a CLI

Esta página reúne o vocabulário essencial e as convenções que valem em **todos** os comandos.
Depois dela, cada guia de comando fica autoexplicativo.

---

## 1. O ciclo central

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

| Passo | Pergunta que responde | Comando |
|---|---|---|
| **Buscar** | "o que já se sabe sobre isto?" | [`kd ask`](05_ask.md) |
| **Gravar** | "o que eu aprendi que vale guardar?" | [`kd write`](06_write.md) |
| **Executar** | "o que falta fazer e como está?" | [`kd task`](07_task.md) |
| **Versionar** | "como compartilho/preservo isso?" | [`kd sync`](15_sync.md) |

Antes de gravar, **sempre** busque. O `kd ask "<rascunho>"` evita duplicata e mostra a nota que
talvez você só precise atualizar.

---

## 2. A memória em duas camadas

| Camada | O que é | Você mexe? |
|---|---|---|
| **Notas** (`.knudge/notas/`) | a verdade: um arquivo Markdown por afirmação | só por `kd write`/`kd task` |
| **Índice** (`.knudge/.idx/`) | atalho derivado e reconstruível | nunca — o `kd` cuida |

> **Regra de ouro:** se o índice divergir, ele é reconstruído (`kd doctor --fix`). As notas nunca
> dependem dele.

---

## 3. Tipos de conhecimento (`kd write --type`)

| Tipo | Para quê |
|---|---|
| `fact` | um fato verificado sobre o sistema (default) |
| `decision` | uma escolha e o porquê (com alternativas) |
| `error` | um problema conhecido e a correção |
| `risk` | algo que pode dar errado, com probabilidade e impacto |
| `question` | uma pergunta em aberto que merece resposta |
| `def` | a definição de um termo do projeto |
| `snippet` | um trecho de código reutilizável |
| `link` | uma referência externa |
| `meta` | conhecimento sobre o próprio projeto/processo |

**Trabalho** (tarefas, épicos) **não** entra em `kd write` — use [`kd task`](07_task.md).

---

## 4. Vocabulário essencial

| Termo | Significado |
|---|---|
| **Nota** | uma afirmação + corpo opcional, em Markdown |
| **Afirmação** (`statement`) | o resumo curto da nota (`--summary`) |
| **Corpo** | o detalhe opcional (o posicional do comando) |
| **Âncora** (`--anchor`) | arquivo/glob que a nota toca — liga memória e código |
| **Tag** (`--tag`) | rótulo livre para agrupar |
| **Classificação** | `foundational` (não expira), `tactical`, `observational` |
| **Status** | `active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten` |
| **Evidência** (`outcome`) | resultado anexado: `success`, `partial`, `failure`, `abandoned` |
| **Escopo** (`scope`) | nível de trabalho: `epic`, `issue`, `task` |
| **Handoff** | retomada de contexto (`kd rewind`) |

### 4.1 Âncora × aresta

São eixos diferentes:

- **Âncora** (`--anchor src/x.rs`): a nota **toca um arquivo** do código. É o único vínculo com
  arquivos; alimenta a busca por `--anchor` e o drift.
- **Aresta** (`--link "A:contradicts:B"`): a nota **se relaciona com outra nota**, com um **tipo**
  (12 relações). Alimenta o grafo: `ask --around --via`, `task list --ready`/`impact`, curadoria
  (`contradicts`/`replaces`), ontologia e o `doctor`.

Em resumo: **âncora = "onde"; aresta = "como se conecta a outra nota"**. Detalhes em
[`kd write`](06_write.md#âncora--aresta-não-confunda).

---

## 5. Convenções da CLI (valem em todo lugar)

### 5.1 Dados e logs nunca se misturam

- **`stdout`** = o resultado (o que você lê ou coloca num pipe).
- **`stderr`** = avisos e logs (nunca entram no resultado).

Por isso `kd ... > arquivo` sempre contém só o dado, e `kd ... 2>/dev/null | jq .` sempre é JSON
válido.

### 5.2 `--json` para máquinas

Toda operação aceita `--json`, devolvendo um envelope estável:

```json
{ "success": true, "command": "ask", "data": { }, "warnings": [] }
```

Use quando estiver automatizando ou consumindo de um script.

### 5.3 `--brief` para gastar menos contexto

Encurta a saída ao essencial (`id|afirmação`). Ideal quando um agente vai ler.

### 5.4 Posicional = conteúdo

- em `kd write` e `kd task new`, o posicional é o **corpo**;
- em `kd ask`, o posicional é a **consulta**;
- `-` lê de **stdin**; sem posicional, com pipe/heredoc, o `kd` também lê de stdin.

```bash
kd write --summary "Cache usa LRU" <<'EOF'
Motivo: custo O(1) e boa taxa de acerto.
EOF
```

### 5.5 `--params` para enviar tudo de uma vez

Em vez de várias flags, mande um objeto JSON (`-` lê de stdin). É o mesmo formato dos lotes.

```bash
kd write --params '{"statement":"Cache expira em 30 dias","type":"fact","tags":["cache"]}'
```

### 5.6 Ajuda embutida

```bash
kd                 # o mesmo que kd help
kd --help          # visão geral de todos os comandos
kd help <verbo>    # ajuda detalhada de um comando
kd prime           # o protocolo completo ("help da IA")
```

### 5.7 Listas: repita a flag ou use vírgula

Toda flag que aceita **vários valores** funciona das duas formas, equivalentes:

```bash
kd ask "cache" --tag a --tag b      # repetindo a flag
kd ask "cache" --tag a,b            # separando por vírgula
```

Vale para `--id`, `--type`, `--class`, `--tag`, `--anchor`, `--edge`, `--claim`, `--checks` e
`--files`. O formato por **espaço** (`--id a b`) **não** existe: em `ask`, `--id`/`--around` ainda
**conflitam** com a query textual (erro, exit 2). **Texto livre** (a query, o corpo, `--step`)
**não** é dividido; para dados estruturados/array, use `--params '<json>'`.

### 5.8 Conjuntos fechados (valores possíveis)

Toda flag com lista fixa **rejeita valor inválido** listando as possibilidades e sugerindo a mais
proposta ("você quis dizer…"). Flag **ausente** não valida nada (sem erro nem lista). Conjuntos do
knudge:

| Flag / campo | Valores |
|---|---|
| `--type` | `fact`, `decision`, `question`, `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk` |
| `--class` | `foundational`, `tactical`, `observational` |
| `--status` | `active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten` |
| `--scope` | `epic`, `issue`, `task` |
| `--kind` | `task`, `error`, `question`, `risk`, `decision` |
| `--outcome` | `success`, `partial`, `failure`, `abandoned` |
| arestas (`--link`/`--edge`/`--via`) | `references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`, `same_as`, `broader`, `narrower`, `related` |
| `--relation` (`ask --suggest`) | `duplicate`, `contradiction`, `link` |
| `--axis` (`map`) | `anchor`, `type`, `classification`, `scope` |
| `--sort` (`task list`) | `impact` |
| `--template` (`task plan`) | `feature`, `bug`, `refactor` |
| `self setup` | `claude`, `cursor`, `codex`, `pi` |
| `self completions` | `bash`, `zsh`, `fish` |
| `--log-level` | `error`, `warn`, `info`, `debug`, `trace`, `off` |

---

## 6. Códigos de saída

| Código | Significado |
|---|---|
| `0` | sucesso (inclui busca sem resultados e pipe fechado) |
| `2` | uso/argumento inválido |
| `3` | id/chave não encontrado |
| `4` | conflito (ex.: atualizar nota inexistente) |
| `5` | erro de arquivo |
| `6` | tempo esgotado (provedor/hook) |
| `7` | configuração inválida |
| `8` | nota/schema inválido |
| `70` | erro interno |

`warnings[]` indica **degradação graciosa**: um recurso opcional (ex.: embeddings) falhou e o
resultado veio parcial, sem quebrar o comando.

---

## 7. Um dia típico de uso

```bash
# Chegando no projeto
kd rewind --budget 2000            # onde eu estava?
kd task list --ready --sort impact # o que vem primeiro?

# Durante o trabalho
kd ask --anchor src/cache.rs --brief      # o que já sei deste arquivo?
kd write --summary "..." --anchor src/cache.rs
kd task update --id task_01abc --status in_progress

# Encerrando
kd task close --id task_01abc --outcome success --note "testes verdes"
kd doctor                          # saúde do corpus
kd sync --message "notas: sessão de hoje"
```

---

## 8. Erros comuns (e como evitar)

| Erro | Correção |
|---|---|
| Gravar sem buscar e duplicar | `kd ask "<rascunho>"` antes de `kd write` |
| Criar tarefa com `kd write` | use `kd task new` (o `write` rejeita `--type task`) |
| Deixar tudo `active` para sempre | classifique (`--class`) e revise com `kd maintenance prune` |
| Editar arquivo em `.knudge/notas/` à mão | use `kd write --update` / `kd doctor --fix` |
| Perder o fio entre sessões | `kd rewind` no início e no fim |

---

## 9. Próximo passo

➡️ [03 · `kd init`](03_init.md) · [Guia por comando](README.md#comandos)
