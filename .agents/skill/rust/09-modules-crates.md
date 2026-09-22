# 9 — Módulos, Pacotes e Crates

## Pacotes e Crates

- **Crate**: menor quantidade de código que o compilador considera.
  Pode ser binário (`main.rs`) ou biblioteca (`lib.rs`).
- **Pacote**: um ou mais crates com um `Cargo.toml`. Pode ter 0 ou 1
  library crate + vários binary crates.

```
my_package/
├── Cargo.toml               # define o pacote
├── src/
│   ├── lib.rs               # library crate root
│   ├── main.rs              # binary crate root (mesmo nome da lib)
│   └── bin/                 # binary crates adicionais
│       ├── server.rs        # binário "server"
│       └── cli.rs           # binário "cli"
```

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"

# automaticamente: lib + bin com mesmo nome
# bin extras:
[[bin]]
name = "server"
path = "src/bin/server.rs"
```

## Módulos: Organização Interna

### Definição

Módulos organizam o código dentro de um crate:

```rust
// src/lib.rs
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        fn seat_at_table() {}       // privado
    }

    mod serving {
        fn take_order() {}
    }
}
```

### Árvore de Módulos

```
crate
 └── front_of_house
     ├── hosting
     │   └── add_to_waitlist
     └── serving
         └── take_order
```

### Visibility (`pub`)

| Modificador | Visível em |
|-------------|------------|
 | (nada) | privado (módulo pai e irmãos) |
 | `pub` | qualquer um |
 | `pub(crate)` | dentro do crate |
 | `pub(super)` | módulo pai |
 | `pub(in path)` | path específico |

```rust
mod outer {
    fn private() {}
    pub fn public() {}
    pub(crate) fn crate_visible() {}

    mod inner {
        pub(super) fn parent_visible() {} // visível em outer
        pub(in crate::outer) fn specific() {}
    }
}
```

### Struct Visibility

```rust
mod building {
    pub struct Window {
        pub width: u32,      // público
        height: u32,         // privado — só criado internamente
    }

    impl Window {
        pub fn new(w: u32, h: u32) -> Window {  // construtor público
            Window { width: w, height: h }
        }
    }
}
```

## Paths: Referenciando Itens

```rust
// Absoluto (começa com crate)
crate::front_of_house::hosting::add_to_waitlist();

// Relativo (self, super)
self::front_of_house::hosting::add_to_waitlist();
super::serve_order();
```

### use Keyword

```rust
// Traz path para o escopo
use crate::front_of_house::hosting;
hosting::add_to_waitlist();

// Item diretamente
use crate::front_of_house::hosting::add_to_waitlist;
add_to_waitlist();

// Alias
use std::fmt::Result as FmtResult;

// Re-export
pub use crate::front_of_house::hosting; // público!

// Nested paths
use std::{cmp::Ordering, io::{self, Write}};

// Glob
use std::collections::*;  // evite em produção
```

## Separando em Arquivos

```rust
// src/lib.rs
mod front_of_house;       // procura front_of_house.rs ou
                          // front_of_house/mod.rs

// src/front_of_house.rs ou src/front_of_house/mod.rs
pub mod hosting;          // procura hosting.rs ou
                          // front_of_house/hosting.rs

// src/front_of_house/hosting.rs
pub fn add_to_waitlist() {}
```

### Estrutura recomendada (Rust Edition 2021+)

```
src/
├── lib.rs               # crate root
├── main.rs              # binário (opcional)
├── front_of_house/
│   ├── mod.rs           # mod front_of_house
│   └── hosting.rs       # pub mod hosting
└── serving.rs           # mod serving
```

## Workspaces (Múltiplos Pacotes)

```toml
# Cargo.toml (workspace root — sem [package])
[workspace]
members = [
    "crates/backend",
    "crates/frontend",
    "crates/common",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = "1"
```

```toml
# crates/common/Cargo.toml
[package]
name = "common"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
```

```bash
cargo build -p backend     # build específico
cargo test -p common       # test específico
```

## Features

```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]      # dep: prefix evita feature implícita
full = ["std", "serde", "async"]

# Optional dependencies criam features implícitas
[dependencies]
serde = { version = "1", optional = true }
```

```rust
// Em runtime
#[cfg(feature = "serde")]
impl Serialize for MyType { ... }

#[cfg(not(feature = "std"))]
extern crate alloc;

// Max 300 features (crates.io)
```

## Conditional Compilation

```rust
#[cfg(target_os = "linux")]
fn platform_specific() { /* linux */ }

#[cfg(not(target_os = "windows"))]
fn fallback() {}

#[cfg(any(unix, target_os = "wasi"))]
fn unix_like() {}

#[cfg(feature = "nightly")]
fn uses_nightly() {}

// cfg! macro — runtime check
if cfg!(target_arch = "wasm32") {
    // web-specific branch
}
```

## Boas Práticas

1. **Estrutura de módulos reflete a API pública** — não a organização
   interna
2. **`pub use`** para exportar API limpa — esconda detalhes internos
3. **Privado por default** — só exponha o necessário
4. **Préfira paths absolutos (`crate::`)** — mais fáceis de mover
5. **Workspaces** para projetos grandes (monorepo)
6. **Features** para funcionalidades opcionais (sem breaking)
7. **`dep:` prefix** (Edition 2021+) para dependências opcionais
8. **Evite glob imports** — polui o namespace
9. **Um módulo por arquivo** — arquivos grandes (>400 linhos) devem
   ser divididos
10. **`build.rs`** para geração de código, linking com C, ou
    detecção de plataforma
