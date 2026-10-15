# 14 — Unsafe Rust, Macros e FFI

## Unsafe Rust

Unsafe não desabilita o borrow checker — dá superpoderes adicionais:

```rust
unsafe fn dangerous() {}
unsafe trait UnsafeTrait {}
unsafe impl UnsafeTrait for MyType {}

// 5 superpoderes:
```

### 1. Dereferenciar Raw Pointers

```rust
let mut num = 5;
let r1 = &num as *const i32;         // raw pointer imutável
let r2 = &mut num as *mut i32;       // raw pointer mutável

unsafe {
    println!("r1: {}", *r1);          // deref
    *r2 = 10;
}

// Raw pointers podem ser nulos
let ptr: *const i32 = std::ptr::null();
```

### 2. Chamar Funções Unsafe (incluindo FFI)

```rust
extern "C" {
    fn abs(input: i32) -> i32;       // libc
}

unsafe {
    println!("abs(-3) = {}", abs(-3));
}
```

### 3. Acessar/Modificar Static Mutáveis

```rust
static mut COUNTER: u32 = 0;

fn add_to_count(inc: u32) {
    unsafe {
        COUNTER += inc;
    }
}
```

### 4. Implementar Unsafe Traits

```rust
unsafe trait MyUnsafeTrait {}

unsafe impl MyUnsafeTrait for MyType {}
```

### 5. Acessar Campos de Union

```rust
#[repr(C)]
union MyUnion {
    i: i32,
    f: f32,
}

let u = MyUnion { i: 42 };
unsafe {
    println!("{}.", u.i);
}
```

### Padrão Seguro sobre Unsafe

```rust
use std::slice;

fn split_at_mut<T>(values: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    assert!(mid <= len);

    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

### Miri (UB Detector)

```bash
cargo +nightly miri test    # detecta undefined behavior
```

## FFI (Foreign Function Interface)

### Chamando C de Rust

```rust
// Declarar funções externas
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

// Linking
#[link(name = "ssl")]
extern "C" { ... }

// Tipos compatíveis com C
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

// Strings C
use std::ffi::{CStr, CString};
let c_str = CString::new("hello").unwrap();
unsafe { println!("{}", strlen(c_str.as_ptr())); }
```

### Exportando Rust para C

```rust
#[no_mangle]
pub extern "C" fn call_from_c() -> i32 {
    println!("called from C");
    42
}

// Previne otimização (mingw / cdylib)
// Cargo.toml: crate-type = ["cdylib"]
```

### Bindgen (auto-generate bindings)

```toml
[build-dependencies]
bindgen = "0.70"
```

```rust
// build.rs
fn main() {
    println!("cargo::rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings");
}
```

## Macros

### macro_rules! (declarative macros)

```rust
// Macro simples
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

// Com argumentos
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("called {}", stringify!($func_name));
        }
    };
}
create_function!(foo);  // gera fn foo()

// Designators
// ident, expr, ty, pat, stmt, block, literal, tt, path, meta, vis
macro_rules! calculate {
    (eval $e:expr) => {{
        let val: usize = $e;
        println!("{} = {}", stringify!($e), val);
        val
    }};
}

// Repetition
macro_rules! vec {
    ( $( $x:expr ),* $(,)? ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

// Variadic
macro_rules! find_min {
    ($x:expr) => ($x);
    ($x:expr, $($y:expr),+) => (
        std::cmp::min($x, find_min!($($y),+))
    );
}
```

### Procedural Macros

```rust
// Três tipos: custom derive, attribute-like, function-like

// Cargo.toml (macro crate)
[lib]
proc-macro = true

// custom_derive
#[proc_macro_derive(MyDerive)]
pub fn my_derive(input: TokenStream) -> TokenStream { ... }

// attribute
#[proc_macro_attribute]
pub fn log_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("calling: {item}");
    item
}

// function-like
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream { ... }

// Exemplo: derive macro básica
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}", stringify!(#name));
            }
        }
    };
    gen.into()
}
```

## Inline Assembly

```rust
use std::arch::asm;

unsafe {
    asm!("nop");  // no-op instruction

    // Inputs, outputs, clobbers
    let x: u64 = 3;
    let y: u64;
    asm!("add {0}, {1}", inout(reg) x => y, in(reg) 5);

    // 128-bit integers via vector registers em x86 (estabilizado 1.99)
    let low: u64 = 0;
    let high: u64 = 0;
    asm!("cpuid", in("eax") 1, out("eax") low, out("ebx") high);
}

