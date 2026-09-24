# Conceitos e arquitetura

Esta página explica **como o knudge pensa**: o que é a verdade, como uma nota é estruturada, como
o índice é derivado e quais decisões de projeto moldam a CLI. Leia uma vez para entender o resto
da documentação; volte a ela quando um comportamento parecer estranho.

## 1. A ideia em uma frase

**knudge é uma memória por projeto para agentes de IA.** Em vez de despejar contexto no prompt, o
agente **busca** o que já se sabe (`kd ask`), **grava** o que aprendeu (`kd write`) e **planeja**
o trabalho (`kd task`) — sempre em notas Markdown versionadas que vivem ao lado do código.

A meta não é "um banco de conhecimento", é **decidir rápido com pouco contexto**: a saída padrão
de uma busca é `id|statement|score|why`, uma linha por hit.

## 2. A verdade são as notas; o índice é derivado

| Camada | Papel | Estabilidade |
|---|---|---|
| `.knudge/notas/<tipo>/<id>.md` | **Fonte da verdade.** Um arquivo = uma unidade de recuperação. | **Fixa** |
| `.knudge/eventos/events.jsonl` | Append-only. Auditoria, `learn`, `diff`, `audit`. Não define ordem. | Recomendada |
| `.knudge/.idx/` | Índice derivado (retrieval, embeddings, contextos, clusters). | Oscila |
| `.knudge/emb_cache.jsonl` | Cache vetorial **versionado** (opt-in) — ver [Embeddings](15-embeddings.md). | Versionado |
| `.knudge/config.toml` | Config efetiva do projeto (precedência sobre a global). | Formato fixo |

**Regra de ouro:** `notas/` + `eventos/` sempre reconstroem o `.idx/` do zero. O índice **nunca**
é fonte da verdade; se ele divergir, `kd maintenance doctor --fix` reconstrói.

## 3. A nota (TOON)

Cada nota é um arquivo com um frontmatter **TOON** (ordem canônica de chaves) e um corpo Markdown.
Só 5 chaves são obrigatórias (`id`, `statement`, `created_at`, `body_hash`, `schema_version`);
as outras 20 são **omitidas** quando vazias — nunca `null`.

### Campos que você usa no dia a dia

| Campo | O que é |
|---|---|
| `statement` | A **afirmação**, curta e autocontida (chave TOON de `--summary`) |
| `type` | A **espécie** da nota (10 valores; omitido em grupos) |
| `classification` | `foundational` \| `tactical` \| `observational` — governa a expiração |
| `status` | `active` \| `in_progress` \| `blocked` \| `closed` \| `superseded` \| `forgotten` |
| `scope` | Nível de trabalho: `epic` \| `issue` \| `task` (só itens de trabalho têm) |
| `tags` | Rótulos livres (repetíveis) |
| `anchors` | Arquivos/globs externos que a nota toca — o **único** vínculo com o código |
| `outcomes` | Evidência (`success`/`partial`/`failure`/`abandoned`) anexada |
| `revision` | Contador de revisões (incrementa a cada `update`) |

### Espécies (`type`)

`fact`, `decision`, `question`, `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk`.

> **Grupo (épico).** Um grupo é uma nota com `scope: epic` e **sem** `type`; o tipo efetivo
> (`epic`) é **derivado**. Por isso o enum de espécies tem 10 valores, mas a CLI aceita `epic`
> onde faz sentido. Ver [`kd task`](06-task.md).

### Arestas explícitas

São 8 e só existem via `kd write --link FROM:ARESTA:TO` (ou no lote/`--params`):

`references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`,
`results_in`.

`depends_on` é a base das views `ready`/`blocked` e do impacto; `results_in` liga um épico aos
filhos (fallback: `depends_on`). Arestas não são inferidas — o `learn` apenas **sugere**.

## 4. IDs e hashes

- **`id = <prefixo>_<base36(8)>`**, endereçado por `type + U+001F + normalize(statement)`. O
  **prefixo é histórico**: reclassificar a nota **não** reescreve o id.
- `normalize` = NFC + trim + colapso de espaços. `body_hash = hex8(SHA-256(normalize(statement) +
  LF + normalize(body)))`.
- Consequência: **a mesma afirmação gera o mesmo id** em qualquer máquina/clone — é o que torna o
  corpus mergeável pelo git (ver §9).

## 5. Layout do `.knudge/`

```
.knudge/
  config.toml            # config efetiva (precedência sobre a global)
  notas/<tipo>/<id>.md   # a verdade (um diretório por tipo; D150)
  notas/MAP.md           # índice materializado (kd knowledge map --write)
  eventos/events.jsonl   # log append-only
  templates.toml         # seções por tipo
  validators.toml        # catálogo de checks executáveis
  .idx/                  # derivado, reconstruível (fora do git)
  cache/                 # respostas caras / warm start (fora do git)
  emb_cache.jsonl        # cache vetorial versionado (opt-in)
  .locks/                # lock advisory local
```

