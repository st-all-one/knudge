# `kd config` — configuração

## O que faz

Lê e escreve a **configuração em dois níveis** do knudge:

| Nível | Caminho | Papel |
|---|---|---|
| **Global** | `~/.config/local/knudge/config.toml` | Template/default de todos os projetos |
| **Projeto** | `.knudge/config.toml` | Efetivo; tem precedência |

Cada chave é validada contra um **schema fechado** — não há chave livre. O `kd init` copia o
template global para o projeto; o `--global` mira o template.

## Em 30 segundos

```bash
kd config list
kd config get --key recall.default_limit
kd config set --key recall.default_limit --value 3
kd config unset --key recall.default_limit
```

## Uso

```
kd config get   --key <CHAVE> [--global]
kd config set   --key <CHAVE> --value <VALOR> [--global]
kd config unset --key <CHAVE> [--global]
kd config list  [--global]
```

## Exemplos

### Nível 1 — inspecionar

```bash
kd config list
```

```
recall.default_limit = 5
recall.semantic = true
embeddings.provider = "http"
behavior.strict = false
...
```

### Nível 2 — ajustar comportamento

```bash
kd config set --key recall.default_limit --value 3
kd config set --key recall.semantic --value false     # desliga o canal vetorial
kd config set --key behavior.strict --value true      # warnings viram erro
```

### Nível 3 — template global

```bash
kd config set --key mcp.hints_cap --value 3 --global
kd config get --key mcp.hints_cap --global
```

Use `--global` para o default de **todos** os projetos; `kd init --force` recopia para o projeto.

### Nível 4 — hooks e validators

```bash
kd config set --key hooks.pre_record --value "./scripts/check-note.sh"
kd config set --key hooks.timeout_ms --value 30000
```

Hooks são comandos externos que podem **bloquear/mutar** o write (saída não-zero bloqueia). O
catálogo de `checks` executáveis fica em `.knudge/validators.toml`.

## Chaves mais usadas

| Chave | Default | Para quê |
|---|---|---|
| `recall.default_limit` | `5` | Hits padrão do `ask` |
| `recall.semantic` | `true` | Liga/desliga o canal vetorial |
| `recall.semantic_weight` | `30.0` | Peso do canal vetorial no RRF |
| `recall.confirmation_from_tasks` | `0.1` | Boost de tarefas que confirmam |
| `dedup.create_below` / `dedup.merge_below` | `0.75` / `0.92` | Limiares do `write` |
| `write.batch_max` / `task.batch_max` | `100` | Teto de `--batch` |
| `retention.*` | — | Shelf-life por classificação (`foundational`/`tactical`/`observational`/`retired`) |
| `decay.anchor_threshold` / `decay.grace_days` | `0.5` / `30` | Drift de âncoras |
| `behavior.strict` | `false` | Promove `warnings` a erro |
| `embeddings.provider` | `"http"` | `http`/`lightweight`/`none` |
| `embeddings.model` / `embeddings.dimensions` | granite / `384` | Identidade do modelo |
| `embeddings.mode` | `"lazy"` | `lazy`/`manual` |
| `embeddings.cache` / `version_cache` | `true` / `false` | Cache (versionado ou não) |
| `mcp.observation_mode` / `mcp.hints_cap` | `true` / `3` | Servidor MCP |
| `knowledge.persist_in_project` | `true` | Versiona `notas/`/`eventos/` |

A lista completa (com tipos e defaults) está em
[`plan/00_panorama.md`](../plan/00_panorama.md) §3 e no próprio `kd config list`.

## Resultados

- `get` — `chave = valor`; `{key, value, scope}`.
- `set` — `chave = valor`; valida contra o schema.
- `unset` — `chave removida`.
- `list` — uma linha por chave.
- Chave ausente → exit 3; chave/valor inválido → exit 7 (config).

## Quando (não) usar

- **Use** para ajustar limiares, embeddings e hooks.
- **Não use** para editar notas nem o corpus; `strict` é config, não flag (D94).

## Próximo passo

➡️ [`kd forget`](11-forget.md) · [Embeddings](15-embeddings.md)
