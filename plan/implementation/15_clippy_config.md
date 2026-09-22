# Configuração de rigor do Clippy — análise e recomendação

> **Fonte:** <https://doc.rust-lang.org/clippy/lint_configuration.html> — seção
> *“Lint Configuration Options”*, **95 opções** de configuração (chaves de `clippy.toml`).
> Captura e parsing feitos nesta revisão; cada opção foi analisada individualmente abaixo.
>
> **Alvo:** projeto knudge (Rust **1.97.0**, Edition **2024**), com a disciplina de D92 e as
> políticas `R01–R43` de [`14_revisao_tecnica.md`](14_revisao_tecnica.md). Meta: **rigor e
> padronização globais**, maximizando **segurança de memória, recursos, logs e tratamento de
> erros**, sem transformar o código em um campo minado de falsos positivos.
>
> **Resultado pronto para copiar:** [`clippy.toml`](clippy.toml) (valores) e §5
> ([`Cargo.toml [workspace.lints]`](#5-níveis-de-lint-workspacelints)).

---

## 0. Duas camadas, não uma

A página trata apenas de **valores de configuração** (`clippy.toml`). Um quadro completo precisa
das duas:

1. **Níveis** (`Cargo.toml` → `[workspace.lints.rust]` / `[workspace.lints.clippy]`): o que é
   `deny`/`warn`/`allow`. **Sem isso, a maioria das opções abaixo não tem efeito**, porque as
   lints correspondentes são `allow` por padrão (`restriction`, parte de `pedantic`/`nursery`).
2. **Valores** (`clippy.toml`): os limites e allowlists que essas lints usam.

> Regra prática: **toda opção recomendada tem a lint correspondente ligada** em §5.

---

## 1. Metodologia

- Cada opção recebe **Default → Recomendado → Fundamento**, com o eixo de impacto
  (**mem** = memória/unsafe · **rec** = recursos · **log** = logs/observabilidade ·
  **err** = erros · **est** = estilo/padronização · **perf** = desempenho).
- Priorizamos: (a) `forbid`/`deny` de construtos perigosos; (b) limites **mais baixos** que o
  default para complexidade/tamanho; (c) **allowlists explícitas** em vez de silêncio;
  (d) **determinismo** (Clock/Env ports, nada de iteração de `HashMap`).
- Onde o default já é o mais rigoroso, registramos **“manter default”**.

---

## 2. Análise individual das 95 opções

### 2.1 Unsafe, unwrap/expect/panic e testes

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `accept-comment-above-attributes` | `true` | **`false`** | mem | O `// SAFETY:` deve ficar colado ao bloco, não acima de `#[allow]` (R01). |
| `accept-comment-above-statement` | `true` | **`false`** | mem | Idem: comentário imediatamente no statement do `unsafe`. |
| `allow-expect-in-consts` | `true` | **`false`** | mem/err | D92 vale também em `const`. |
| `allow-expect-in-tests` | `false` | **manter `false`** | err | Testes usam `Result`/`?`; nada de `expect`. |
| `allow-unwrap-in-consts` | `true` | **`false`** | mem/err | Idem para `unwrap`. |
| `allow-unwrap-in-tests` | `false` | **manter `false`** | err | Idem. |
| `allow-panic-in-tests` | `false` | **manter `false`** | err | `assert!` não é afetado; `#[should_panic]` continua válido. |
| `allow-unwrap-types` | `[]` | **`[]`** | err | Nenhum tipo isento; locks usam `into_inner` (R32), não `unwrap`. |
| `allow-dbg-in-tests` | `false` | **manter `false`** | log | `dbg!` nunca. |
| `allow-print-in-tests` | `false` | **manter `false`** | log | Testes não escrevem em stdout/stderr. |
| `allow-useless-vec-in-tests` | `false` | **manter `false`** | perf | `vec!` desnecessário é desperdício também no teste. |
| `allow-indexing-slicing-in-tests` | `false` | **manter `false`** | mem | Indexação pode panicar; testes usam `get`. |
| `allow-large-stack-frames-in-tests` | `true` | **manter `true`** | rec | Fixtures de teste podem ter arrays grandes; não penalizar. |

### 2.2 Estilo e padronização global

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `allow-mixed-uninlined-format-args` | `true` | **`false`** | est | Obriga inlining total em `format!`/`write!`. |
| `allow-one-hash-in-raw-strings` | `false` | **manter `false`** | est | `r#""#` só quando necessário. |
| `unreadable-literal-lint-fractions` | `true` | **manter `true`** | est | Separadores também na fração (`1_000.000_1`). |
| `upper-case-acronyms-aggressive` | `false` | **`true`** | est | Nomes como `HttpUrl` em vez de `HTTPUrl`. |
| `enforce-iter-loop-reborrow` | `false` | **`true`** | perf | Evita reborrow implícito em `for`; explicita `iter_mut`. |
| `lint-commented-code` | `false` | **`true`** | est | `if` colapsável continua lintado mesmo com comentários. |
| `semicolon-inside-block-ignore-singleline` | `false` | **manter `false`** | est | Sem `;` supérfluo em bloco. |
| `semicolon-outside-block-ignore-multiline` | `false` | **manter `false`** | est | Consistência de `;`. |
| `module-items-ordered-within-groupings` | `"none"` | **`"all"`** | est | Ordem alfabética dentro de cada grupo de itens. |
| `warn-on-all-wildcard-imports` | `false` | **`true`** | est | Proíbe `use foo::*` (exceto preludes, que a lint já isenta). |
| `allowed-wildcard-imports` | `[]` | **`[]`** (sem efeito com o acima) | est | Explícito: nenhum wildcard tolerado. |
| `enforced-import-renames` | `[]` | **`[]`** | est | Sem aliases obrigatórios por ora. |
| `allow-exact-repetitions` | `true` | **manter `true`** | est | `foo::Foo` é legítimo. |
| `allowed-prefixes` | `to/as/into/from/...` | **manter default** | est | Boa heurística para `module_name_repetitions`. |
| `allow-renamed-params-for` | `From/TryFrom/FromStr` | **manter default** | est | Idem. |
| `standard-macro-braces` | `[]` | **ver §4** | est | Uniformiza `vec![]`, `format!()`, asserts etc. |
| `source-item-ordering` | `enum/impl/module/struct/trait` | **manter default** | est | Ordem já opinativa. |
| `trait-assoc-item-kinds-order` | `const/type/fn` | **manter default** | est | Idem. |
| `module-item-order-groupings` | grupo padrão | **manter default** | est | Boa organização; não sobrepor. |
| `literal-representation-threshold` | `16384` | **manter `16384`** | est | Só literais decimais enormes. |
| `const-literal-digits-threshold` | `30` | **manter `30`** | est | Idem. |
| `verbose-bit-mask-threshold` | `1` | **manter `1`** | perf | Sugere `trailing_zeros` cedo. |
| `pub-underscore-fields-behavior` | `"PubliclyExported"` | **manter default** | est | Campo `_x` público continua lintado. |
| `recursive-self-in-type-definitions` | `true` | **manter `true`** | est | Usa `Self` em tipos recursivos. |

### 2.3 Complexidade, tamanho e API

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `cognitive-complexity-threshold` | `25` | **`15`** | est/perf | Funções pequenas e testáveis; casa com arquivos ≤300 linhas (D92). |
| `type-complexity-threshold` | `250` | **`150`** | est | Tipos ilegíveis viram aliases. |
| `too-many-lines-threshold` | `100` | **`60`** | est | Função curta por padrão. |
| `too-many-arguments-threshold` | `7` | **`5`** | est | Muitos parâmetros → struct/Builder. |
| `excessive-nesting-threshold` | `0` (desligado) | **`3`** | est | Profundidade máxima de blocos. |
| `max-trait-bounds` | `3` | **manter `3`** | est | Limita repetição em bounds. |
| `max-suggested-slice-pattern-length` | `3` | **manter `3`** | est | Sugestões de slice pattern legíveis. |
| `single-char-binding-names-threshold` | `4` | **`2`** | est | Menos `a`, `b`, `c` soltos. |
| `min-ident-chars-threshold` | `1` | **`2`** | est | Nomes mínimos de 2 chars (com allowlist). |
| `allowed-idents-below-min-chars` | `i/j/x/y/z/w/n` | **+ `id/io/fs/db/ui/ok/err/ty/kv` e `..`** | est | Não luta contra nomes consagrados. |
| `max-fn-params-bools` | `3` | **`0`** | est/perf | Bool como parâmetro → enum com semântica. |
| `max-struct-bools` | `3` | **`1`** | est | Mais de um bool indica estado mal modelado. |
| `enum-variant-size-threshold` | `200` | **`128`** | mem/perf | Evita enum grande; favorece `Box`. |
| `enum-variant-name-threshold` | `3` | **manter `3`** | est | Só aponta repetição com 3+ variantes. |
| `struct-field-name-threshold` | `3` | **manter `3`** | est | Aponta repetição de campo com 3+ campos. |
| `avoid-breaking-exported-api` | `true` | **`false`** | est | Crate interno (`publish = false`): aceitar sugestões mais duras. |
| `inherent-impl-lint-scope` | `"crate"` | **manter `"crate"`** | est | Impls inerentes duplicados evitados na crate inteira. |
| `check-private-items` | `false` | **`true`** | est | Cobrar docs (`missing_*`) também no privado. |
| `check-incompatible-msrv-in-tests` | `false` | **`true`** | est | Testes também respeitam a MSRV. |
| `check-inconsistent-struct-field-initializers` | `false` | **manter `false`** | est | Evita falso positivo com efeitos colaterais. |
| `check-grouped-late-init` | `true` | **manter `true`** | est | `let` tardio agrupado → tupla. |
| `missing-docs-allow-unused` | `false` | **manter `false`** | est | Exige doc mesmo em campo `_x`. |
| `missing-docs-in-crate-items` | `false` | **manter `false`** | est | Documenta tudo, não só `pub(crate)`. |
| `msrv` | `current` | **`"1.97"`** | est | **MSRV obrigatório: 1.97+** (Edition 2024); habilita os checks de MSRV. |

### 2.4 Segurança de memória, recursos e concorrência

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `enable-raw-pointer-heuristic-for-send` | `true` | **manter `true`** | mem | Conservador ao decidir `Send`. |
| `array-size-threshold` | `16384` | **`4096`** | rec | Arrays grandes na pilha são risco de overflow. |
| `stack-size-threshold` | `512000` | **`65536`** | rec | Frame > 64 KiB é suspeito (R14). |
| `future-size-threshold` | `16384` | **`8192`** | rec | Futures gordos escondem cópias (R16). |
| `pass-by-value-size-limit` | `256` | **`128`** | perf | Passa por referência antes. |
| `too-large-for-stack` | `200` | **manter `200`** | rec | Acima disso, `Box`/heap. |
| `unnecessary-box-size` | `128` | **manter `128`** | perf | `Box<T>` pequeno desnecessário. |
| `vec-box-size-threshold` | `4096` | **manter `4096`** | perf | `Vec<Box<T>>` só vale para `T` grande. |
| `trivial-copy-size-limit` | `target_pointer_width` | **manter default** | perf | Ajusta `Copy` × referência por arquitetura. |
| `large-error-threshold` | `128` | **manter `128`** | perf | `Err` grande deve ir para `Box`. |
| `large-error-ignored` | `[]` | **`[]`** | perf | Nenhuma exceção. |
| `max-include-file-size` | `1000000` | **`262144`** | rec | Limita `include_bytes!`/`include_str!`. |
| `await-holding-invalid-types` | `[]` | **locks e `Ref`/`RefMut` (ver §4)** | mem/rec | Impede guard atravessando `.await` (R11/R16). |
| `arithmetic-side-effects-allowed` | `[]` | **`Wrapping/Saturating/f32/f64/atômicos`** | mem | Aritmética que não panic por overflow. |
| `arithmetic-side-effects-allowed-binary` | `[]` | **`[]`** | mem | Sem pares isentos adicionais. |
| `arithmetic-side-effects-allowed-unary` | `[]` | **`[]`** | mem | Negação continua checada (MIN overflow). |
| `suppress-restriction-lint-in-const` | `false` | **manter `false`** | mem | `indexing_slicing` vale em `const`. |
| `absolute-paths-allowed-crates` | `[]` | **`["std","core","alloc"]`** | est | Permite caminhos absolutos só da std. |
| `absolute-paths-max-segments` | `2` | **manter `2`** | est | A partir de 3 segmentos, importe. |
| `allowed-scripts` | `["Latin"]` | **manter `["Latin"]`** | est | Identificadores em alfabeto latino. |
| `allowed-dotfiles` | `[]` | **`[]`** | est | Nenhum dotfile extra permitido. |
| `ignore-interior-mutability` | `["bytes::Bytes"]` | **manter default** | mem | Tipos conhecidamente seguros como const. |
| `allowed-duplicate-crates` | `[]` | **`[]`** (adicionar só se o CI exigir) | est | Evita versões duplicadas no grafo. |
| `cargo-ignore-publish` | `false` | **manter `false`** | est | Uso interno; não desligar metadados. |

### 2.5 Listas de proibição (as de maior ganho)

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `disallowed-types` | `[]` | **`Rc`/`Weak`/`RefCell`/`Cell`/`LinkedList`/`HashMap`/`HashSet`** | mem/rec | Fecha R03 e o determinismo; `HashMap` com allow no adaptador. |
| `disallowed-methods` | `[]` | **`transmute`/`forget`/`zeroed`/`uninitialized`/`from_raw_parts`/`ptr::*`/`env::var`/`SystemTime::now`/`Instant::now`/`process::exit`** | mem/rec | Confina `unsafe` (R01) e força os ports `Clock`/`Env` (D65). |
| `disallowed-macros` | `[]` | **`dbg!`/`todo!`/`unimplemented!`/`unreachable!`/`panic!`** | err | Defesa em profundidade junto às lints de restrição. |
| `disallowed-names` | `foo/baz/quux` | **+ `tmp/thing/stuff/misc` e `..`** | est | Elimina nomes de rascunho. |
| `disallowed-fields` | `[]` | **`[]`** | est | Sem campos proibidos por ora. |

### 2.6 Documentação (padronização global)

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `doc-valid-idents` | lista da std | **+ termos do domínio e `..`** | est/log | `knudge`, `RRF`, `BM25`, `TOON`, `body_hash`, `context_id`, `nDCG@k`, etc. |

### 2.7 Correção e segurança complementar

| Opção | Default | Recomendado | Eixo | Fundamento |
|---|---|---|---|---|
| `allow-comparison-to-zero` | `true` | **manter `true`** | est | `x % y == 0` é legítimo; evitar falso positivo. |
| `matches-for-let-else` | `"WellKnownTypes"` | **manter default** | est | Filtra `matches!` só nos tipos conhecidos no `let-else`. |
| `allow-private-module-inception` | `false` | **manter `false`** | est | `foo::foo` continua lintado, mesmo privado. |
| `warn-unsafe-macro-metavars-in-private-macros` | `false` | **`true`** | mem | Metavars em `unsafe` de macros privadas também são avisadas (R01). |

---

## 3. Quadro geral — princípios da configuração

1. **Zero `unsafe` fora do adaptador** (R01): `forbid` nas crates puras; `// SAFETY:` exigido.
2. **Zero `unwrap/expect/panic`** (D92) em qualquer contexto — inclusive `const` e testes.
3. **Nada de aritmética que panic** (overflow) e **nada de `as`** (R02): `checked_*`/`From`.
4. **Determinismo por contrato**: `env::var`, `SystemTime/Instant::now` e iteração de
   `HashMap`/`HashSet` são proibidos (ports `Clock`/`Env`, `BTreeMap`/`IndexMap`).
5. **Recursos com teto**: pilha, futures, arrays, includes, erros, parâmetros por valor.
6. **stdout é dados; stderr é log** (R20): `print_stdout`/`print_stderr` = `deny`.
7. **Toda exceção tem motivo**: `allow_attributes_without_reason = "deny"`.
8. **Limites abaixo do default** para complexidade, tamanho e aninhamento.

---

## 4. `clippy.toml` recomendado

O arquivo completo e comentado está em **[`clippy.toml`](clippy.toml)**. Destaques de valores que
não são óbvios:

```toml
accept-comment-above-attributes = false
allow-expect-in-consts          = false
allow-unwrap-in-consts          = false
cognitive-complexity-threshold  = 15
too-many-lines-threshold        = 60
too-many-arguments-threshold    = 5
excessive-nesting-threshold     = 3      # 0 = desligado
max-fn-params-bools             = 0
max-struct-bools                = 1
stack-size-threshold            = 65536
array-size-threshold            = 4096
future-size-threshold           = 8192
msrv                            = "1.97"

await-holding-invalid-types = [
  "std::sync::MutexGuard", "std::sync::RwLockReadGuard",
  "std::sync::RwLockWriteGuard", "std::cell::Ref", "std::cell::RefMut",
]

arithmetic-side-effects-allowed = [
  "Wrapping", "Saturating", "f32", "f64",
  "AtomicU8", "AtomicU16", "AtomicU32", "AtomicU64", "AtomicUsize",
  "AtomicI8", "AtomicI16", "AtomicI32", "AtomicI64", "AtomicIsize",
]
```

---

## 5. Níveis de lint (`[workspace.lints]`)

```toml
[workspace.lints.rust]
unsafe_code             = "deny"   # as crates puras usam #![forbid] no lib.rs
missing_docs            = "warn"
unused_qualifications   = "deny"
unused_lifetimes        = "deny"
elided_lifetimes_in_paths = "deny"
single_use_lifetimes    = "deny"
trivial_casts           = "deny"
trivial_numeric_casts   = "deny"
unreachable_pub         = "warn"
unsafe_op_in_unsafe_fn  = "deny"
let_underscore_drop     = "deny"
macro_use_extern_crate  = "deny"
meta_variable_misuse    = "deny"
non_ascii_idents        = "deny"
noop_method_call        = "deny"
unused_import_braces    = "deny"
variant_size_differences = "warn"

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"

[workspace.lints.clippy]
# Grupos
all         = "deny"
correctness = "deny"
suspicious  = "deny"
style       = "deny"
complexity  = "deny"
perf        = "deny"
pedantic    = "warn"
nursery     = "warn"
cargo       = "warn"

# Restriction (curadoria orientada pelas políticas R)
unwrap_used                 = "deny"
expect_used                 = "deny"
panic                       = "deny"
todo                        = "deny"
unimplemented               = "deny"
unreachable                 = "deny"
dbg_macro                   = "deny"
print_stdout                = "deny"
print_stderr                = "deny"
exit                        = "deny"
mem_forget                  = "deny"
indexing_slicing            = "deny"
arithmetic_side_effects     = "warn"
as_conversions              = "deny"
cast_possible_truncation    = "deny"
cast_sign_loss              = "deny"
cast_possible_wrap          = "deny"
cast_precision_loss         = "warn"
undocumented_unsafe_blocks  = "deny"
multiple_unsafe_ops_per_block = "deny"
let_underscore_must_use     = "deny"
let_underscore_untyped      = "warn"
missing_docs_in_private_items = "warn"
absolute_paths              = "warn"
wildcard_imports            = "deny"
min_ident_chars             = "warn"
fn_params_excessive_bools   = "deny"
struct_excessive_bools      = "deny"
excessive_nesting           = "warn"
disallowed_fields           = "deny"
disallowed_macros           = "deny"
disallowed_methods          = "deny"
disallowed_names            = "deny"
disallowed_types            = "deny"
await_holding_lock          = "deny"
await_holding_refcell_ref   = "deny"
allow_attributes            = "warn"
allow_attributes_without_reason = "deny"

# Tamanho/perf
large_enum_variant          = "deny"
result_large_err            = "warn"
large_types_passed_by_value = "warn"
large_stack_arrays          = "deny"
large_stack_frames          = "warn"
large_futures               = "warn"

# Ruidosas: abrimos mão (documentar)
module_name_repetitions = "allow"
must_use_candidate      = "allow"
implicit_hasher         = "allow"

# Documentação (mantidas como warn)
missing_errors_doc  = "warn"
missing_panics_doc  = "warn"
missing_safety_doc  = "warn"
```

Cada crate herda com `[lints] workspace = true`; `knudge-core`, `knudge-cli` e `knudge-mcp`
acrescentam `#![forbid(unsafe_code)]` no topo.

> **Nota sobre `forbid` vs `deny`:** `forbid` não pode ser sobreposto por `#[allow]`. Por isso o
> workspace usa `unsafe_code = "deny"` e as crates **puras** usam `#![forbid(unsafe_code)]`; se o
> adaptador de embedding precisar de FFI, ele fica como a **única** crate com `unsafe` permitido.

---

## 6. Escapes e higiene do `#[allow]`

- Todo `#[allow]` exige `reason` (`allow_attributes_without_reason = "deny"`), no formato
  `#[allow(clippy::x, reason = "…; ver R..")]`.
- Escapes previstos: `HashMap`/`HashSet` no adaptador de índice; `std::process::exit` no `main`;
  `std::env::*`/`SystemTime` no adaptador dos ports; `#[allow(unsafe_code)]` no adaptador.
- `#[expect]` (Edition 2024) é preferível a `#[allow]` quando o lint deveria realmente disparar:
  se parar de disparar, o `#[expect]` vira erro (evita supressão zumbi).

---

## 7. Integração com o plano

- **E01-T03** — copiar [`clippy.toml`](clippy.toml) e o bloco `[workspace.lints]` (§5) para a raiz.
- **E01-T08** — `unsafe`: `deny` no workspace + `forbid` nas crates puras; `disallowed-types`
  reforça R03.
- **E01-T09 / E11-T10** — `await-holding-invalid-types` e `large-futures` sustentam R11/R16.
- **E12-T07/T08** — `print_stdout`/`print_stderr` ligam R20; `exit` e o mapa de erro ligam R31.
- **E13-T07** — o CI roda `cargo clippy --workspace --all-targets -- -D warnings` lendo
  `clippy.toml`.
- **`14_revisao_tecnica.md`** — este documento é a política **R44** (rigor de Clippy).

---

## 8. Riscos e adoção

- **Volume inicial de warnings**: os grupos `pedantic`/`nursery` geram muitos. Estratégia:
  aterrissar por grupos (primeiro `all`+`perf`+`correctness`+restriction curada; depois
  `pedantic`; `nursery` por último), sempre com `#[expect]`/allowlist + `reason`.
- **Estado do 1º aterrissamento (E01):** grupos com `priority = -1` (para permitir overrides
  pontuais, corrigindo `lint_groups_priority`); `pedantic`/`nursery` em `warn`. Temporariamente
  em `allow` (baixo valor/ruído, reavaliar quando os módulos estabilizarem): `min_ident_chars`,
  `many_single_char_names`, `missing_docs_in_private_items`, `missing_errors_doc`,
  `missing_panics_doc`, `allow_attributes`, `option_if_let_else`, `significant_drop_tightening`,
  `missing_const_for_fn`, `excessive_nesting` e `multiple_crate_versions` (`clippy::cargo`).
  `allow_attributes_without_reason` permanece `deny`: todo `#[allow]` exige `reason`.
- **`disallowed-types` com `HashMap`** é a regra mais invasiva; se o índice exigir `HashMap`
  por desempenho, manter o `allow` local com `reason` e um teste de determinismo.
- **`max-fn-params-bools = 0`** força modelagem por enums; se doer, relaxar para `1` antes de
  espalhar `#[allow]`.
- **MSRV**: `msrv = "1.97"` é **obrigatório** neste projeto e precisa casar com o
  `rust-version = "1.97"` do `Cargo.toml`, senão a lint `incompatible_msrv` fica inconsistente.
- **Config mutável = contrato**: `clippy.toml` entra em revisão de código; mudar um limite exige
  justificar no PR.
