# 06 · `kd write` — toda a escrita

## Para que serve

É o comando de **escrita de conhecimento**. Ele cria notas, versiona (`--update`), liga notas
(`--link`), anexa evidência (`--outcome`), registra claims e proveniência e aceita lotes. Trabalho
(tarefas, épicos) **não** entra aqui — use [`kd task`](07_task.md).

O posicional é o **corpo** (Markdown); a afirmação é `--summary`. Antes de gravar, o `write` procura
duplicatas e decide se **cria**, **mescla** ou **recusa**.

## Quando usar

- **Use** para fatos, decisões, erros, riscos, perguntas, definições, snippets, links e metas.
- **Use `--update`** para corrigir ou complementar uma nota existente.
- **Não use** para trabalho (`--type task` é rejeitado) nem para apagar
  ([`kd forget`](14_forget.md)).

## Sintaxe

```
kd write [BODY]... --summary <TXT> [--type T] [--tag T]... [--anchor P]... [--class C] [--status S]
kd write --update <ID> [--summary TXT] [...] [--clear-anchors] [--params JSON]
kd write --link <FROM:ARESTA:TO>
kd write --outcome <OUTCOME> --id <ID> [--note TXT]
kd write --batch <FONTE|-> [--dry-run]
kd write --params '<json>' [--dry-run]
```

## Exemplos

### Criar uma nota

```bash
# 1. fato com tag e âncora
kd write --summary "O gateway faz retry exponencial" --type fact \
  --tag gateway --anchor src/gateway.rs

# 2. com corpo explicativo (heredoc)
kd write --summary "Cache usa LRU" --type decision <<'EOF'
Alternativas: FIFO e LFU.
Por quê: LRU tem custo O(1) e melhor taxa de acerto no padrão de acesso.
EOF

# 3. decisão fundamental (não expira)
kd write --summary "A API é versionada por header" --type decision \
  --class foundational --anchor src/api.rs
```

Saída (texto): `created|fact_01qejflt|r1` — ação, id e revisão.

### Dedup automático

O `write` roda a busca internamente:

| Situação | Ação |
|---|---|
| Nada parecido | `created` — cria nota nova |
| Parecido | `merged` — incorpora à nota existente |
| Muito parecido | `rejected` — é duplicata |

```bash
# 1. revisar antes de gravar
kd ask "O gateway faz retry exponencial" --brief

# 2. gravar mesmo assim (o write decide)
kd write --summary "O gateway faz retry exponencial com jitter" --type fact

# 3. forçar a atualização da nota encontrada
kd write --update fact_01qejflt --summary "O gateway faz retry exponencial com jitter"
```

### Atualizar uma nota

```bash
# 1. novo texto
kd write --update fact_01qejflt --summary "Retry exponencial com jitter"

# 2. trocar campos via objeto (patch)
kd write --update fact_01qejflt --params '{"body":"novo corpo","tags":["cache"]}'

# 3. limpar âncoras
kd write --update fact_01qejflt --clear-anchors
```

Mudar `type` ou a afirmação cria uma **nova** nota e marca a antiga como substituída. A revisão
(`rN`) incrementa a cada atualização.

### Ligar notas (arestas)

```bash
# 1. uma decisão estende um fato
kd write --link "decision_01abc:extends:fact_01xyz"

# 2. uma tarefa depende de outra
kd write --link "task_01def:depends_on:task_01ghi"

# 3. contradição declarada
kd write --link "fact_01aaa:contradicts:fact_01bbb"
```

As relações disponíveis (12) são: `references`, `depends_on`, `contradicts`, `supports`, `extends`,
`replaces`, `rejects`, `results_in` e as de vocabulário `same_as`, `broader`, `narrower`, `related`.
Um valor fora dessas é erro (exit 8, com sugestão da mais provável).

Também dá para criar uma aresta **a partir da nota recém-criada**, sem precisar saber o id depois:

```bash
# 1. nova nota que estende uma existente
kd write --summary "O retry tem jitter" --type fact --edge extends:fact_01xyz

# 2. nova nota que apoia uma decisão
kd write --summary "Medição confirma a janela deslizante" --type fact --edge supports:decision_01abc

# 3. nova nota que substitui uma antiga
kd write --summary "Janela fixa (revisão)" --type decision --edge replaces:decision_01old
```

#### Âncora × aresta (não confunda)

- **Âncora** (`--anchor src/x.rs`) liga a nota a um **arquivo** do código — é o único vínculo com
  arquivos e o que a busca por `--anchor` usa.
- **Aresta** (`--link`) liga a nota a **outra nota**, com um **tipo** de relação.

| | `--anchor` | `--link` |
|---|---|---|
| Liga a | arquivo/glob | outra nota (id) |
| Vocabulário | livre | fechado (12 relações) |
| Direção | — | dirigida (`from → to`) |
| Serve para | recuperar por arquivo, medir drift | navegar (`ask --around --via`), planejar (`ready`/`impact`), curar (`contradicts`/`replaces`), inferir (ontologia) |

