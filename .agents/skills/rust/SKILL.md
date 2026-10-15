---
name: rust
description: >
  Rust 1.99.1 / Edition 2024: ownership, borrowing, lifetimes, structs/enums,
  traits/generics, error handling, iterators, smart pointers, concurrency,
  async/Tokio, macros, unsafe/FFI, testing, Cargo. Load when writing,
  reviewing, or debugging Rust or structuring a crate/workspace.
category: languages
version: "1.99.1 (Edition 2024)"
tags: [rust, ownership, borrowing, async, tokio, concurrency, cargo, macros, ffi, testing]
license: MIT
---

# Rust 1.99.1 (Edition 2024)

## Use When
- Writing/reviewing/debugging Rust (systems, CLI, web, libraries)
- Modeling ownership/lifetimes, traits, or async concurrency
- Structuring a crate, workspace, or publishing to crates.io
- Choosing between `Arc`/`Rc`, `Mutex`/`RwLock`, `Box`/`Cow`, channels

## Core Rules
- Default to immutability and borrowing; `clone()` only with justification.
- Error handling: `Result` for recoverable, `panic!` for bugs. Ban `unwrap()` in libraries; use `?` + `thiserror`/`anyhow`.
- Prefer iterators/adapters over index loops; avoid needless `collect()`.
- Model state with enums + `match`; make illegal states unrepresentable.
- Traits: small and cohesive; use generics for static dispatch, `dyn Trait` for heterogeneity.
- Unsafe: isolate and document with a `// SAFETY:` comment; minimize surface.
- Edition 2024: `gen` blocks, `async` closures, `unsafe extern`, RPIT lifetime capture changes.
- Rust 1.98+: `f32`/`f64` `algebraic_*` methods relaxam ordem/precisão (estilo `-ffast-math`); use só em hot loops numéricos, nunca quando reprodutibilidade importa.
- Rust 1.99+: defina variádicas `extern "C"` com `...`/`VaList`/`next_arg`; `#[my_macro] mod foo;`; `Box::into_non_null`.
- No `mem::transmute` unless unavoidable; no leaking abstractions across modules.
- Test: doc tests + unit tests in-module + integration tests in `tests/`.

## Core Patterns
```rust
// Ownership + borrowing
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() >= b.len() { a } else { b } }

// Result + ? + custom error
#[derive(Debug, thiserror::Error)]
enum AppError { #[error("io: {0}")] Io(#[from] std::io::Error), #[error("bad id")] BadId }
fn run() -> Result<(), AppError> { let f = std::fs::read_to_string("x")?; Ok(()) }

// Enums + match
enum Msg { Quit, Move { x: i32, y: i32 }, Write(String) }
match msg { Msg::Quit => {}, Msg::Move{x,y} => {}, Msg::Write(s) => println!("{s}") }

// Shared state: Arc<Mutex<T>> across threads
let state = Arc::new(Mutex::new(0));
let s = Arc::clone(&state);
std::thread::spawn(move || { *s.lock().unwrap() += 1; });

// Async (Tokio)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let h = tokio::spawn(async { 42 });
    println!("{}", h.await?);
    Ok(())
}

// Smart pointers
let b: Box<dyn std::error::Error> = Box::new(std::io::Error::other("x"));
let rc = Arc::new(vec![1,2,3]);            // shared, atomic
let cell = std::cell::RefCell::new(0);     // interior mutability (single thread)
```

## File Map
| File | Content |
|---|---|
| `00-introduction.md` | Language overview, edition 2024 |
| `01-toolchain.md` | rustup, Cargo, targets, components |
| `02-fundamentals.md` | variables, types, control flow, functions |
| `03-ownership.md` | ownership, borrowing, lifetimes, slices |
| `04-structs-enums.md` | structs, enums, pattern matching, impl |
| `05-collections.md` | Vec, HashMap, String, iterators |
| `06-error-handling.md` | Option, Result, `?`, panic, error crates |
| `07-generics-traits.md` | generics, traits, bounds, associated types |
| `08-closures-iterators.md` | closures, iterator adapters, `Fn` traits |
| `09-modules-crates.md` | modules, visibility, crates, workspaces |
| `10-smart-pointers.md` | Box, Rc, Arc, RefCell, Cow, Weak |
| `11-concurrency.md` | threads, Send/Sync, channels, Mutex/RwLock, rayon |
| `12-async.md` | async/await, futures, Tokio, tasks, cancellation |
| `13-testing.md` | unit/integration/doc tests, benches, mocking |
| `14-unsafe-macros-ffi.md` | unsafe, macros, FFI, `extern "C"` |
| `15-cargo-deep.md` | features, profiles, workspaces, publishing |
| `16-project-practical.md` | project structure and patterns |
| `CHANGELOG.md` | mudanças 1.97.0 → 1.99.1 |

## Read Order
Linear for learning (`00`→`16`); for reference jump to the topic. Start at `03-ownership.md` for correctness issues.

## Prereqs
Basic programming; no prior Rust required. Rust 1.99+ / Edition 2024.
