# Rust — Changelog 1.97.0 → 1.99.1

Síntese das mudanças relevantes entre a versão base do guia (1.97.0) e a
atual (1.99.1). Fontes: [releases.rs](https://releases.rs/) e os anúncios
oficiais do Rust Release Team.

---

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
| `00-introduction.md` | Versão 1.99.1 |
| `01-toolchain.md` | LLVM 23, lints unsafe |
| `02-fundamentals.md` | Métodos algébricos de float, `format_into`/`NumBuffer` |
| `03-ownership.md` | Lifetime shortening em posições invariantes |
| `04-structs-enums.md` | `repr(transparent)` mais estrito |
| `05-collections.md` | `from_utf16*`, `strip_circumfix`, `IntoIterator` Box, `retain_back`, `into_parts`/`from_parts` |
| `06-error-handling.md` | lint `c_void_returns` |
| `07-generics-traits.md` | Resolução em never type, coerção de inference vars |
| `08-closures-iterators.md` | `FusedIterator` para `StepBy` |
| `10-smart-pointers.md` | `ManuallyDrop`/`Box`, `Box::into_non_null`/`from_non_null`, `Pin::new_unchecked` |
| `11-concurrency.md` | `Atomic::from_mut`/slices, `Send`/`Sync` de `CommandArgs` |
| `13-testing.md` | `doc(cfg())` em doctests, `unused_footnote_definition` |
| `14-unsafe-macros-ffi.md` | C-variadics (`VaList`/`VaArgSafe`), layout de raw pointers, lints de runtime symbols |
| `15-cargo-deep.md` | Profile `debug`, override de `default-features`, incremental em CI |
| `16-project-practical.md` | CI, profile `debug` |