Use os dois juntos quando fizer sentido: uma nota pode falar de um arquivo **e** contradizer outra.

### Anexar evidência (outcome)

```bash
# 1. confirmar com sucesso
kd write --outcome success --id task_01def --note "testes verdes em CI"

# 2. registrar uma falha
kd write --outcome failure --id fact_01xyz --note "medição refutou a hipótese"

# 3. resultado parcial
kd write --outcome partial --id task_01def --note "2 de 3 casos cobertos"
```

Evidência de sucesso **fortalece** a nota (aparece em [`kd ask --rank`](05_ask.md)); falhas a
enfraquecem.

### Claims e proveniência (opcional)

```bash
# 1. declarar um fato atômico sujeito:relação:objeto
kd write --summary "embeddings escuta na 8889" --claim "embeddings:porta:8889"

# 2. registrar quem produziu a nota
kd write --summary "..." --agent "claude" --activity "write"

# 3. dois claims na mesma nota
kd write --summary "..." --claim "servico:porta:8889" --claim "servico:versao:2"
```

Claims habilitam detecção precisa de contradição: duas notas com o mesmo sujeito/relação e objetos
diferentes são apontadas pelo [`kd doctor`](10_doctor.md).

### Lote (várias notas de uma vez)

```bash
# 1. objeto único (atalho do lote)
kd write --params '{"statement":"Cache expira em 30 dias","type":"fact","tags":["cache"]}'

# 2. JSONL por stdin
kd write --batch - --dry-run <<'EOF'
{"statement":"Nota A","type":"fact"}
{"statement":"Nota B","type":"decision","tags":["x"]}
EOF

# 3. aplicar o lote de verdade
kd write --batch notas.jsonl
```

O lote usa as chaves **canônicas** (`statement`, `body`, `type`, `tags`, `anchors`,
`classification`, `status`, `claims`, `provenance`), não os nomes das flags. `--dry-run` só avalia.

## Flags

| Flag | Efeito |
|---|---|
| `[BODY]...` | Corpo Markdown (posicional; `-`/pipe lê stdin) |
| `--summary <TXT>` | Afirmação da nota |
| `--type <TIPO>` | Espécie: `fact`, `decision`, `question`, `def`, `error`, `snippet`, `link`, `meta`, `risk` (default `fact`; `task` é rejeitado — use [`kd task`](07_task.md)) |
| `--tag <T>` | Tag (repetível; aceita vírgula) |
| `--anchor <PATH>` | Âncora (repetível; aceita vírgula) |
| `--clear-anchors` | Com `--update`, limpa as âncoras (conflita com `--anchor`) |
| `--class <C>` | Classificação: `foundational`/`tactical`/`observational` |
| `--status <S>` | Status inicial: `active`/`in_progress`/`blocked`/`closed` |
| `--edge <ARESTA:ID>` | Aresta a partir da nota criada (uma das 12 abaixo; repetível; aceita vírgula) |
| `--update <ID>` | Versiona a nota existente |
| `--id <ID>` | Id alvo (com `--outcome`) |
| `--link <FROM:ARESTA:TO>` | Cria aresta entre notas existentes |
| `--outcome <OUTCOME>` | Anexa evidência (`success`/`partial`/`failure`/`abandoned`) |
| `--note <TXT>` | Texto da evidência |
| `--claim <S:R:O>` | Claim atômica (repetível; aceita vírgula) |
| `--agent <NOME>` | Quem produziu a nota |
| `--activity <NOME>` | Como a nota foi produzida |
| `--batch <FONTE>` | Lote JSONL (`-` lê stdin) |
| `--params <JSON>` | Objeto de uma nota (`-` lê stdin) |
| `--dry-run` | Simula sem gravar |

## Resultado esperado

- **Texto:** `acao|id|rN` (ex.: `created|fact_01abc|r1`).
- **`--json`:** `data.action`, `data.id`, `data.revision`; no lote, `data.items[]`.
- **Avisos por tipo:** o `write` confere se o corpo tem as seções esperadas de cada tipo (ex.:
  `decision` → alternativas/porquê/consequência). Faltar seção gera **aviso**, não erro — a nota é
  gravada. Com `behavior.strict=true`, o aviso vira erro.
- **`--type task`** é rejeitado (exit 2). Erros de schema → exit 8; atualizar id inexistente → exit 4.

## Quando não usar

- Para trabalho → [`kd task new`](07_task.md).
- Para apagar → [`kd forget`](14_forget.md).
- Para editar arquivos do corpus à mão → nunca; use `kd write --update`.

## Veja também

➡️ [`kd ask`](05_ask.md) · [`kd task`](07_task.md) · [`kd maintenance`](11_maintenance.md)
