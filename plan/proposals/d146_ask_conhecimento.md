# D146 — `kd ask` só conhecimento e superfície enxuta

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão de `docs/03-ask.md` + `kd ask --help`. Revisa **D39/D107/D121**.
> A/B/C e a redução (D) confirmados.

## 1. Decisões confirmadas

- **A — `ask` é conhecimento.** Por padrão, só notas **sem `scope`** (conhecimento). Itens de
  trabalho entram **só com `--with-task`**. Separa `ask` (conhecimento) de `task list` (trabalho),
  alinhando com o eixo `scope` (D134/D142).
- **B — `--rank` exige escopo.** `ask --rank` (varredura do corpus) exige um filtro de escopo ou
  `--universe` (princípio D143/D144). Sem escopo → exit 2 (D130).
- **C — `--with-body` → `--full-content`.** Alinha com `task list --full-content` (D137);
  `--brief` continua sendo o mínimo.

## 2. Redução da superfície (D — confirmada)

Hoje o `ask --help` tem ~19 opções e 5 modos. Proposta: `ask` = **recall/get/expand**; o resto
migra para `kd knowledge`.

**`kd ask` (conhecimento):**

```
kd ask <QUERY> [filtros] [--limit N] [--brief|--full-content] [--with-task]
kd ask --id <ID>...
kd ask --around <ID> [--via ARESTA] [--depth N]

filtros: --type --class --tag --status --scope --anchor --since --until
saída:   --limit --brief --full-content --json
```

**Migra para `kd knowledge`:**
- `kd knowledge rank [filtros]` (ex-`ask --rank`) — exige escopo/`--universe` (B).
- `kd knowledge tags` (ex-`ask --tags`) — vocabulário `tag|count`.

**Renomes:**
- `--with-body` → `--full-content`.
- `--container` → `--scope` (consistente com D143/D144).

`kd knowledge` fica: **`map` · `digest` · `rank` · `tags`**. `ask` fica com 3 modos e uma
superfície focada em achar/ler.

## 3. Bugs de doc (propagação, sem decisão)

- **`why`**: o conjunto real é `file_match|anchor_match|tracker_match|stars|semantic|recent|universal`
  — a doc diz `lexical`/`anchor`, que **não existem**.
- **`--via refines`**: não há `refines` no vocabulário (8 `EdgeKind`); é `extends`. Corrigir em
  `docs/03-ask.md` e `docs/04-write.md`.

## 4. Raio de alcance (quando executar)

- `cli/mod.rs` (`AskArgs`: `--with-task`, `--full-content`, `--scope`, remove `rank`/`tags`),
  `commands/ask/*` (default conhecimento + `--with-task`; validação de escopo no rank),
  `cli/knowledge.rs` + `commands/knowledge/{rank,tags}.rs` (novos), `retrieval/filter.rs`
  (filtro por `scope`), `docs/03-ask.md`/`04-write.md`, `prime.rs`, goldens/testes.

## 5. Aceite

- [x] `ask "<q>"` devolve só conhecimento; `--with-task` inclui trabalho.
- [x] `ask --rank` sem escopo e sem `--universe` → exit 2.
- [x] `ask --full-content` substitui `--with-body`; `--container` virou `--scope`.
- [x] `kd knowledge rank`/`tags` substituem os modos do `ask`.
- [x] `why` e `--via` corretos na doc; `make check` verde.
