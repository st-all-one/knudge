# 13 — Testes

## Tipos de Teste

| Tipo | Localização | Propósito |
|------|-------------|-----------|
| Unitários | `src/` (inline) | Testar unidades individuais |
| Integração | `tests/` | Testar API pública |
| Doc Tests | Doc comments | Exemplos executáveis |
| Benchmarks | `benches/` | Performance |

## Testes Unitários

```rust
// src/lib.rs ou no módulo específico

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn with_result() -> Result<(), String> {
        if 2 + 2 == 4 { Ok(()) } else { Err("wrong".into()) }
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn panics() {
        add(u64::MAX, 1);
    }

    #[test]
    #[ignore = "expensive"]
    fn expensive_test() {
        // só roda com cargo test -- --ignored
    }
}
```

### Macros de Asserção

```rust
assert!(condition);
assert!(condition, "custom message: {val}");

assert_eq!(a, b);       // a == b (PartialEq + Debug)
assert_ne!(a, b);       // a != b

// Custom failure message
assert_eq!(left, right, "left: {left}, right: {right}");

// Para tipos sem PartialEq
assert!(a == b, "{a:?} != {b:?}");
```

## Testes de Integração

```rust
// tests/integration_test.rs
use my_crate;  // tratado como crate externo

#[test]
fn test_add() {
    assert_eq!(my_crate::add(2, 2), 4);
}

// tests/common/mod.rs — módulo compartilhado
// tests/common/mod.rs (não compila como test separado)
pub fn setup() -> Config {
    Config { /* ... */ }
}
```

```bash
cargo test                    # todos os testes
cargo test --test integration_test  # só integração
cargo test nome_parcial       # filtra por nome
cargo test -- --test-threads=1
cargo test -- --show-output
```

## Doc Tests

```rust
/// Adiciona dois números.
///
/// # Examples
///
/// ```
/// use my_crate::add;
/// assert_eq!(add(2, 3), 5);
/// ```
///
/// ```rust,should_panic
/// // este teste deve panicar
/// ```
///
/// ```rust,ignore
/// // não compila como teste
/// ```
///
/// ```rust,no_run
/// // compila mas não executa
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

```bash
cargo test --doc              # só doc tests
cargo test                    # doc tests + unitários + integração
```

## Benchmarks

### Standard (nightly)

```rust
#![feature(test)]
extern crate test;
use test::Bencher;

#[bench]
fn bench_add(b: &mut Bencher) {
    b.iter(|| add(1, 2));
}
```

### Criterion (stable — recomendado)

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_bench"
harness = false
```

```rust
// benches/my_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n { 0 => 1, 1 => 1, n => fibonacci(n-1) + fibonacci(n-2) }
}

fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, bench_fib);
criterion_main!(benches);
```

```bash
cargo bench
```

## Test Organization

```rust
// src/lib.rs
pub fn public_api() {}
fn private_helpers() {}

#[cfg(test)]
mod tests {
    use super::*;  // importa items privados também

    #[test]
    fn test_private_helper() {
        private_helpers();  // privado, mas acessível aqui
    }
}
```

### Mocking

```rust
// Sem framework
trait DataSource {
    fn fetch(&self) -> Vec<i32>;
}

struct RealDB;
impl DataSource for RealDB { /* ... */ }

#[cfg(test)]
struct MockDB;
impl DataSource for MockDB {
    fn fetch(&self) -> Vec<i32> { vec![1, 2, 3] }
}

#[cfg(test)]
mod tests {
    fn test_with_mock() {
        let mock = MockDB;
        let result = process(mock);
        assert_eq!(result, 6);
    }
}

// mockall crate (auto-generate mocks)
use mockall::automock;

#[automock]
trait DataSource {
    fn fetch(&self) -> Vec<i32>;
}
```

## Usando `cargo test` Avançado

```bash
cargo test              # todos os testes
cargo test test_name    # filtra por nome
cargo test --lib        # só testes unitários
cargo test --test integ # só integração em tests/integ.rs
cargo test -p my_crate  # workspace: só um crate

# Flags
-- --test-threads=1     # serial
-- --nocapture          # mostra stdout
-- --show-output        # mostra output de testes
-- --ignored            # só testes com #[ignore]
-- --include-ignored    # todos (incluindo ignored)
```

## Para onde vão os testes

```
src/
├── lib.rs              # código principal
└── math.rs             # código principal
    └── ...             # #[cfg(test)] mod tests inline
tests/
├── integration.rs
└── common/
    └── mod.rs          # módulo compartilhado
benches/
└── my_bench.rs
```

## Boas Práticas

1. **Testes unitários no mesmo arquivo** — `#[cfg(test)] mod tests`
2. **`#[should_panic]`** para testar condições de erro
3. **`Result<(), E>`** em testes — mais idiomático que `assert!`
4. **Doc tests são testes de verdade** — mantenha-os atualizados
5. **Testes de integração em `tests/`** — testam API pública
6. **Mock traits** para isolar testes de I/O
7. **`cargo nextest`** é mais rápido que `cargo test` para CI
8. **Teste nomes descritivos** — `test_parse_empty_string` em vez de
   `test1`
9. **Property-based testing com `proptest`** — testa casos aleatórios
10. **Code coverage com `cargo tarpaulin`** ou `cargo-llvm-cov`
