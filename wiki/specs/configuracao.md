# Configuração

Como o knudge lê e escreve configuração em **dois níveis** com um **codec TOML próprio** (sem
dependência externa), mantendo precedência, segredos e diff mínimo.

- Código: `crates/knudge-core/src/config/`
- Decisões: D61–D64, D91, D94, D97

## Dois níveis (D61–D64)

| Nível | Caminho | Papel |
|---|---|---|
| **Global** | `~/.config/local/knudge/config.toml` (Linux), `~/Library/Application Support/knudge/config.toml` (macOS), `%APPDATA%\knudge\config.toml` (Windows) | template/default curado pelo usuário |
| **Projeto** | `.knudge/config.toml` | efetivo, **tem precedência** |

- O projeto **clona** o global na instanciação (cópia literal — D62); merge por chave fica como
  evolução futura.
- `Config::effective(global, project)` mescla os dois e **remove segredos do projeto** (D91): o
  projeto nunca carrega credenciais — `api_key_env` e afins vivem **só no global**.
- `config set/unset` **valida contra o schema**, poda ancestrais vazios e revalida (D64).
- `strict` é config de **projeto** (`[behavior] strict`), não flag de CLI (D94); promove
  `warnings[]` a erro.

## Schema (D64)

O catálogo de chaves conhecidas, tipos e defaults vive em `config/schema/keys.rs` +
`config/schema/keys_embeddings.rs` (`KEYS`, `KeySpec`, `Kind`, `default_table`). `Config::validate`
recusa chave desconhecida e tipo errado; `config set` rejeita valor fora do schema. Grupos
principais: `recall.*`, `retrieval.*`, `embeddings.*`, `mcp.*`, `rules.*`, `proposals.*`,
`clusters.*`, `suggestions.*`, `retention.*`, `write.*`, `task.*`, `programs.*`, `behavior.*`,
`[secrets]`.

## Codec TOML próprio (D97)

Subset implementado em `config/toml/` — **sem `toml` crate**:

- **Aceita:** comentários, `[seção]`/`[seção.sub]`, chaves bare/citadas/pontilhadas, strings
  básicas/literais de **uma linha**, inteiros com `_`, floats, booleanos e listas (inclusive
  multilinha).
- **Rejeita:** `[[array-of-tables]]`, strings multilinha e `null` — com erro `config` (exit 7).
- **Leitura preserva a ordem** (para diff mínimo); a **escrita é canônica** (D63): ordem de
  inserção e *quoting* estáveis, então `config set` gera diff de uma linha.

O mesmo codec serve o catálogo de validators (`.knudge/validators.toml`, D99) e os templates de
plano (`.knudge/templates.toml`, D105) — um só parser para toda config estruturada.

## Fluxo

1. `Session::open` resolve o projeto e carrega `global` + `projeto`.
2. `Config::effective` aplica precedência e remove segredos.
3. O domínio lê via `get`/`get_bool`/`get_int`/`get_float`/`get_str` (nunca lê o arquivo direto).
4. `config set/unset/save` reescrevem o arquivo do nível escolhido preservando ordem.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| `Config`, `effective`, load/save | `config/mod.rs` |
| Schema de chaves | `config/schema.rs`, `config/schema/keys*.rs` |
| Valores e tabelas ordenadas | `config/value.rs`, `config/table.rs` |
| Codec TOML | `config/toml/{mod,emit,parse/}` |

## Testes

`config/tests.rs`, `config/toml/tests.rs` — round-trip (ordem/quoting), rejeição do não-suportado,
precedência e remoção de segredos.
