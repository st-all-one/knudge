# D140 — convenção universal do posicional: ele é conteúdo, nunca metadado

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão da superfície. **Universal** — vale para todos os verbos do `kd`.

## 1. Regra

O argumento **posicional** de um verbo do `kd` é **sempre uma string de conteúdo**, nunca um
identificador nem um metadado:

- `kd write` / `kd task new` → o posicional é o **corpo** (Markdown, a verdade do knudge).
- `kd ask` → o posicional é a **consulta** (consulta direta).
- Qualquer verbo que **não cria nem consulta** → **rejeita** posicional (`invalid_input` = 2).

Todo o resto vira **flag com nome semântico** (`--id`, `--summary`, `--type`, `--tag`, …), e
**some qualquer flag que altere o corpo** — em particular **`--body`** (o corpo é o posicional).

## 2. Mapeamento por verbo

| Verbo | Antes | Depois |
|---|---|---|
| `kd write` | `write [STATEMENT...] --body TXT` | `write [BODY] --summary TXT [--type ...]` |
| `kd task new` | `task new [STATEMENT...] --body TXT` | `task new [BODY] --summary TXT [--scope ...]` |
| `kd ask` | `ask [QUERY]` | `ask [QUERY]` (posicional = consulta) |
| `kd task show` | `task show [ID...]` | `task show --id ID...` |
| `kd task update` | `task update <ID>` | `task update --id ID` |
| `kd task close` | `task close <ID>` | `task close --id ID` |
| `kd forget` | `forget <ID>` | `forget --id ID` |
| `kd config get/set/unset` | `config get <KEY>` / `set <KEY> <VALUE>` | `config get --key K` / `set --key K --value V` |
| `prime`, `init`, `rewind`, `sync`, `knowledge`, `maintenance`, `self`, `task list/graph` | — | rejeitam posicional |

## 3. Nomes confirmados

| # | Questão | Decisão |
|---|---|---|
| O1 | Flag da afirmação (chave TOON `statement`) | **`--summary`** (`statement` fica como chave TOON) |
| O2 | `ask` | posicional = **consulta** |
| O3 | Corpo via stdin | posicional **`-`** |

## 4. Entrada do corpo (multilinha)

O corpo é o insumo mais frequente; a entrada precisa ser fácil por **pipe** e **heredoc** (o shell
já fornece o `EOF`), sem inventar sintaxe:

```
kd write --summary S "corpo curto inline"
echo "corpo" | kd write --summary S -            # pipe explícito
cat body.md | kd write --summary S               # sem posicional + stdin não-TTY → lê stdin
kd write --summary S - <<'EOF'                   # heredoc (EOF é do shell)
linha 1
linha 2
EOF
kd write --summary S - < body.md                 # arquivo via redirecionamento
```

- **`-`** = stdin explícito.
- **Sem posicional + stdin não-TTY** = lê o corpo do stdin (cobre `cat x | kd write`).
- **Sem posicional + stdin TTY** = **erro** (`invalid_input`), sem bloquear (D130 — falha alto).
- **Não** haverá token `EOF` próprio nem parsing de `"""`: o heredoc do shell resolve, e um
  delimitador custom seria ambíguo e frágil. O corpo é bytes crus até o fim do stdin.

## 5. Consequências

- `--body` some de `WriteArgs`/`TaskNewArgs`; o posicional preenche `Draft.body`/`TaskSpec.body`;
  `--summary` preenche `statement`/`Draft.statement`.
- Ids saem do posicional e viram `--id` (repetível/com vírgula onde há vários).
- `write --batch` (JSONL) e `--from` do plano **não** mudam (são flags de fonte).
- Mensagens de uso/`prime` e goldens de `--help` atualizados; verbos sem conteúdo que recebem
  posicional → exit 2.

## 6. Raio de alcance (quando executar)

- `cli/mod.rs` (`WriteArgs`, `AskArgs`, `ForgetArgs`), `cli/task.rs` (`TaskNewArgs`,
  `TaskCommand::Show/Update/Close`), `cli/config` (get/set/unset), `commands/*` correspondentes,
  leitura de stdin (reusar `read_body`/`read_source`), `docs/*`, `prime.rs`, `SKILL.md`, goldens.

## 7. Aceite

- [ ] `kd write --summary S "corpo"` grava `statement=S`, `body=corpo`.
- [ ] `echo C | kd write --summary S -` e `cat body.md | kd write --summary S` gravam o corpo.
- [ ] `kd write --summary S - <<'EOF' … EOF` grava o corpo multilinha.
- [ ] Sem posicional e com TTY → exit 2 (não bloqueia).
- [ ] `kd task new --summary S "corpo"` idem; `--body` inexistente (exit 2).
- [ ] `kd task show --id A --id B` / `kd forget --id X`; posicional neles → exit 2.
- [ ] `kd sync foo` / `kd prime foo` → exit 2.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