O `kd init` cuida do `.git/info/exclude` (nunca do `.gitignore` versionado) e do bloco
`.gitattributes` que dá `merge=union` ao log de eventos e ao cache vetorial.

## 6. Ciclo de vida do conhecimento

1. **Dedup no `write`.** O score do `ask` decide: `< 0.75` cria, `0.75–0.92` faz merge,
   `≥ 0.92` rejeita. `--update` versiona; mudar `type`/`statement` cria novo id e **supersede**.
2. **Classificação → expiração.** `foundational` não expira; `tactical` e `observational` têm
   shelf-life. O `maintenance prune` **propõe** `forget` por shelf-life/decay (nunca age).
3. **`forget` → `restore` → `purge`.** Soft-delete marca `forgotten`; `--purge` remove de fato
   após a retenção. `forgotten`/`superseded` ficam **fora** do `ask` por padrão.

## 7. Configuração em dois níveis

| Nível | Caminho | Papel |
|---|---|---|
| **Global** | `~/.config/local/knudge/config.toml` | Template/default de todos os projetos |
| **Projeto** | `.knudge/config.toml` | Efetivo; tem precedência |

`kd config` valida cada chave contra um schema fechado (`embeddings.*`, `recall.*`, `dedup.*`,
`retention.*`, `decay.*`, `clusters.*`, `hooks.*`, `mcp.*`, `behavior.*`…). `behavior.strict`
promove `warnings` a erro. Ver [`kd config`](10-config.md).

## 8. Arquitetura em camadas

O código é isolado e documentado por **escopo temático**:

- **`knudge-core`** — núcleo **puro**: modelo, schema, TOON, retrieval, lifecycle, handoff,
  tarefas, saúde, embeddings. **Nunca** acessa terminal, `argv`, relógio/RNG global ou FS direto:
  depende de **portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`). Erros são
  valores (`enum Error`), sem `Box<dyn Error>` na API pública.
- **`knudge-cli`** (`kd`) — adaptadores de borda: `clap`, envelope `--json`, logging, montagem das
  portas reais.
- **`knudge-mcp`** (`knudge-mcp`) — servidor MCP (JSON-RPC sobre stdio) que expõe os mesmos verbos
  a um agente.

Essa separação garante **determinismo** (o core usa fakes em teste) e mantém a lógica testável
sem I/O. Detalhes em [`ARCHITECTURE.md`](../ARCHITECTURE.md) e
[`plan/implementation/14_revisao_tecnica.md`](../plan/implementation/14_revisao_tecnica.md).

## 9. Multi-dev (sincronização)

Premissa: os devs usam **o mesmo modelo e as mesmas configs** de embeddings.

- **Notas** — endereçadas por conteúdo, um arquivo por nota → o git faz merge natural. O único
  conflito real é a **mesma `statement` com corpos divergentes**: o arquivo fica com marcadores de
  conflito, o TOON não parseia, a leitura tolerante **pula** a nota e o `doctor` a reporta.
- **Índice** (`.idx/`) — derivado; cada clone reconstrói após o `pull`.
- **Eventos e cache vetorial** — `merge=union` + dedup; a chave lógica do cache é
  `(body_hash, model)`.

Nunca "resolvemos" um conflito num artefato derivado: **re-derive**.

## 10. Decisões de projeto (Dxx)

O knudge registra cada decisão fechada em
[`plan/03_decisoes-fechadas.md`](../plan/03_decisoes-fechadas.md) (`D01`–`D153`). As que mais
aparecem no uso:

- **D47** — o knudge **propõe, nunca age em silêncio**: `learn`/`compact`/`prune` são read-only.
- **D95** — `id` derivado do conteúdo (não reescreve ao reclassificar).
- **D134/D149** — `epic` é a raiz; `issue` opcional; `scope=plan` e `type=container` removidos.
- **D140/D147** — o posicional é **conteúdo**; `--summary` é a afirmação; `--params` universal.
- **D143/D144** — operação que varre o corpus **exige escopo** (ou `--universe`).
- **D146** — `ask` é **conhecimento** por padrão; `--with-task` inclui trabalho.
- **D150** — layout material `notas/<tipo>/<id>.md` + `MAP.md` + hubs.
- **D148/D153** — cache vetorial versionado e sincronização multi-dev.

## 11. Convenções da CLI

- **`stdout` = dados** (pipe/`--json`); **`stderr` = logs**. Nunca se misturam.
- **`--json`** → `{success, command, data?, error?, warnings?}`.
- **Exit codes:** `2` invalid, `3` not_found, `4` conflict, `5` io, `6` timeout, `7` config,
  `8` schema, `70` internal (`101` é reservado a panic). EPIPE (pipe fechado) → **exit 0**.
- **Posicional = conteúdo** (`write`/`task new` = corpo; `ask` = consulta); `-` lê de stdin.
- **Degradação graciosa:** canal opcional que falha vira `warnings[]` + resultado parcial; com
  `behavior.strict=true`, vira erro.

## Próximo passo

➡️ [Quickstart](00-quickstart.md) · [Comandos](README.md#comandos-um-guia-por-comando)
