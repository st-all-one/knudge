# Handoff e `rewind`

`prime` é o **protocolo estático** (byte-idêntico por versão — D57); `rewind` é o **estado
dinâmico**: situa o próximo agente/rodada com o mínimo de tokens e um `context_id` retomável 1:1.

- Código: `crates/knudge-core/src/handoff/`
- Decisões: D40/D41, D57/D58, D82, D88, D106, D143, D162

## Modos (`RewindMode`)

| Modo | O que devolve |
|---|---|
| `Manifest` | manifest curto (~30 tokens) — estado do projeto. |
| `Scope(String)` | escopo por container/domínio. |
| `Files(Vec<String>)` | working set a partir de arquivos alterados (`--files`). |
| `Auto` | detecta o escopo automaticamente. |

## Manifest e trust-tier (D106)

- `manifest.rs` ranqueia os itens por **trust-tier** (foundational → decision → …).
- `next.rs` adiciona as linhas **dinâmicas**: `next:` (tarefas `ready` abertas por impacto) e
  `fresh:` (`stale`/`expiring`/`pending`). O `prime` permanece estático (D57).
- Anexa o **corpo** de notas `foundational`/`decision` (D162).

## Orçamento sem tokenizer (D40/D82)

- Heurística `ceil(len/4)` tokens; `DEFAULT_BUDGET = 4000`, `MIN_TAIL = 100`.
- Aplicado **item a item**: trunca o último e ignora sobra < 100 tokens.
- `budget.rs`: `estimate_tokens`, `Budgeted`, `apply_into`, `BudgetSummary`.

## `context_id` (D88)

- `context.rs`: `derive_id` gera `ctx_<base36>` (`CONTEXT_PREFIX = "ctx"`); `is_valid_context_id`.
- `kd rewind --resume <id>` devolve o **mesmo contexto 1:1**, sem re-busca — handoff reprodutível
  entre agentes/rodadas. `ContextStore` persiste o contexto endereçável.

## Escopo automático (D41/D143)

- `scope.rs`: auto-context-scope (`git status -uall` + active work) e auto-flip
  (`FLIP_NOTES`/`FLIP_CONTAINERS`: >100 notas / >5 containers).
- Filtros de corpus (`--tag`/`--anchor`/`--type`/`--class`/`--around`) escopam o handoff (D143).
- `impact`/`compute_views` são computados **uma vez** (E15-T09).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Orquestração `rewind` | `handoff/mod.rs` |
| Manifest/trust-tier | `handoff/manifest.rs` |
| `next:`/`fresh:` | `handoff/next.rs` |
| Escopo/auto-flip | `handoff/scope.rs` |
| Orçamento | `handoff/budget.rs` |
| `context_id` | `handoff/context.rs` |

## Testes

`handoff/tests/` (manifest, next, scope, budget, context, rewind).
