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
}
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
