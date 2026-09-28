# 13 · `kd config` — configuração

## Para que serve

Lê e ajusta a configuração do knudge em **dois níveis**:

| Nível | Onde fica | Papel |
|---|---|---|
| **Global** | `~/.config/local/knudge/config.toml` | template/default de todos os projetos |
| **Projeto** | `.knudge/config.toml` | config efetiva; tem precedência |

Cada chave é validada contra um catálogo fechado — não há chave livre. O subcomando `promote`
transforma conhecimento maduro em **regras governadas** do `AGENTS.md`.

## Quando usar

- **Use `list`/`get`** para inspecionar o comportamento atual.
- **Use `set`/`unset`** para ajustar limites, ligar/desligar a busca semântica ou configurar hooks.
- **Use `promote`** quando quiser fixar algumas decisões como regras do projeto.

## Sintaxe

```
kd config get   --key <CHAVE> [--global]
kd config set   --key <CHAVE> --value <VALOR> [--global]
kd config unset --key <CHAVE> [--global]
kd config list  [--global]
kd config promote <recommend|approve|edit|remove|list>
```

## Exemplos

### 1. Inspecionar

```bash
# 1. tudo
kd config list

# 2. uma chave
kd config get --key recall.default_limit

# 3. o template global
kd config get --key mcp.hints_cap --global
```

### 2. Ajustar o comportamento

```bash
# 1. menos resultados por busca
kd config set --key recall.default_limit --value 3

# 2. desligar a busca semântica
kd config set --key recall.semantic --value false

# 3. avisos viram erro
kd config set --key behavior.strict --value true
```

### 3. Configurar no nível global

```bash
# 1. default para todos os projetos
kd config set --key mcp.hints_cap --value 3 --global

# 2. confirmar
kd config get --key mcp.hints_cap --global

# 3. recopiar para o projeto
kd init --force
```

### 4. Remover um ajuste

```bash
# 1. voltar ao default
kd config unset --key recall.default_limit

# 2. remover do global
kd config unset --key mcp.hints_cap --global

# 3. conferir
kd config get --key recall.default_limit
```

### 5. Promover conhecimento a regra

```bash
# 1. ver candidatas (read-only)
kd config promote recommend --universe

# 2. aprovar uma nota
kd config promote approve decision_01abc --universe

# 3. editar a regra promovida
kd config promote edit decision_01abc --summary "Regra revisada"
kd config promote remove decision_01abc --universe
kd config promote list
```

As regras vão para um bloco governado do `AGENTS.md`. O recurso é desligado por padrão e respeita
um teto de regras.

## Flags

| Comando | Flags |
|---|---|
| `get` | `--key <CHAVE>`, `--global` |
| `set` | `--key`, `--value`, `--global` |
| `unset` | `--key`, `--global` |
| `list` | `--global` |
| `promote recommend` | `--universe`, `--limit` |
| `promote approve`/`remove` | `--id`, `--universe` |
| `promote edit` | `--id`, `--summary`, `--universe` |

## Chaves mais usadas

| Chave | Default | Para quê |
|---|---|---|
| `recall.default_limit` | `5` | Resultados padrão do `ask` |
| `recall.semantic` | `true` | Liga/desliga o canal semântico |
| `recall.semantic_weight` | `30.0` | Peso do canal semântico |
| `recall.anchor_weight` | `2.0` | Peso do canal de âncoras |
| `dedup.create_below` / `dedup.merge_below` | `0.75` / `0.92` | Limiares do `write` |
| `write.batch_max` / `task.batch_max` | `100` | Teto de lote |
| `retention.*` | — | Validade por classificação |
| `decay.anchor_threshold` / `decay.grace_days` | `0.5` / `30` | Detecção de âncora quebrada |
| `behavior.strict` | `false` | Promove avisos a erro |
| `embeddings.provider` | `"http"` | `http`/`lightweight`/`none` |
| `embeddings.model` / `embeddings.dimensions` | granite / `384` | Identidade do modelo |
| `embeddings.mode` | `"lazy"` | `lazy`/`manual` |
| `embeddings.endpoint` | `:8889` | Endereço do servidor |
| `mcp.observation_mode` / `mcp.hints_cap` | `true` / `3` | Servidor MCP |

A lista completa sai de `kd config list`.

## Resultado esperado

- **`get`** — `chave = valor`.
- **`set`** — `chave = valor` (validado contra o catálogo).
- **`unset`** — confirmação.
- **`list`** — uma linha por chave.
- Chave ausente → exit 3; valor inválido → exit 7.

## Quando não usar

- Para editar notas ou o corpus → [`kd write`](06_write.md).
- `strict` é configuração, não uma flag de comando.

## Veja também

➡️ [Embeddings](18_embeddings.md) · [`kd drain`](12_drain.md) · [`kd self`](16_self.md)