// global_asm! considera global target features (1.99)
core::arch::global_asm!(
    ".global my_symbol",
    "my_symbol:",
    "nop",
    options(att_syntax)
);
```

## C-variadic Functions (1.99+)

Rust 1.99.0 estabiliza a **definição** de funções variádicas com ABI
`"C"` e `"C-unwind"`. Antes já era possível *chamar* funções variádicas
externas (ex.: `libc::printf`); agora elas podem ser escritas em Rust:

```rust
/// SAFETY: deve ser chamada com (pelo menos) 2 argumentos i32.
unsafe extern "C" fn sum(mut args: ...) -> i32 {
    // SAFETY: garantido pelo chamador.
    let a = unsafe { args.next_arg::<i32>() };
    let b = unsafe { args.next_arg::<i32>() };
    a + b
}

fn foo() -> i32 {
    unsafe { sum(0i32, 2i32) }
}
```

- O tipo de `...` é `core::ffi::VaList`, compatível em ABI com `va_list`
  do C em todos os targets.
- A leitura de cada argumento é feita via `VaList::next_arg::<T>()`, que é
  **unsafe** (o chamador deve garantir o tipo correto).
- Os tipos que podem ser lidos de um `VaList` são restringidos pelo trait
  `VaArgSafe`.

Também foi estabilizado o suporte a **naked variadic functions** com ABIs
não-`"C"`, que precisam ser escritas via inline assembly:

```rust
#[unsafe(naked)]
pub unsafe extern "C" fn my_naked_function() {
    asm!("ret", options(noreturn));
}
```

> Ver a Referência da linguagem para detalhes de c-variadic functions.

## Layout de Raw Pointers (1.99+)

A 1.99.0 fixa os requisitos de segurança para obter **tamanho e
alinhamento** a partir de raw pointers, tanto para tipos `Sized` (já
possível em stable) quanto **não-`Sized`**. Três funções foram
estabilizadas:

```rust
use core::alloc::Layout;
use core::mem;

let value: u32 = 42;
let ptr: *const u32 = &value;

// Layout de um valor a partir de um raw pointer
let layout = unsafe { Layout::for_value_raw(ptr) };

// Tamanho e alinhamento a partir de raw pointers a tipos ?Sized
let size = unsafe { mem::size_of_val_raw(ptr) };
let align = unsafe { mem::align_of_val_raw(ptr) };
```

Para tipos `?Sized`, como `[T]` ou `dyn Trait`, o ponteiro precisa ser um
"fat pointer" válido (com metadados de length/vtable) para que as
funções sejam seguras:

```rust
let slice: &[u8] = &[1, 2, 3];
let ptr: *const [u8] = slice;
let size = unsafe { mem::size_of_val_raw(ptr) }; // 3
```

> **Segurança**: `size_of_val_raw`/`align_of_val_raw`/`for_value_raw`
> são `unsafe` para `?Sized` porque dependem dos metadados do ponteiro
> estarem válidos. Para tipos `Sized`, o equivalente seguro
> (`size_of_val`/`align_of_val`/`Layout::for_value`) já existe.

## Lints de Runtime Symbols (1.98+)

```rust
// deny-by-default: invalid_runtime_symbol_definitions
// Detecta definições de símbolos runtime como memcmp, memset, strlen
// Exemplo de código que seria rejeitado:
// #[no_mangle] pub extern "C" fn memcmp(...) { ... }  // ERRO!

// warn-by-default: suspicious_runtime_symbol_definitions
// Para símbolos suspeitos que podem conflitar com o runtime

// warn-by-default: c_void_returns
// Detecta core::ffi::c_void como tipo de retorno
// Prefira () em vez de c_void em FFI
```

## Boas Práticas (Unsafe)

1. **Encapsular unsafe em safe abstractions** — a função safe deve ser
   a API
2. **Mínimo de código unsafe** — cada `unsafe { }` deve ser pequeno e
   justificado
3. **Documentar invariantes** — // SAFETY: explica por que é safe
4. **Rodar Miri em CI** se usar unsafe extensivamente
5. **Preferir raw pointers a referências** em código unsafe (controle
   fino)
6. **assert! invariantes** antes de código unsafe
7. **Unsafe não desabilita borrow checker** — só dá poderes extras
8. **Evitar transmute** — prefira conversões seguras
9. **repr(C)** para compatibilidade com C
10. **macro_rules! para código repetitivo**, proc macros para
    transformações complexas
