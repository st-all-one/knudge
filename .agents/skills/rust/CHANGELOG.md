# Rust — Changelog 1.97.0 → 1.100.0

Síntese das mudanças relevantes entre a versão base do guia (1.97.0) e a
atual (1.100.0). Fontes: [releases.rs](https://releases.rs/), os anúncios
oficiais do Rust Release Team e a documentação de `std`/`cargo`/`rustc`.

---

## 1.100.0 — 12 Nov 2026 (minor)

### Never type (`!`) — estável

- `!` pode ser **nomeado em qualquer posição** (antes só no retorno de
  funções): `Result<T, !>`, campos de struct, argumentos genéricos.
- Coerção **never-to-any** (`!` → qualquer `T`) mantida. `return`,
  `break`, `continue`, `panic!`, `process::exit` e `loop {}` têm tipo `!`.
- **`Infallible` virou `pub type Infallible = !`** — APIs que usavam
  `Infallible` passam a interoperar com `!`.
- **Breaking (todas as editions)**: o *fallback* de `!` mudou de `()`
  para `!` também em 2015/2018/2021 (2024 já era `!`). O caso clássico é
  `f()?;` com `f` genérico: `T` passa a ser inferido como `!`
  (`!: Default`), exigindo anotação (`f::<T>()?`). O lint de migração
  `dependency_on_unit_never_type_fallback` foi removido.
- Impls estáveis para `!`: `Clone`, `Copy`, `Debug`, `Display`, `Eq`,
  `Hash`, `Ord`, `PartialEq`, `PartialOrd`, `Not`, `Error`,
  `Termination`, `AsRef<T>`, `AsMut<T>` (as duas últimas novas).
- Um `match` é exaustivo sem cobrir variantes com campo inabitado
  (`Miu(!)`, `Nya(u32, !)`, `Mjau(Void)`), mas `Option<!>` continua
  habitado por `None`.
- Clippy: `unused_async_trait_impl` ignora impls para `!`.

### `Allocator` API (`allocator_api`) — núcleo estável

- Trait `std::alloc::Allocator` (`unsafe trait`): `allocate` e
  `deallocate` obrigatórios; `allocate_zeroed`, `grow`, `grow_zeroed`,
  `shrink` com default. `AllocError` estável.
- `Global` (delega ao `#[global_allocator]` ou ao default do `std`) e
  `System` (`malloc`/`HeapAlloc`) implementam o trait.
- Impls de `Allocator` para `&A`, `&mut A`, `Box<T, A>`, `Rc<T, A>` e
  `Arc<T, A>`; esses wrappers são *equivalentes* ao alocador interno.
- `Box`: `new_in`, `from_raw_in`, `from_non_null_in`,
  `into_raw_with_allocator`, `into_non_null_with_allocator`, `allocator`.
- `Vec`: `new_in`, `with_capacity_in`, `from_raw_parts_in`,
  `from_parts_in`, `into_raw_parts_with_allocator`,
  `into_parts_with_allocator`, `allocator`.
- Só `Vec` e `Box` aceitam `A` por enquanto; `String`, `HashMap`,
  `VecDeque` etc. ficam para o projeto "Allocators 1.0".
- `Allocator` permite alocação ZST (≠ `GlobalAlloc`), não pode fazer
  unwind e exige que o bloco permaneça *currently allocated* até
  desalocar/realocar.
- **Compatibilidade**: algumas impls de `Box` foram generalizadas para
  `A: Allocator`, o que pode causar novos erros de inferência.

### Reuso de alocação em `Box`/`Rc`/`Arc`

- `Box::take(Box<T>) -> (T, Box<MaybeUninit<T>>)` e `Box::write` reusam
  o slot (sem realocar).
- `Box::map`, `Rc::map`, `Arc::map` transformam o valor reusando a
  alocação quando possível. São funções associativas para não colidir
  com métodos do tipo interno.

### `DropGuard` (`core::mem`)

- `DropGuard::new(value, f)` roda `f` no drop, em qualquer saída
  (`return`, `?`, panic); `Deref`/`DerefMut` para o valor.
- `DropGuard::dismiss(guard)` devolve o valor sem rodar a closure —
  preferível a `mem::forget`, que vazaria as capturas.
- Habilita rollback all-or-nothing sem `impl Drop` dedicado.

### `conjure_zst` (`core::mem`)

- `unsafe fn conjure_zst<T>() -> T` cria um ZST **inhabited** sem
  `Default`/`Clone`.
- Substitui `transmute(())` e `MaybeUninit::uninit().assume_init()`.
- Panics se `size_of::<T>() != 0`; UB para `!`, enums vazios e tokens
  com invariantes (ex.: `Python<'py>` do PyO3).

### Bibliotecas — demais estabilizações

- `bool::toggle`.
- `str::trim_prefix`/`trim_suffix`; `[T]::trim_prefix`/`trim_suffix`;
  `Path::trim_prefix` (não há `Path::trim_suffix`).
- `Vec::from_fn`.
- `UnsafeCell::replace`, `UnsafeCell::as_ref_unchecked`,
  `UnsafeCell::as_mut_unchecked` (as duas últimas `const`).
- `impl PartialEq<Cow<'_, [U]>> for Vec<T, A>`, `&[T]` e `&mut [T]`.
- `impl Default for std::sync::Once`.
- `ChildExt::main_thread_handle` (Windows).
- `os::windows::process::CommandExt::show_window`.
- Mapeamento de `ENOTSUP` para `ErrorKind::Unsupported`.
- Unicode 18.0.0 na biblioteca padrão.

### Linguagem / compiler

- `extern "custom"`: ABI desconhecida; função `unsafe`, sem parâmetros,
  retorno `()` explícito e `#[unsafe(naked)]`; chamável só via `asm!`.
- Lint `repeated_reprs` (warn) para `#[repr(..)]` duplicado.
- `aligned_fields_in_packed` (deny): campo `repr(align)` dentro de
  `repr(packed)` virou lint (antes erro rígido).
- `dropping_references`, `dropping_copy_types` e
  `undropped_manually_drops` estendidos a `ptr::drop_in_place`.
- Closures capturam com mais precisão variáveis usadas em `if let`
  (pode quebrar código que dependia da captura ampla).
- Checagem de *dyn-compatibility* em mais posições.
- Padrões `box x` não são mais aceitos sintaticamente.
- `let-else` respeita o agrupamento de `expr` de `macro_rules`.
- `str::escape_debug` sempre escapa grapheme extenders.
- `Iterator::{min,max,min_by_key,max_by_key}` usam outros métodos de
  `Ord` (só afeta impls inconsistentes).
- Test harness aceita `--test-threads`/`--color` repetidos (o último
  vence).
- `-Cprofile-sample-use=<.prof>` estável (SPGO).
- SSE/AVX em target x86 softfloat viram erro; ARMv7-A hardfloat perdeu
  `d32` do baseline (use `-Ctarget-feature=+d32`).
- ABI "Rust" em x86 usa registradores SSE quando sempre disponíveis;
  `windows-gnullvm` linka libunwind estaticamente.

### Platform support

- `i686-pc-windows-gnu`: Tier 2 **sem host tools**.
- `wasm32-wasip3`: promovido a **Tier 2**.
- `powerpc64-sony-ps3`: novo **Tier 3**.

### Cargo

- **Sistema de lints próprio** (`[lints.cargo]`): `unused_dependencies`
  (style/warn), `unused_workspace_dependencies`,
  `unused_workspace_package_fields`, `missing_lints_inheritance`,
  `blanket_hint_mostly_unused`, `manual_readme`; grupos
  `cargo::{default,correctness,style,suspicious,pedantic,restriction}`.
- **`min-publish-age`** (RFC 3923): `[registry]
  global-min-publish-age`, override por registry e
  `resolver.incompatible-publish-age` (`deny` default, `allow` desliga);
  `CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow` para um update
  pontual; `cargo install` ignora o cooldown.
- **Novo layout do `build-dir`**: hash de configuração aplicado no
  diretório pai e organização por etapa de build. Layout é interno e
  instável — ferramentas devem usar `--message-format=json`; ver
  cargo#17287.
- `cargo install` usa o `Cargo.lock` embutido no `.crate`; `--locked`
  virou no-op.
- Chaves `[target]` com `.` no nome precisam de aspas
  (`[target."thumbv8m.main-none-eabihf"]`).
- `cargo add` respeita symlink relativo do `Cargo.toml`.
- `cargo clean` considera hardlinks no tamanho; `cargo run` no Windows
  não sobrescreve erro quando a saída termina em `\r`.
- `build.warnings='deny'` mostra contagem de erros no resumo.

### Rustdoc / Clippy

- Rustdoc: novo lint `invalid_markdown_table` (warn) para `|` não
  escapado em tabelas de doc comments.
- Clippy: novos `option_zip_none` (suspicious) e
  `needless_nonzero_get` (complexity).
- Clippy (melhorias): `map_unwrap_or` sugere `map_or_default`;
  `unnecessary_fold` em `Option` sugere `map_or`; `unnecessary_map_or`
  sugere `Result::is_ok`/`is_err`; `redundant_clone` detecta loops sem
  alias; `imprecise_flops` sugere `hypot`; `manual_swap` e
  `suboptimal_flops` em const; `unwrap_used` com
  `allow-unwrap-in-tests` cobre testes de integração.
- Clippy: correções de falso positivo e ICE (`map_clone`,
  `regex_creation_in_loops`, `let_and_return`, `large_futures`, etc.).

### Compatibilidade (seleção)

- Fallback de `!` em todas as editions e `Infallible = !`.
- Captura de closures em `if let`.
- Mais checagens de dyn-compatibility.
- Padrões `box` removidos.
- Impls de `Box` generalizadas para allocators (inferência).
- `Iterator::min/max` mudam métodos de `Ord`.
- ARMv7-A hardfloat sem `d32` no baseline; SSE/AVX rejeitados em x86
  softfloat.
- Unicode 18.0.0.

## 1.97.1 — 16 Jul 2026 (patch)

- **rustc**: corrige miscompilação na otimização do LLVM (bump do submódulo
  LLVM + revert da mudança no rustc que disparava o bug).

## 1.98.0 — 20 Ago 2026 (minor)

### Linguagem / Biblioteca de destaque
- **Métodos algébricos de ponto flutuante** (`f32`/`f64`):
  `algebraic_add`, `algebraic_sub`, `algebraic_mul`, `algebraic_div`,
  `algebraic_rem`. Permitem reordenação/otimizações (estilo `-ffast-math`),
  **não-determinísticos** e **sem UB**. Ver `02-fundamentals.md`.
- **Formatação de inteiros em buffer**: `format_into(&mut NumBuffer<Self>)`
  em todos os inteiros primitivos; `core::fmt::NumBuffer`. Performance
  comparável ao `itoa`.
- **`ManuallyDrop` × `Box`**: garantia estável de que mover
  `ManuallyDrop<Box<_>>` após o drop **não é UB** (RFC 3336). Ver
  `10-smart-pointers.md`.
- **`derive` em `{core,std}::derive`**: estabilizado explicitamente (MSRV
  1.96, não 1.98).

### Linguagem
- **Lifetime shortening em unsize-coercing**: encurtar lifetime de `&mut`
  agora funciona em posições invariantes (ex.: `Cell<&'long mut i32>` →
  `Cell<&'short mut dyn Send>`). Ver `03-ownership.md`.
- Novos lints:
  - `invalid_runtime_symbol_definitions` (**deny**) — símbolos runtime como
    `memcmp`, `memset`, `strlen`.
  - `suspicious_runtime_symbol_definitions` (**warn**).
  - `c_void_returns` (**warn**) — `core::ffi::c_void` como tipo de retorno.

### Platform Support
- Novos Tier 3: `powerpc64-unknown-linux-gnuelfv2`,
  `aarch64-unknown-linux-pauthtest`.
- Promovidos a Tier 2: `thumbv7a-none-eabi`, `thumbv7a-none-eabihf`,
  `thumbv7r-none-eabi`, `thumbv7r-none-eabihf`, `thumbv8r-none-eabihf`.

### APIs estabilizadas
`str::substr_range`, `[T]::subslice_range`, `core::fmt::NumBuffer`,
`<{integer}>::format_into`, `Send/Sync` para `std::process::CommandArgs`,
`{fN}::algebraic_{add,sub,mul,div,rem}`, `NonZero<{integer}>::from_str_radix`,
`String::from_utf16le{,_lossy}`, `String::from_utf16be{,_lossy}`,
`[T]::strip_circumfix`, `str::strip_circumfix`, `Atomic<T>::from_mut`,
`Atomic<T>::get_mut_slice`, `Atomic<T>::from_mut_slice`, `std::range::legacy`,
`bool::ok_or`, `bool::ok_or_else`, LoongArch CRC intrinsics.

### Compatibility Notes (seleção)
- Lifetime bounds de trait objects (fully elided) podem resolver diferente
  ou ser rejeitados em cenários raros.
- `ambiguous_glob_imports` vira **hard error** em alguns casos.
- Where-bounds `Type = Type` e `Type == Type` deixam de ser sintaticamente
  válidos.
- `repr(transparent)` mais estrito: `repr(C)`, campos privados e
  `#[non_exhaustive]` **não** são mais "triviais".
- `transmute()` verifica tamanho corretamente com `repr`.
- `Send/Sync` **não** implementados para `std::env::Vars{,Os}`.
- Fast path de `derive(PartialOrd)` ao derivar `Ord` pode quebrar crates
  com impls inconsistentes.
- `assert_eq!`/`assert_ne!` ganham escopo temporário.
- Emscripten: WASM exception handling ABI incondicional (flag
  `-Zemscripten-wasm-eh=false` removida).
- Solaris: `File::lock` removido (retorna "unsupported").
- `UNSAFE_CODE` lint consistente para todos os atributos unsafe.
- Windows-gnu: baseline de versões de tools especificada.
- rustfmt descobre módulos definidos em `cfg_select!`.

## 1.98.1 — 3 Set 2026 (patch)

- **rustc**: corrige miscompilação na geração de vtables.

## 1.99.0 — 1 Out 2026 (minor)

### Linguagem / FFI de destaque
- **Variádicas `extern "C"`**: estabilizada a *definição* de funções
  variádicas com ABIs `"C"` e `"C-unwind"`. Tipo de `...` é
  `core::ffi::VaList` (ABI-compatível com `va_list`); leitura via
  `VaList::next_arg::<T>()` (unsafe), restringida pelo trait `VaArgSafe`.
  Ver `14-unsafe-macros-ffi.md`.
- **Naked variadic functions** com ABIs não-`"C"` (via inline asm).
- **Layout de raw pointers**: estabilizadas `Layout::for_value_raw`,
  `mem::size_of_val_raw`, `mem::align_of_val_raw` (para `Sized` e
  não-`Sized`).
- **`#[my_macro] mod foo;`**: permite outlined modules em body de custom
  attribute / derive macro.
- **128-bit integers via vector registers** com `asm!` em x86.

### Linguagem
- Novo lint `raw_borrows_via_references` (**allow**).
- `unconditional_panic` estendido a `chunks`/`windows` com size zero.
- POSIX symbols adicionados aos lints de runtime symbols.
- Lint de `#[path]` não usado em inline modules.
- `unreachable_cfg_select_predicates` no grupo `unused`.
- `overflowing_literals` corrigido para negação repetida (ex.: `--128_i8`).
- Trait methods resolvidos em adjusted never type (FCW).
- Coerção de inference variables para trait objects quando related via
  subtyping a tipo `Sized`.
- Garantia: conteúdo de `UnsafeCell` acessível sem `get`
  (`invalid_reference_casting` ajustado).
- `global_asm!` considera global target features.

### Compiler
- `-Ctarget-cpu` vira target-modifier para AVR, AMDGCN, NVPTX.
- `static_position_independent_executables` em todos os gnu targets.
- Sugestões de métodos preferem `doc alias` exato sobre similaridade.
- **Internal**: update para **LLVM 23**.

### Platform Support
- `riscv64-unknown-linux-musl` promovido a Tier 2 (com host tools).

### Libraries
- Iteração em `RangeInclusive` melhor otimizada (comportamento de ranges
  exaustos pode mudar — não garantido).
- `transmute_copy` relaxado para `?Sized` e usa non-unwinding panic.
- U+FF9E/U+FF9F não escapados em `escape_debug_ext`.
- `core::fmt::NumBuffer` re-exportado em `alloc`/`std`.

### APIs estabilizadas
`IntoIterator` para `Box<[T; N]>` e refs, `VecDeque::retain_back`,
`core::ffi::VaList`, `Box::into_non_null`, `Box::from_non_null`,
`Vec::into_parts`, `Vec::from_parts`, `core::mem::size_of_val_raw`,
`core::mem::align_of_val_raw`, `core::alloc::Layout::for_value_raw`,
`String::from_utf8_lossy_owned`, `FromUtf8Error::into_utf8_lossy`,
`FusedIterator` para `StepBy<I>`, `std::fs::set_times`,
`std::fs::set_times_nofollow`.

### Cargo
- Novo profile built-in **`debug`** (preparação para transição do `dev`).
- Workspace members em edition 2024+ podem override `default-features` de
  dependência herdada (RFC 3945).
- **Incremental compilation desabilitada por default em CI** (env `CI`).

### Rustdoc
- Novo lint `unused_footnote_definition`.
- Filtragem de trait impls mais inteligente (~20% média, até 40%).

### Compatibility Notes (seleção)
- Legacy integral modules **deprecados** (`std::i32::MAX` → `i32::MAX`).
- `no_mangle_generic_items` vira **hard error**.
- `Pin::new_unchecked` teve safety invariants levemente alterados.
- Não promover references para extern statics.
- `#[repr(simd)]` não é mais aceito acidentalmente em macro invocations.
- Attributes não aplicáveis são error em code blocks de doc comments.
- `doc(cfg())` não considerado ao filtrar doctests.
- Anonymous lifetimes em associated consts inferidos como `'static`.
- `semicolon_in_expressions_from_non_local_macros` para macros de outras
  crates (lint novo separado).
- Rust Reference não recomenda mais single pattern para match de union
  (risco de UB ao ler union antes de checar o resto do scrutinee).

## 1.99.1 — 15 Out 2026 (patch)

- **library**: corrige símbolo `wcslen` ausente em UEFI e embedded targets.
- **rustc**: corrige miscompilação de retornos FFI `bool` em x86_64.
- **rust-lld**: adiciona rpath para `libLLVM` em macOS.

---

## Impacto nos arquivos do guia

| Arquivo | Mudanças |
|---|---|
| `00-introduction.md` | Versão 1.100.0, destaques |
| `01-toolchain.md` | SPGO (`-Cprofile-sample-use`), Tier 2/3, LLVM 23, lints unsafe |
| `02-fundamentals.md` | Never type nomeável, `bool::toggle`, métodos algébricos, `format_into` |
| `03-ownership.md` | `DropGuard`, lifetime shortening |
| `04-structs-enums.md` | Variantes inabitadas (`!`), lints de `repr`, `box` patterns removidos |
| `05-collections.md` | `Vec::from_fn`, `trim_prefix`/`trim_suffix`, constructors de allocator, `Cow`, Unicode 18 |
| `06-error-handling.md` | `Result<T, !>`/`Infallible`, fallback de `!`, `DropGuard`, `ENOTSUP` |
| `07-generics-traits.md` | `!` em genéricos e impls, caveat de `impl Trait` |
| `08-closures-iterators.md` | Captura em `if let`, `min`/`max` e `Ord`, `StepBy` |
| `09-modules-crates.md` | `[workspace.lints.cargo]` |
| `10-smart-pointers.md` | `Allocator` API, `Box`/`Rc`/`Arc` map/take, `UnsafeCell`, `Cow`, lints de drop |
| `11-concurrency.md` | `Once: Default`, `Atomic<T>::*` |
| `12-async.md` | Sem mudanças na 1.100 |
| `13-testing.md` | `invalid_markdown_table`, flags repetíveis, `doc(cfg())` |
| `14-unsafe-macros-ffi.md` | `extern "custom"`, `conjure_zst`, C-variadics, layout de raw pointers |
| `15-cargo-deep.md` | Cargo lints, `min-publish-age`, build-dir, `cargo install` |
| `16-project-practical.md` | CI, cooldowns, processos (Windows), allocators |
